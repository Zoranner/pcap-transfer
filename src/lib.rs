//! 参数发送器库
//!
//! 高性能数据包传输测试工具
//!
//! ## 功能特性
//!
//! - 多报文配置和管理
//! - 可编辑字段类型系统
//! - 实时传输统计
//! - 跨平台配置管理
//! - 统一错误处理
//!
//! ## 使用示例
//!
//! ```no_run
//! use param_sender::app::config::manager::ConfigManager;
//! use param_sender::core::field_types::FieldDataType;
//!
//! // 创建配置管理器
//! let mut config_manager = ConfigManager::new()?;
//! config_manager.load()?;
//!
//! // 获取消息配置
//! let messages = config_manager.get_messages();
//! println!("Loaded {} messages", messages.len());
//! # Ok::<(), param_sender::app::error::types::DataTransferError>(())
//! ```

pub mod app;
pub mod core;
pub mod ui;
pub mod utils;

// 重新导出主要功能
pub use app::config::message_types::{
    FieldValue, MessageDefinition, MessageField,
    MessageRuntimeState,
};
pub use app::error::types::{DataTransferError, Result};
pub use core::field_types::{
    parse_field_value_by_type, DefaultExpr, FieldDataType,
};
