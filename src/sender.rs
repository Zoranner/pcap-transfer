use pcapfile_io::{PcapReader, ReaderConfig};
use std::path::Path;
use tracing::{debug, error};

use crate::cli::NetworkType;
use crate::config::{AppConfig, DisplayConfig, OperationConfig};
use crate::display::Display;
use crate::error::Result;
use crate::network::UdpSocketFactory;
use crate::stats::TransferStats;
use crate::timing::TimingController;

// 统计逻辑已移至 stats.rs 模块

// 时序控制逻辑已移至 timing.rs 模块

/// 运行发送器
pub async fn run_sender(
    dataset_path: impl AsRef<Path>,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
) -> Result<()> {
    let dataset_path = dataset_path.as_ref().to_path_buf();

    // 创建配置
    let config = AppConfig::for_sender(
        dataset_path.clone(),
        address.clone(),
        port,
        network_type.clone(),
        interface,
    )?;

    // 验证配置
    config.validate()?;

    // 创建显示器
    let display = Display::new(DisplayConfig::default());
    display.print_welcome();
    display.print_info("初始化发送器...");

    // 创建UDP发送器
    let socket = UdpSocketFactory::create_sender(&config.network).await?;

    // 创建pcap读取器
    let dataset_name = dataset_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset");

    let mut reader = PcapReader::new_with_config(
        dataset_path.parent().unwrap_or(&dataset_path),
        dataset_name,
        ReaderConfig::default(),
    )?;

    // 获取数据集信息
    let dataset_info = reader.get_dataset_info()?;

    let time_span = if let (Some(start_time), Some(end_time)) =
        (dataset_info.start_timestamp, dataset_info.end_timestamp)
    {
        let duration_ns = end_time - start_time;
        Some(duration_ns as f64 / 1_000_000_000.0)
    } else {
        None
    };

    display.print_dataset_info(
        dataset_info.file_count,
        dataset_info.total_packets as usize,
        dataset_info.total_size,
        time_span,
    );

    display.print_network_config(
        "发送",
        &address,
        port,
        config.network.get_type_description(),
    );

    // 初始化控制器和进度条
    let mut timing_controller = if let OperationConfig::Send {
        timing_enabled,
        max_delay_threshold_ms,
        ..
    } = &config.operation
    {
        if *timing_enabled {
            Some(TimingController::with_delay_threshold(
                *max_delay_threshold_ms,
            ))
        } else {
            None
        }
    } else {
        None
    };

    let progress_bar = display.create_progress_bar(dataset_info.total_packets);
    let mut stats = TransferStats::new(progress_bar);

    display.print_info("开始数据包传输 (保持原始时序)");

    // 读取并发送数据包
    while let Some(packet) = reader.read_packet()? {
        let packet_data = &packet.data;
        let packet_time = packet.capture_time();

        // 时序控制（如果启用）
        if let Some(controller) = &mut timing_controller {
            controller.wait_for_packet_time(packet_time).await;
        }

        // 发送数据包
        match socket
            .send_to(
                packet_data,
                format!("{}:{}", config.network.address, config.network.port),
            )
            .await
        {
            Ok(bytes_sent) => {
                debug!(
                    "发送数据包: {} 字节, 时间戳: {}",
                    bytes_sent,
                    packet_time.format("%H:%M:%S%.9f")
                );
                stats.update(bytes_sent);
                stats.update_progress_sender(&display);
            }
            Err(e) => {
                error!("发送数据包失败: {}", e);
                stats.add_error();
            }
        }
    }

    // 完成进度条并输出统计
    stats.finish_progress(&display);
    stats.print_summary(&display, "发送统计信息");
    display.print_success("发送操作已成功完成");

    Ok(())
}
