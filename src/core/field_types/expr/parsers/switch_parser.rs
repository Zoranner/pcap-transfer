//! 切换表达式解析器

use crate::core::field_types::expr::types::{
    DefaultExpr, SwitchCondition, SwitchRule,
};

/// 解析切换表达式
/// 语法：switch(默认值, 条件1:值1, 条件2:值2, ...)
/// 条件支持：
/// - 正数：绝对位置（从1开始）
/// - 负数：相对位置（-1表示最后一个包）
/// - 范围：start-end（如 3-5 表示第3到第5个包）
pub fn parse_switch_expr(
    inner: &str,
) -> std::result::Result<DefaultExpr, String> {
    let parts: Vec<&str> = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() < 2 {
        return Err("switch requires at least default value and one rule".to_string());
    }

    let default_value = parts[0].to_string();
    let mut rules = Vec::new();

    for rule_str in &parts[1..] {
        let rule_parts: Vec<&str> =
            rule_str.split(':').collect();
        if rule_parts.len() != 2 {
            return Err(format!("Invalid switch rule format: '{}', expected 'condition:value'", rule_str));
        }

        let condition_str = rule_parts[0].trim();
        let value = rule_parts[1].trim().to_string();

        let condition =
            parse_switch_condition(condition_str)?;
        rules.push(SwitchRule { condition, value });
    }

    Ok(DefaultExpr::Switch {
        default_value,
        rules,
    })
}

/// 解析切换条件
fn parse_switch_condition(
    condition_str: &str,
) -> std::result::Result<SwitchCondition, String> {
    // 先尝试解析为整数（包括负数）
    if let Ok(num) = condition_str.parse::<i32>() {
        if num > 0 {
            Ok(SwitchCondition::Absolute(num as usize))
        } else if num < 0 {
            Ok(SwitchCondition::Relative(num))
        } else {
            Err("Position cannot be 0, use positive numbers (1-based) or negative numbers (-1 for last)".to_string())
        }
    } else {
        // 检查是否是范围格式 (start-end)，只在不是负数的情况下检查
        if let Some(dash_pos) = condition_str.find('-') {
            // 确保不是负数（负数的 - 在开头）
            if dash_pos > 0 {
                let start_str = &condition_str[..dash_pos];
                let end_str =
                    &condition_str[dash_pos + 1..];

                let start: usize =
                    start_str.parse().map_err(|_| {
                        format!(
                            "Invalid range start: '{}'",
                            start_str
                        )
                    })?;
                let end: usize =
                    end_str.parse().map_err(|_| {
                        format!(
                            "Invalid range end: '{}'",
                            end_str
                        )
                    })?;

                if start == 0 || end == 0 {
                    return Err(
                        "Range positions must start from 1"
                            .to_string(),
                    );
                }
                if start > end {
                    return Err(
                        "Range start must be <= end"
                            .to_string(),
                    );
                }

                return Ok(SwitchCondition::Range(
                    start, end,
                ));
            }
        }

        Err(format!(
            "Invalid condition format: '{}'",
            condition_str
        ))
    }
}
