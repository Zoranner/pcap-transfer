//! 接收器模块 - 处理数据包接收逻辑

use crate::config::{NetworkType, ReceiverAppConfig};
use crate::error::Result;
use crate::network::UdpSocketFactory;
use crate::sender::TransferState;
use crate::stats::TransferStats;
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
    let mut writer_config = WriterConfig::default();
    writer_config.common.enable_index_cache = true;
    writer_config.max_packets_per_file = 10000;

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

    // 批量统计变量
    let batch_size = 100;
    let mut batch_bytes = Vec::with_capacity(batch_size);
    let mut batch_times = Vec::with_capacity(batch_size);
    let mut errors_count = 0;
    let error_batch_size = 10;
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
                            error!("写入数据包失败: {}", e);
                            errors_count += 1;
                            if errors_count >= error_batch_size {
                                if let Ok(mut stats_guard) = stats.lock() {
                                    for _ in 0..errors_count {
                                        stats_guard.add_error();
                                    }
                                }
                                errors_count = 0;
                            }
                        } else {
                            // 收集统计信息到批次
                            batch_bytes.push(bytes_received);
                            batch_times.push(capture_time);

                            // 批量更新统计信息
                            if batch_bytes.len() >= batch_size {
                                if let Ok(mut stats_guard) = stats.lock() {
                                    for (bytes, time) in batch_bytes.iter().zip(batch_times.iter()) {
                                        stats_guard.update_with_timestamp(*bytes, *time);
                                    }
                                }
                                batch_bytes.clear();
                                batch_times.clear();
                            }

                            // 移除了最大包数限制检查
                        }
                    }
                    Err(e) => {
                        error!("创建数据包失败: {}", e);
                        errors_count += 1;
                        if errors_count >= error_batch_size {
                            if let Ok(mut stats_guard) = stats.lock() {
                                for _ in 0..errors_count {
                                    stats_guard.add_error();
                                }
                            }
                            errors_count = 0;
                        }
                    }
                }
                    }
                    Err(e) => {
                        error!("接收数据包失败: {}", e);
                        errors_count += 1;
                        if errors_count >= error_batch_size {
                            if let Ok(mut stats_guard) = stats.lock() {
                                for _ in 0..errors_count {
                                    stats_guard.add_error();
                                }
                            }
                            errors_count = 0;
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

    // 处理剩余的批次统计
    if !batch_bytes.is_empty() {
        if let Ok(mut stats_guard) = stats.lock() {
            for (bytes, time) in
                batch_bytes.iter().zip(batch_times.iter())
            {
                stats_guard
                    .update_with_timestamp(*bytes, *time);
            }
        }
    }

    // 处理剩余的错误统计
    if errors_count > 0 {
        if let Ok(mut stats_guard) = stats.lock() {
            for _ in 0..errors_count {
                stats_guard.add_error();
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
