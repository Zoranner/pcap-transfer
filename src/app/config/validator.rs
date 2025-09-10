//! 配置验证模块
//!
//! 负责验证发送器和接收器的配置参数

use crate::app::error::types::{AppError, Result};
use crate::ui::config::{ReceiverConfig, SenderConfig};

/// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    /// 验证发送器配置
    pub fn validate_sender_config(
        config: &SenderConfig,
    ) -> Result<()> {
        if config.dataset_path.is_empty() {
            return Err(AppError::validation(
                "Dataset Path",
                "Path cannot be empty",
            ));
        }

        let dataset_path =
            std::path::PathBuf::from(&config.dataset_path);
        if !dataset_path.exists() {
            return Err(AppError::validation(
                "Dataset Path",
                "Path does not exist",
            ));
        }

        if config.address.is_empty() {
            return Err(AppError::validation(
                "Target Address",
                "Address cannot be empty",
            ));
        }

        Ok(())
    }

    /// 验证接收器配置
    pub fn validate_receiver_config(
        config: &ReceiverConfig,
    ) -> Result<()> {
        if config.output_path.is_empty() {
            return Err(AppError::validation(
                "Output Path",
                "Path cannot be empty",
            ));
        }

        if config.dataset_name.is_empty() {
            return Err(AppError::validation(
                "Dataset Name",
                "Name cannot be empty",
            ));
        }

        if config.address.is_empty() {
            return Err(AppError::validation(
                "Listen Address",
                "Address cannot be empty",
            ));
        }

        Ok(())
    }
}
