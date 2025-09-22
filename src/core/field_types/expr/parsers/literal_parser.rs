//! 字面量表达式解析器

use crate::core::field_types::expr::types::DefaultExpr;

/// 解析字面量表达式
pub fn parse_literal(expr: &str) -> DefaultExpr {
    DefaultExpr::Literal(expr.to_string())
}
