//! 应用程序错误类型定义
//!
//! 提供统一的错误类型系统，支持错误链和上下文信息

use std::io;
use std::net::AddrParseError;
use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum DataTransferError {
    /// 网络相关错误
    #[error("Network error: {0}")]
    Network(#[from] io::Error),

    /// 配置错误
    #[error("Configuration error: {message}")]
    Config {
        /// 错误消息
        message: String,
    },

    /// IP地址解析错误
    #[error("Invalid IP address format: {0}")]
    IpAddress(#[from] AddrParseError),

    /// 验证错误
    #[error("Validation failed: {field} - {message}")]
    Validation {
        /// 验证失败的字段名
        field: String,
        /// 错误消息
        message: String,
    },

    /// GUI 相关错误
    #[error("GUI error: {0}")]
    Gui(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    /// 序列化错误（写入）
    #[error("Serialization write error: {0}")]
    SerializationWrite(#[from] toml::ser::Error),
}

impl From<anyhow::Error> for DataTransferError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(io_error) =
            err.downcast_ref::<std::io::Error>()
        {
            return DataTransferError::Network(
                std::io::Error::new(
                    io_error.kind(),
                    err.to_string(),
                ),
            );
        }
        DataTransferError::config(err.to_string())
    }
}

impl DataTransferError {
    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(std::io::Error::other(message.into()))
    }

    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 创建验证错误
    pub fn validation(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// 结果类型别名
pub type Result<T> =
    std::result::Result<T, DataTransferError>;
/// 应用程序错误类型别名
pub type AppError = DataTransferError;
