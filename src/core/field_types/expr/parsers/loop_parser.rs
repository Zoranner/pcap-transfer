//! 循环表达式解析器

use crate::core::field_types::expr::types::DefaultExpr;

/// 解析循环表达式
pub fn parse_loop_expr(
    inner: &str,
) -> std::result::Result<DefaultExpr, String> {
    let items: Vec<String> = inner
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if items.len() < 2 {
        return Err(
            "loop requires at least two values".to_string()
        );
    }
    Ok(DefaultExpr::Loop(items))
}
