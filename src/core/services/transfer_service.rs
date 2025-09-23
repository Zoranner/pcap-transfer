//! 传输服务模块
//!
//! 负责管理消息发送器的启动、停止和状态管理

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing;

use crate::app::config::manager::ConfigManager;
use crate::app::config::types::SenderAppConfig;
use crate::app::error::types::Result;
use crate::core::network::TransferState;
use crate::core::packet::PacketParser;
use crate::core::stats::collector::TransferStats;
use crate::core::stats::message_stats::MessageStatsManager;
use crate::ui::config::SenderConfig;

/// 传输服务
pub struct TransferService {
    /// 配置管理器
    pub config_manager: ConfigManager,
    /// 消息统计管理器
    pub message_stats: Arc<Mutex<MessageStatsManager>>,
}

impl TransferService {
    /// 创建新的传输服务实例
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            config_manager,
            message_stats: Arc::new(Mutex::new(
                MessageStatsManager::new(),
            )),
        }
    }

    /// 获取消息统计管理器
    pub fn get_message_stats(
        &self,
    ) -> Arc<Mutex<MessageStatsManager>> {
        Arc::clone(&self.message_stats)
    }

    /// 启动发送器
    pub fn start_sender(
        &mut self,
        config: &SenderConfig,
        stats: Arc<Mutex<TransferStats>>,
        runtime_handle: &tokio::runtime::Handle,
    ) -> Result<Arc<Mutex<TransferState>>> {
        // 开始新的统计
        if let Ok(mut message_stats) =
            self.message_stats.lock()
        {
            message_stats.reset();
        }
        // 创建共享状态
        let transfer_state =
            Arc::new(Mutex::new(TransferState::Idle));
        let shared_state = Arc::clone(&transfer_state);

        // 更新配置管理器
        self.config_manager.update_sender_config(config);

        // 获取消息配置
        let messages =
            self.config_manager.get_messages().clone();
        let enabled_messages: Vec<_> = messages
            .into_iter()
            .filter(|msg| msg.enabled)
            .collect();

        if enabled_messages.is_empty() {
            tracing::warn!("No enabled messages found, sender will not start");
            return Ok(shared_state);
        }

        // 在异步运行时中启动发送器
        let config_clone = config.clone();
        let _stats_clone = Arc::clone(&stats);
        let state_clone = Arc::clone(&transfer_state);
        let stats_manager_clone =
            Arc::clone(&self.message_stats);

        runtime_handle.spawn(async move {
            // 更新状态为运行中
            if let Ok(mut state) = state_clone.lock() {
                *state = TransferState::Running;
            }

            // 创建发送器配置
            match SenderAppConfig::new(
                config_clone.address.clone(),
                config_clone.port,
                config_clone.network_type,
                config_clone.interface,
            ) {
                Ok(sender_config) => {
                    // 实际的消息发送逻辑
                    if let Err(e) =
                        Self::run_message_sender(
                            sender_config,
                            enabled_messages,
                            state_clone.clone(),
                            stats_manager_clone,
                        )
                        .await
                    {
                        tracing::error!(
                            "Message sender failed: {}",
                            e
                        );
                        if let Ok(mut state) =
                            state_clone.lock()
                        {
                            *state = TransferState::Error(
                                e.to_string(),
                            );
                        }
                    }
                }
                Err(e) => {
                    // 在错误情况下更新状态
                    if let Ok(mut state) =
                        state_clone.lock()
                    {
                        *state = TransferState::Error(
                            e.to_string(),
                        );
                    }
                    tracing::error!(
                        "Sender config failed: {}",
                        e
                    );
                }
            }
        });

        tracing::info!("Sender started successfully");
        Ok(shared_state)
    }

    /// 运行消息发送器
    async fn run_message_sender(
        sender_config: SenderAppConfig,
        messages: Vec<crate::app::config::message_types::MessageDefinition>,
        state: Arc<Mutex<TransferState>>,
        stats_manager: Arc<Mutex<MessageStatsManager>>,
    ) -> Result<()> {
        use crate::app::config::message_types::MessageRuntimeState;

        // 创建UDP套接字
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| {
                crate::app::error::types::AppError::network(
                    format!(
                        "Failed to bind UDP socket: {}",
                        e
                    ),
                )
            })?;

        let target_addr = format!(
            "{}:{}",
            sender_config.network.address,
            sender_config.network.port
        );

        tracing::info!(
            "Sending messages to {}",
            target_addr
        );

        // 将消息定义转换为运行时状态
        let mut runtime_messages: Vec<MessageRuntimeState> =
            messages
                .into_iter()
                .map(MessageRuntimeState::from_definition)
                .collect();

        let mut packet_counters: Vec<u64> =
            vec![0; runtime_messages.len()];
        let mut last_send_times: Vec<
            Option<std::time::Instant>,
        > = vec![None; runtime_messages.len()];

        // 主发送循环
        loop {
            // 检查是否应该停止
            if let Ok(state_guard) = state.lock() {
                if matches!(
                    *state_guard,
                    TransferState::Idle
                        | TransferState::Error(_)
                ) {
                    break;
                }
            }

            let now = std::time::Instant::now();

            // 检查是否所有设置了包数量限制的消息都已完成（自动停止条件）
            let messages_with_limit: Vec<_> = runtime_messages
                .iter()
                .enumerate()
                .filter(|(_, runtime_msg)| runtime_msg.definition.packet_count > 0)
                .collect();
            
            let all_limited_messages_complete = if messages_with_limit.is_empty() {
                false // 如果没有消息设置了限制，则不自动停止
            } else {
                messages_with_limit.iter().all(|(i, runtime_msg)| {
                    packet_counters[*i] >= runtime_msg.definition.packet_count
                })
            };

            // 如果所有设置了包数量限制的消息都完成了，自动停止
            if all_limited_messages_complete {
                tracing::info!("All messages with packet count limits have completed, stopping automatically");
                break;
            }

            // 检查每个消息是否需要发送
            for (i, runtime_msg) in
                runtime_messages.iter_mut().enumerate()
            {
                let msg_def = &runtime_msg.definition;

                // 检查是否到了发送时间
                let should_send = match last_send_times[i] {
                    None => true, // 第一次发送
                    Some(last_time) => {
                        let elapsed =
                            now.duration_since(last_time);
                        elapsed
                            >= Duration::from_millis(
                                msg_def.interval,
                            )
                    }
                };

                if should_send {
                    // 检查包数量限制
                    if msg_def.packet_count > 0
                        && packet_counters[i]
                            >= msg_def.packet_count
                    {
                        continue; // 已达到包数量限制
                    }

                    // 生成数据包
                    let total_packets =
                        if msg_def.packet_count == 0 {
                            None
                        } else {
                            Some(msg_def.packet_count)
                        };
                    match runtime_msg.generate_packet(
                        packet_counters[i] as usize,
                        total_packets,
                    ) {
                        Ok(packet_data) => {
                            // 发送数据包
                            match socket
                                .send_to(
                                    &packet_data,
                                    &target_addr,
                                )
                                .await
                            {
                                Ok(_) => {
                                    packet_counters[i] += 1;
                                    last_send_times[i] =
                                        Some(now);

                                    // 更新统计信息
                                    if let Ok(mut stats) =
                                        stats_manager.lock()
                                    {
                                        stats.record_message_sent(
                                            &msg_def.name,
                                            packet_data.len(),
                                            chrono::Utc::now(),
                                        );
                                    }

                                    tracing::debug!(
                                        "Sent message '{}' #{} ({} bytes) to {}",
                                        msg_def.name,
                                        packet_counters[i],
                                        packet_data.len(),
                                        target_addr
                                    );

                                    // 解析并打印包内容
                                    let parsed_content = PacketParser::parse_and_format_packet(
                                        runtime_msg,
                                        &packet_data,
                                        packet_counters[i]
                                    );
                                    tracing::debug!("{}", parsed_content);
                                }
                                Err(e) => {
                                    // 记录发送错误
                                    if let Ok(mut stats) =
                                        stats_manager.lock()
                                    {
                                        stats.record_message_error(&msg_def.name);
                                    }
                                    tracing::error!("Failed to send message '{}': {}", msg_def.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            // 记录数据包生成错误
                            if let Ok(mut stats) =
                                stats_manager.lock()
                            {
                                stats.record_message_error(
                                    &msg_def.name,
                                );
                            }
                            tracing::error!("Failed to generate packet for message '{}': {}", msg_def.name, e);
                        }
                    }
                }
            }

            // 短暂休眠避免CPU占用过高
            tokio::time::sleep(Duration::from_millis(10))
                .await;
        }

        // 发送完成后更新状态为空闲
        if let Ok(mut state_guard) = state.lock() {
            if matches!(
                *state_guard,
                TransferState::Running
            ) {
                *state_guard = TransferState::Idle;
            }
        }

        // 结束所有消息的统计
        if let Ok(mut stats) = stats_manager.lock() {
            stats.finish_all();
        }

        tracing::info!("Message sender stopped");
        Ok(())
    }

    /// 停止传输（通用方法）
    pub fn stop_transfer(
        shared_state: &Option<Arc<Mutex<TransferState>>>,
    ) {
        if let Some(state) = shared_state {
            if let Ok(mut transfer_state) = state.lock() {
                *transfer_state = TransferState::Idle;
            }
        }
    }
}
