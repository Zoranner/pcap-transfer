//! 接收器模块 - 处理数据包接收逻辑

use crate::app::config::types::{
    NetworkType, ReceiverAppConfig,
};
use crate::app::error::types::Result;
use crate::core::network::sender::TransferState;
use crate::core::network::types::UdpSocketFactory;
use crate::core::stats::collector::TransferStats;
use chrono::Utc;
use pcapfile_io::{DataPacket, PcapWriter, WriterConfig};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::error;

/// GUI专用的接收器函数，支持共享状态和统计信息
#[allow(clippy::too_many_arguments)]
pub async fn run_receiver_with_gui_stats(
    output_path: PathBuf,
    dataset_name: String,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
    stats: Arc<Mutex<TransferStats>>,
    transfer_state: Arc<Mutex<TransferState>>,
) -> Result<()> {
    // 创建配置
    let config = ReceiverAppConfig::new(
        output_path.clone(),
        dataset_name.clone(),
        address.clone(),
        port,
        network_type,
        interface,
    )?;

    // 验证配置
    config.validate()?;

    // 创建UDP接收器
    let socket =
        UdpSocketFactory::create_receiver(&config.network)
            .await?;

    // 创建pcap写入器
    let writer_config = WriterConfig {
        index_cache_size: 1000, // 设置索引缓存大小
        max_packets_per_file: 1000,
        ..Default::default()
    };

    let mut writer = PcapWriter::new_with_config(
        &config.output_path,
        &config.dataset_name,
        writer_config,
    )?;

    // 获取配置中的缓冲区大小
    let buffer_size = config.buffer_size;

    // 重置并初始化统计信息
    if let Ok(mut stats_guard) = stats.lock() {
        *stats_guard = TransferStats::new(); // GUI不需要进度条
    }

    let mut buffer = vec![0u8; buffer_size];

    // 统计信息
    // 接收循环 - 使用 tokio::select! 来同时监听数据包接收和停止信号
    loop {
        tokio::select! {
            // 接收数据包分支
            recv_result = socket.recv_from(&mut buffer) => {
                match recv_result {
            Ok((bytes_received, _source_addr)) => {
                // 创建数据包
                let packet_data = buffer[..bytes_received].to_vec();
                let capture_time = Utc::now();

                match DataPacket::from_datetime(capture_time, packet_data) {
                    Ok(packet) => {
                        // 写入数据包
                        if let Err(e) = writer.write_packet(&packet) {
                            error!("Failed to write packet: {}", e);

                            // 立即更新错误统计
                            if let Ok(mut stats_guard) = stats.lock() {
                                stats_guard.add_error();
                            }
                        } else {
                            // 立即更新统计信息
                            if let Ok(mut stats_guard) = stats.lock() {
                                stats_guard.update_with_timestamp(bytes_received, capture_time);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create packet: {}", e);

                        // 立即更新错误统计
                        if let Ok(mut stats_guard) = stats.lock() {
                            stats_guard.add_error();
                        }
                    }
                }
                    }
                    Err(e) => {
                        error!("Failed to receive packet: {}", e);

                        // 立即更新错误统计
                        if let Ok(mut stats_guard) = stats.lock() {
                            stats_guard.add_error();
                        }
                    }
                }
            },
            // 定期检查停止状态分支 - 每100ms检查一次
            _ = sleep(Duration::from_millis(100)) => {
                if let Ok(state) = transfer_state.lock() {
                    if matches!(*state, TransferState::Idle) {
                        break;
                    }
                }
            }
        }
    }

    // 完成写入
    writer.finalize()?;

    // 完成统计信息
    if let Ok(mut stats_guard) = stats.lock() {
        stats_guard.finish();
    }

    // 更新传输状态为完成
    if let Ok(mut state) = transfer_state.lock() {
        *state = TransferState::Completed;
    }

    Ok(())
}
