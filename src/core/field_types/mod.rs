//! 字段类型和表达式处理模块
//!
//! 提供统一的字段类型定义、表达式解析和值转换功能
//! 支持配置文件中定义的各种字段类型和默认值表达式

pub mod expr;
pub mod parser;
pub mod types;

pub use expr::{parse_field_value_by_type, DefaultExpr};
pub use types::FieldDataType;
