use std::io;
use std::net::AddrParseError;
use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum DataTransferError {
    /// 网络相关错误
    #[error("网络错误: {0}")]
    Network(#[from] io::Error),

    /// 配置错误
    #[error("配置错误: {message}")]
    Config { message: String },

    /// IP地址解析错误
    #[error("IP地址格式错误: {0}")]
    IpAddress(#[from] AddrParseError),

    /// pcapfile-io 库错误
    #[error("PCAP文件处理错误: {0}")]
    PcapIo(#[from] pcapfile_io::PcapError),

    /// 验证错误
    #[error("验证失败: {field} - {message}")]
    Validation { field: String, message: String },

    /// GUI 相关错误
    #[error("GUI错误: {0}")]
    Gui(String),
}

impl From<anyhow::Error> for DataTransferError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
            return DataTransferError::Network(std::io::Error::new(
                io_error.kind(),
                err.to_string(),
            ));
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
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, DataTransferError>;
pub type AppError = DataTransferError;
