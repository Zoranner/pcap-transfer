//! 表达式解析器模块
//!
//! 提供各种表达式类型的解析功能

pub mod literal_parser;
pub mod loop_parser;
pub mod random_parser;
pub mod switch_parser;

// 重新导出主要的解析函数
pub use literal_parser::parse_literal;
pub use loop_parser::parse_loop_expr;
pub use random_parser::parse_random_expr;
pub use switch_parser::parse_switch_expr;
