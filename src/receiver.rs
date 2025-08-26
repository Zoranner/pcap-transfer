use chrono::Utc;
use pcapfile_io::{DataPacket, PcapWriter, WriterConfig};
use std::path::Path;
use tokio::signal;
use tracing::{debug, error};

use crate::cli::NetworkType;
use crate::config::{
    AppConfig, DisplayConfig, OperationConfig,
};
use crate::display::Display;
use crate::error::Result;
use crate::network::UdpSocketFactory;
use crate::stats::TransferStats;

// 统计逻辑已移至 stats.rs 模块

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
    let output_path = output_path.as_ref().to_path_buf();

    // 创建配置
    let config = AppConfig::for_receiver(
        output_path.clone(),
        dataset_name.clone(),
        address.clone(),
        port,
        network_type.clone(),
        interface,
        max_packets,
    )?;

    // 验证配置
    config.validate()?;

    // 创建显示器
    let display = Display::new(DisplayConfig::default());
    display.print_welcome();
    display.print_info("初始化接收器...");

    // 创建UDP接收器
    let socket =
        UdpSocketFactory::create_receiver(&config.network)
            .await?;

    // 创建pcap写入器
    let mut writer_config = WriterConfig::default();
    writer_config.common.enable_index_cache = true; // 启用索引
    writer_config.max_packets_per_file = 10000; // 每10000包一个文件

    let mut writer = PcapWriter::new_with_config(
        &output_path,
        &dataset_name,
        writer_config,
    )?;

    // 获取配置中的缓冲区大小
    let (buffer_size, max_packets_limit) =
        if let OperationConfig::Receive {
            buffer_size,
            max_packets,
            ..
        } = &config.operation
        {
            (*buffer_size, *max_packets)
        } else {
            (65536, None) // 默认64KB缓冲区
        };

    // 创建进度条
    let progress_bar = if let Some(max) = max_packets_limit
    {
        display.create_progress_bar(max as u64)
    } else {
        None // 无限制模式不显示进度条
    };

    let mut stats = TransferStats::new(progress_bar);
    let mut buffer = vec![0u8; buffer_size];

    // 显示网络配置
    display.print_network_config(
        "接收",
        &address,
        port,
        config.network.get_type_description(),
    );

    display.print_info("开始接收数据包");
    display.print_info("按 Ctrl+C 停止接收");

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
                                    stats.update_with_timestamp(bytes_received, capture_time);

                                    // 更新进度显示
                                    stats.update_progress_receiver(&display);

                                    // 检查是否达到最大包数
                                    if let Some(max) = max_packets_limit {
                                        if stats.packets_count() >= max {
                                            display.print_info(&format!("已达到最大包数限制: {max}"));
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
                display.print_info("收到停止信号，正在结束接收...");
                break;
            }
        }
    }

    // 完成写入
    display.print_info("正在完成数据集写入...");
    writer.finalize()?;

    // 输出最终统计
    println!(); // 接收结束和统计信息之间的空行
    stats.print_summary(&display, "接收统计信息");

    // 获取数据集信息
    let dataset_info = writer.get_dataset_info();
    display.print_dataset_info(
        dataset_info.file_count,
        dataset_info.total_packets as usize,
        dataset_info.total_size,
        None,
    );

    display.print_success("接收操作已成功完成");

    Ok(())
}
