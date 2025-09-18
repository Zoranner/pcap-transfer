//! 传输服务模块
//!
//! 负责管理发送器和接收器的启动、停止和状态管理

use std::sync::{Arc, Mutex};
use tracing;

use crate::app::config::manager::ConfigManager;
use crate::app::error::types::{AppError, Result};
use crate::core::network::receiver::run_receiver_with_gui_stats;
use crate::core::network::sender::{
    run_sender_with_gui_stats, TransferState,
};
use crate::core::stats::collector::TransferStats;
use crate::ui::config::{ReceiverConfig, SenderConfig};

/// 传输服务
pub struct TransferService {
    pub config_manager: ConfigManager,
}

impl TransferService {
    /// 创建新的传输服务实例
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }

    /// 启动发送器
    pub fn start_sender(
        &mut self,
        config: &SenderConfig,
        stats: Arc<Mutex<TransferStats>>,
        runtime_handle: &tokio::runtime::Handle,
    ) -> Result<Arc<Mutex<TransferState>>> {
        // 保存发送器配置（统一接口）
        self.config_manager.update_sender_config(config);
        if let Err(e) = self.config_manager.save() {
            tracing::warn!("Failed to save config: {}", e);
        }

        // 根据数据格式选择对应的路径
        let dataset_path = match config.data_format {
            crate::app::config::types::DataFormat::Pcap => {
                std::path::PathBuf::from(&config.pcap_path)
            }
            crate::app::config::types::DataFormat::Csv => {
                std::path::PathBuf::from(&config.csv_file)
            }
        };
        let address = config.address.clone();
        let port = config.port;
        let network_type = config.network_type;
        let interface = config.interface.clone();
        let data_format = config.data_format;
        let csv_packet_interval =
            config.csv_packet_interval;

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        } else {
            tracing::error!(
                "Unable to acquire statistics lock"
            );
            return Err(AppError::validation(
                "Statistics",
                "Statistics initialization failed",
            ));
        }

        // 创建共享状态
        let transfer_state_ref =
            Arc::new(Mutex::new(TransferState::Running));
        let transfer_state_clone =
            Arc::clone(&transfer_state_ref);
        let transfer_state_for_error =
            Arc::clone(&transfer_state_ref);

        // 在后台运行发送任务
        runtime_handle.spawn(async move {
            match run_sender_with_gui_stats(
                dataset_path,
                address,
                port,
                network_type,
                interface,
                data_format,
                csv_packet_interval,
                stats,
                transfer_state_clone,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!("Send task completed");
                }
                Err(e) => {
                    tracing::error!(
                        "Send task failed: {}",
                        e
                    );
                    if let Ok(mut state) =
                        transfer_state_for_error.lock()
                    {
                        *state = TransferState::Error(
                            e.to_string(),
                        );
                    }
                }
            }
        });

        Ok(transfer_state_ref)
    }

    /// 启动接收器
    pub fn start_receiver(
        &mut self,
        config: &ReceiverConfig,
        stats: Arc<Mutex<TransferStats>>,
        runtime_handle: &tokio::runtime::Handle,
    ) -> Result<Arc<Mutex<TransferState>>> {
        // 保存当前配置到配置管理器
        self.config_manager.update_receiver_config(config);
        if let Err(e) = self.config_manager.save() {
            tracing::warn!("Failed to save config: {}", e);
        }

        let output_path =
            std::path::PathBuf::from(&config.output_path);
        let dataset_name = config.dataset_name.clone();
        let address = config.address.clone();
        let port = config.port;
        let network_type = config.network_type;
        let interface = config.interface.clone();

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        }

        // 创建共享状态
        let shared_state =
            Arc::new(Mutex::new(TransferState::Running));
        let transfer_state_clone =
            Arc::clone(&shared_state);
        let transfer_state_for_error =
            Arc::clone(&shared_state);

        // 在后台运行接收任务
        runtime_handle.spawn(async move {
            match run_receiver_with_gui_stats(
                output_path,
                dataset_name,
                address,
                port,
                network_type,
                interface,
                stats,
                transfer_state_clone,
            )
            .await
            {
                Ok(_) => {
                    tracing::info!(
                        "Receive task completed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Receive task failed: {}",
                        e
                    );
                    if let Ok(mut state) =
                        transfer_state_for_error.lock()
                    {
                        *state = TransferState::Error(
                            e.to_string(),
                        );
                    }
                }
            }
        });

        Ok(shared_state)
    }

    /// 停止传输
    pub fn stop_transfer(
        shared_state: &Option<Arc<Mutex<TransferState>>>,
    ) {
        if let Some(shared_state) = shared_state {
            if let Ok(mut state) = shared_state.lock() {
                *state = TransferState::Idle;
            }
        }
    }
}
