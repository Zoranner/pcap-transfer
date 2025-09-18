//! 发送器模块 - 处理数据包发送逻辑

use crate::app::config::types::{
    DataFormat, NetworkType, SenderAppConfig,
};
use crate::app::error::types::Result;
use crate::core::csv::CsvParser;
use crate::core::network::types::UdpSocketFactory;
use crate::core::stats::collector::TransferStats;
use crate::core::timing::utils::TimingController;
use pcapfile_io::{PcapReader, ReaderConfig};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::error;

/// 传输状态枚举
#[derive(Debug, Clone)]
pub enum TransferState {
    Idle,
    Running,
    Completed,
    Error(String),
}

/// GUI专用的发送器函数，支持共享状态和统计信息
#[allow(clippy::too_many_arguments)]
pub async fn run_sender_with_gui_stats(
    dataset_path: PathBuf,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
    data_format: DataFormat,
    csv_packet_interval: u64, // CSV发送周期（毫秒）
    stats: Arc<Mutex<TransferStats>>,
    transfer_state: Arc<Mutex<TransferState>>,
) -> Result<()> {
    // 创建配置
    let config = SenderAppConfig::new(
        dataset_path.clone(),
        address.clone(),
        port,
        network_type,
        interface,
        data_format,
    )?;

    // 验证配置
    config.validate()?;

    // 创建UDP发送器
    let socket =
        UdpSocketFactory::create_sender(&config.network)
            .await?;

    // 初始化时序控制器（始终启用精确时序控制）
    let mut timing_controller = TimingController::new();

    // 重置并初始化统计信息
    if let Ok(mut stats_guard) = stats.lock() {
        *stats_guard = TransferStats::new(); // GUI不需要进度条
    }

    // 预计算目标地址以避免重复字符串格式化
    let target_addr = format!(
        "{}:{}",
        config.network.address, config.network.port
    );

    // 基于时间的停止状态检查
    let mut last_stop_check = std::time::Instant::now();
    let stop_check_interval =
        std::time::Duration::from_millis(100);

    // 根据数据格式选择不同的处理方式
    match config.data_format {
        DataFormat::Pcap => {
            // PCAP数据集处理
            let dataset_path = &config.dataset_path;
            let dataset_name = dataset_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("dataset");

            let mut reader = PcapReader::new_with_config(
                dataset_path
                    .parent()
                    .unwrap_or(dataset_path),
                dataset_name,
                ReaderConfig::default(),
            )?;

            // 获取数据集信息
            let _dataset_info =
                reader.get_dataset_info()?;

            // 读取并发送数据包
            while let Some(packet) = reader.read_packet()? {
                // 每100ms检查一次停止状态
                if last_stop_check.elapsed()
                    >= stop_check_interval
                {
                    if let Ok(state) = transfer_state.lock()
                    {
                        if matches!(
                            *state,
                            TransferState::Idle
                        ) {
                            tracing::info!("Sender received stop signal, breaking loop");
                            break;
                        }
                    }
                    last_stop_check =
                        std::time::Instant::now();
                }

                let packet_data = &packet.packet.data;
                let packet_time = packet.capture_time();

                // 时序控制（精确重放）
                timing_controller
                    .wait_for_packet_time(packet_time)
                    .await;

                // 发送数据包
                match socket
                    .send_to(packet_data, &target_addr)
                    .await
                {
                    Ok(bytes_sent) => {
                        // 立即更新统计信息
                        if let Ok(mut stats_guard) =
                            stats.lock()
                        {
                            stats_guard
                                .update_with_timestamp(
                                    bytes_sent,
                                    packet_time,
                                );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to send packet: {}",
                            e
                        );

                        // 立即更新错误统计
                        if let Ok(mut stats_guard) =
                            stats.lock()
                        {
                            stats_guard.add_error();
                        }
                    }
                }
            }
        }
        DataFormat::Csv => {
            // CSV数据处理
            let csv_parser =
                CsvParser::from_file(&config.dataset_path)?;
            let row_count = csv_parser.row_count();

            tracing::info!("CSV file loaded: {} rows, packet interval: {}ms", row_count, csv_packet_interval);

            // 发送CSV数据包
            for row_index in 0..row_count {
                // 每100ms检查一次停止状态
                if last_stop_check.elapsed()
                    >= stop_check_interval
                {
                    if let Ok(state) = transfer_state.lock()
                    {
                        if matches!(
                            *state,
                            TransferState::Idle
                        ) {
                            tracing::info!("Sender received stop signal, breaking loop");
                            break;
                        }
                    }
                    last_stop_check =
                        std::time::Instant::now();
                }

                // 生成数据包
                let csv_packet = csv_parser
                    .generate_packet(row_index)?;
                let packet_data = &csv_packet.data;
                let packet_time = csv_packet.timestamp;

                tracing::info!(
                    "Sending row {}: {} bytes",
                    row_index,
                    packet_data.len()
                );

                // 发送数据包
                match socket
                    .send_to(packet_data, &target_addr)
                    .await
                {
                    Ok(bytes_sent) => {
                        // 立即更新统计信息
                        if let Ok(mut stats_guard) =
                            stats.lock()
                        {
                            stats_guard
                                .update_with_timestamp(
                                    bytes_sent,
                                    packet_time,
                                );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to send packet: {}",
                            e
                        );

                        // 立即更新错误统计
                        if let Ok(mut stats_guard) =
                            stats.lock()
                        {
                            stats_guard.add_error();
                        }
                    }
                }

                // CSV数据发送间隔控制（除了最后一行）
                if row_index < row_count - 1 {
                    tokio::time::sleep(
                        tokio::time::Duration::from_millis(
                            csv_packet_interval,
                        ),
                    )
                    .await;
                }
            }
        }
    }

    // 标记统计信息完成并更新传输状态为完成
    if let Ok(mut stats_guard) = stats.lock() {
        stats_guard.finish();
    }
    if let Ok(mut state) = transfer_state.lock() {
        *state = TransferState::Completed;
    }

    Ok(())
}
