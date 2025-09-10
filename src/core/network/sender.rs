//! 发送器模块 - 处理数据包发送逻辑

use crate::app::config::types::{
    NetworkType, SenderAppConfig,
};
use crate::app::error::types::Result;
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
    )?;

    // 验证配置
    config.validate()?;

    // 创建UDP发送器
    let socket =
        UdpSocketFactory::create_sender(&config.network)
            .await?;

    // 创建pcap读取器
    let dataset_path = &config.dataset_path;
    let dataset_name = dataset_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset");

    let mut reader = PcapReader::new_with_config(
        dataset_path.parent().unwrap_or(dataset_path),
        dataset_name,
        ReaderConfig::default(),
    )?;

    // 获取数据集信息
    let _dataset_info = reader.get_dataset_info()?;

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

    // 批量统计信息
    let mut packets_sent = 0usize;
    let mut errors_count = 0usize;
    let mut batch_bytes = Vec::new();
    let mut batch_times = Vec::new();
    let batch_size = 100; // 每100个包更新一次统计

    // 读取并发送数据包
    while let Some(packet) = reader.read_packet()? {
        // 每批次检查一次停止状态以减少锁竞争
        if packets_sent % batch_size == 0 {
            if let Ok(state) = transfer_state.lock() {
                if matches!(*state, TransferState::Idle) {
                    break;
                }
            }
        }

        let packet_data = &packet.data;
        let packet_time = packet.capture_time();

        // 时序控制（精确重放）
        timing_controller
            .wait_for_packet_time(packet_time)
            .await;

        // 发送数据包（移除debug日志以提升性能）
        match socket
            .send_to(packet_data, &target_addr)
            .await
        {
            Ok(bytes_sent) => {
                packets_sent += 1;
                batch_bytes.push(bytes_sent);
                batch_times.push(packet_time);

                // 批量更新统计信息以减少锁竞争
                if packets_sent % batch_size == 0 {
                    if let Ok(mut stats_guard) =
                        stats.lock()
                    {
                        for (bytes, time) in batch_bytes
                            .iter()
                            .zip(batch_times.iter())
                        {
                            stats_guard
                                .update_with_timestamp(
                                    *bytes, *time,
                                );
                        }
                    }
                    batch_bytes.clear();
                    batch_times.clear();
                }
            }
            Err(e) => {
                error!("发送数据包失败: {}", e);
                errors_count += 1;

                // 批量更新错误统计
                if errors_count % 10 == 0 {
                    if let Ok(mut stats_guard) =
                        stats.lock()
                    {
                        for _ in 0..10 {
                            stats_guard.add_error();
                        }
                    }
                    errors_count = 0;
                }
            }
        }
    }

    // 处理剩余的统计信息
    if !batch_bytes.is_empty() || errors_count > 0 {
        if let Ok(mut stats_guard) = stats.lock() {
            // 更新剩余的包统计
            for (bytes, time) in
                batch_bytes.iter().zip(batch_times.iter())
            {
                stats_guard
                    .update_with_timestamp(*bytes, *time);
            }
            // 更新剩余的错误统计
            for _ in 0..errors_count {
                stats_guard.add_error();
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
