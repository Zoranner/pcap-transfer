use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use pcapfile_io::{PcapReader, ReaderConfig};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::{sleep_until, Instant as TokioInstant};
use tracing::{debug, error, info, warn};

use crate::cli::NetworkType;
use crate::network::{create_udp_sender, validate_network_config, SenderConfig};
use crate::utils::{format_bytes, format_rate, validate_dataset_path, validate_port};

/// 发送器统计信息
#[derive(Debug, Default)]
struct SenderStats {
    packets_sent: usize,
    bytes_sent: u64,
    errors: usize,
    start_time: Option<Instant>,
}

impl SenderStats {
    fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    fn update(&mut self, bytes: usize) {
        self.packets_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    fn add_error(&mut self) {
        self.errors += 1;
    }

    fn get_duration(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }

    fn get_rate_bps(&self) -> f64 {
        let duration_secs = self.get_duration().as_secs_f64();
        if duration_secs > 0.0 {
            (self.bytes_sent as f64 * 8.0) / duration_secs
        } else {
            0.0
        }
    }

    fn print_summary(&self) {
        let duration = self.get_duration();
        let rate_bps = self.get_rate_bps();

        info!("发送完成统计:");
        info!("  发送包数: {}", self.packets_sent);
        info!("  发送字节: {}", format_bytes(self.bytes_sent));
        info!("  错误数量: {}", self.errors);
        info!("  用时: {:.2} 秒", duration.as_secs_f64());
        info!("  平均速率: {}", format_rate(rate_bps));

        if self.packets_sent > 0 {
            let avg_packet_size = self.bytes_sent / self.packets_sent as u64;
            info!("  平均包大小: {}", format_bytes(avg_packet_size));
        }
    }
}

/// 时序控制器（基于数据包时间戳）
struct TimingController {
    first_packet_time: Option<DateTime<Utc>>,
    real_start_time: Option<TokioInstant>,
}

impl TimingController {
    fn new() -> Self {
        Self {
            first_packet_time: None,
            real_start_time: None,
        }
    }

    async fn wait_for_packet_time(&mut self, packet_time: DateTime<Utc>) {
        if self.first_packet_time.is_none() {
            // 第一个数据包，记录基准时间
            self.first_packet_time = Some(packet_time);
            self.real_start_time = Some(TokioInstant::now());
            return;
        }

        let first_time = self.first_packet_time.unwrap();
        let real_start = self.real_start_time.unwrap();

        // 计算数据包相对于第一个包的时间差
        let packet_offset = packet_time
            .signed_duration_since(first_time)
            .to_std()
            .unwrap_or_default();

        // 计算应该发送的实际时间
        let target_time = real_start + packet_offset;
        let now = TokioInstant::now();

        if target_time > now {
            sleep_until(target_time).await;
        } else if now.duration_since(real_start) > packet_offset + Duration::from_millis(100) {
            // 如果延迟超过100ms，给出警告
            warn!(
                "数据包发送延迟: 预期={:.3}s, 实际={:.3}s",
                packet_offset.as_secs_f64(),
                now.duration_since(real_start).as_secs_f64()
            );
        }
    }
}

/// 运行发送器
pub async fn run_sender(
    dataset_path: impl AsRef<Path>,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
) -> Result<()> {
    let dataset_path = dataset_path.as_ref();

    // 验证输入参数
    validate_dataset_path(dataset_path)?;
    validate_port(port)?;
    let target_ip = validate_network_config(&address, &network_type)?;

    info!("初始化发送器...");

    // 创建UDP发送器
    let sender_config = SenderConfig {
        target_address: target_ip,
        target_port: port,
        network_type: network_type.clone(),
        interface,
    };

    let socket = create_udp_sender(sender_config)
        .await
        .context("创建UDP发送器失败")?;

    // 创建pcap读取器
    let dataset_name = dataset_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset");

    let mut reader = PcapReader::new_with_config(
        dataset_path.parent().unwrap_or(dataset_path),
        dataset_name,
        ReaderConfig::default(),
    )
    .context("创建pcap读取器失败")?;

    // 获取数据集信息
    let dataset_info = reader.get_dataset_info()?;
    info!("数据集信息:");
    info!("  文件数量: {}", dataset_info.file_count);
    info!("  数据包总数: {}", dataset_info.total_packets);
    info!("  数据集大小: {}", format_bytes(dataset_info.total_size));

    if let (Some(start_time), Some(end_time)) =
        (dataset_info.start_timestamp, dataset_info.end_timestamp)
    {
        let duration_ns = end_time - start_time;
        let duration_secs = duration_ns as f64 / 1_000_000_000.0;
        info!("  时间跨度: {:.3} 秒", duration_secs);
    }

    // 初始化控制器
    let mut timing_controller = TimingController::new();
    let mut stats = SenderStats::new();

    info!("开始发送数据包 (按原始时序)...");

    // 读取并发送数据包
    while let Some(packet) = reader.read_packet()? {
        let packet_data = &packet.data;
        let packet_time = packet.capture_time();

        // 时序控制（基于数据包时间戳）
        timing_controller.wait_for_packet_time(packet_time).await;

        // 发送数据包
        match socket
            .send_to(packet_data, format!("{address}:{port}"))
            .await
        {
            Ok(bytes_sent) => {
                debug!(
                    "发送数据包: {} 字节, 时间戳: {}",
                    bytes_sent,
                    packet_time.format("%H:%M:%S%.9f")
                );
                stats.update(bytes_sent);
            }
            Err(e) => {
                error!("发送数据包失败: {}", e);
                stats.add_error();
            }
        }

        // 每1000个包输出一次进度
        if stats.packets_sent % 1000 == 0 {
            let rate_bps = stats.get_rate_bps();
            info!(
                "已发送 {} 包, 当前速率: {}",
                stats.packets_sent,
                format_rate(rate_bps)
            );
        }
    }

    // 输出最终统计
    stats.print_summary();

    Ok(())
}
