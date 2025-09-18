//! 配置验证模块
//!
//! 负责验证发送器和接收器的配置参数

use crate::app::config::types::DataFormat;
use crate::app::error::types::{AppError, Result};
use crate::ui::config::{ReceiverConfig, SenderConfig};

/// 配置验证器
pub struct ConfigValidator;

impl ConfigValidator {
    /// 验证发送器配置
    pub fn validate_sender_config(
        config: &SenderConfig,
    ) -> Result<()> {
        // 根据数据格式验证对应的路径
        match config.data_format {
            DataFormat::Pcap => {
                if config.pcap_path.is_empty() {
                    return Err(AppError::validation(
                        "PCAP Path",
                        "PCAP path cannot be empty",
                    ));
                }

                let pcap_path = std::path::PathBuf::from(
                    &config.pcap_path,
                );
                if !pcap_path.exists() {
                    return Err(AppError::validation(
                        "PCAP Path",
                        "PCAP path does not exist",
                    ));
                }

                if !pcap_path.is_dir() {
                    return Err(AppError::validation(
                        "PCAP Path",
                        "PCAP path must be a directory",
                    ));
                }
            }
            DataFormat::Csv => {
                if config.csv_file.is_empty() {
                    return Err(AppError::validation(
                        "CSV File",
                        "CSV file path cannot be empty",
                    ));
                }

                let csv_path = std::path::PathBuf::from(
                    &config.csv_file,
                );
                if !csv_path.exists() {
                    return Err(AppError::validation(
                        "CSV File",
                        "CSV file does not exist",
                    ));
                }

                if !csv_path.is_file() {
                    return Err(AppError::validation(
                        "CSV File",
                        "CSV path must be a file",
                    ));
                }

                // 检查文件扩展名
                if let Some(extension) =
                    csv_path.extension()
                {
                    if extension != "csv" {
                        return Err(AppError::validation(
                            "CSV File",
                            "File must have .csv extension",
                        ));
                    }
                } else {
                    return Err(AppError::validation(
                        "CSV File",
                        "File must have .csv extension",
                    ));
                }
            }
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
