use anyhow::{Context, Result};
use chrono::Utc;
use pcapfile_io::{DataPacket, PcapWriter, WriterConfig};
use std::path::Path;
use std::time::Instant;
use tokio::signal;
use tracing::{debug, error, info};

use crate::cli::NetworkType;
use crate::network::{create_udp_receiver, validate_network_config, ReceiverConfig};
use crate::utils::{ensure_output_directory, format_bytes, format_rate, validate_port};

/// 接收器统计信息
#[derive(Debug, Default)]
struct ReceiverStats {
    packets_received: usize,
    bytes_received: u64,
    errors: usize,
    start_time: Option<Instant>,
}

impl ReceiverStats {
    fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    fn update(&mut self, bytes: usize) {
        self.packets_received += 1;
        self.bytes_received += bytes as u64;
    }

    fn add_error(&mut self) {
        self.errors += 1;
    }

    fn get_duration(&self) -> std::time::Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }

    fn get_rate_bps(&self) -> f64 {
        let duration_secs = self.get_duration().as_secs_f64();
        if duration_secs > 0.0 {
            (self.bytes_received as f64 * 8.0) / duration_secs
        } else {
            0.0
        }
    }

    fn print_progress(&self) {
        let rate_bps = self.get_rate_bps();
        info!(
            "已接收 {} 包, {} 字节, 速率: {}",
            self.packets_received,
            format_bytes(self.bytes_received),
            format_rate(rate_bps)
        );
    }

    fn print_summary(&self) {
        let duration = self.get_duration();
        let rate_bps = self.get_rate_bps();

        info!("接收完成统计:");
        info!("  接收包数: {}", self.packets_received);
        info!("  接收字节: {}", format_bytes(self.bytes_received));
        info!("  错误数量: {}", self.errors);
        info!("  用时: {:.2} 秒", duration.as_secs_f64());
        info!("  平均速率: {}", format_rate(rate_bps));

        if self.packets_received > 0 {
            let avg_packet_size = self.bytes_received / self.packets_received as u64;
            info!("  平均包大小: {}", format_bytes(avg_packet_size));
        }
    }
}

/// 运行接收器
pub async fn run_receiver(
    output_path: impl AsRef<Path>,
    dataset_name: String,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
    max_packets: Option<usize>,
) -> Result<()> {
    let output_path = output_path.as_ref();

    // 验证输入参数
    ensure_output_directory(output_path)?;
    validate_port(port)?;
    let bind_ip = validate_network_config(&address, &network_type)?;

    info!("初始化接收器...");

    // 创建UDP接收器
    let receiver_config = ReceiverConfig {
        bind_address: bind_ip,
        bind_port: port,
        network_type: network_type.clone(),
        interface,
    };

    let socket = create_udp_receiver(receiver_config)
        .await
        .context("创建UDP接收器失败")?;

    // 创建pcap写入器
    let mut writer_config = WriterConfig::default();
    writer_config.common.enable_index_cache = true; // 启用索引
    writer_config.max_packets_per_file = 10000; // 每10000包一个文件

    let mut writer = PcapWriter::new_with_config(output_path, &dataset_name, writer_config)
        .context("创建pcap写入器失败")?;

    let mut stats = ReceiverStats::new();
    let mut buffer = vec![0u8; 65536]; // 64KB缓冲区

    info!("开始接收数据包...");
    info!("  监听地址: {}:{}", address, port);
    info!("  网络类型: {}", network_type);
    info!("  输出路径: {}", output_path.display());
    info!("  数据集名称: {}", dataset_name);

    if let Some(max) = max_packets {
        info!("  最大包数: {}", max);
    } else {
        info!("  最大包数: 无限制");
    }

    info!("按 Ctrl+C 停止接收");

    loop {
        tokio::select! {
            // 处理接收的数据包
            recv_result = socket.recv_from(&mut buffer) => {
                match recv_result {
                    Ok((bytes_received, source_addr)) => {
                        debug!(
                            "接收数据包: {} 字节, 来源: {}",
                            bytes_received,
                            source_addr
                        );

                        // 创建数据包
                        let packet_data = buffer[..bytes_received].to_vec();
                        let capture_time = Utc::now();

                        match DataPacket::from_datetime(capture_time, packet_data) {
                            Ok(packet) => {
                                // 写入数据包
                                if let Err(e) = writer.write_packet(&packet) {
                                    error!("写入数据包失败: {}", e);
                                    stats.add_error();
                                } else {
                                    stats.update(bytes_received);

                                    // 每1000个包输出一次进度
                                    if stats.packets_received % 1000 == 0 {
                                        stats.print_progress();
                                    }

                                    // 检查是否达到最大包数
                                    if let Some(max) = max_packets {
                                        if stats.packets_received >= max {
                                            info!("已达到最大包数限制: {}", max);
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("创建数据包失败: {}", e);
                                stats.add_error();
                            }
                        }
                    }
                    Err(e) => {
                        error!("接收数据包失败: {}", e);
                        stats.add_error();
                    }
                }
            }

            // 处理Ctrl+C信号
            _ = signal::ctrl_c() => {
                info!("收到停止信号，正在结束接收...");
                break;
            }
        }
    }

    // 完成写入
    info!("正在完成数据集写入...");
    writer.finalize().context("完成pcap写入失败")?;

    // 输出最终统计
    stats.print_summary();

    // 获取数据集信息
    let dataset_info = writer.get_dataset_info();
    info!("保存的数据集信息:");
    info!("  文件数量: {}", dataset_info.file_count);
    info!("  数据包总数: {}", dataset_info.total_packets);
    info!("  数据集大小: {}", format_bytes(dataset_info.total_size));

    Ok(())
}
