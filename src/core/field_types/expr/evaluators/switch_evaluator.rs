//! 切换表达式求值器

use crate::app::error::types::Result;
use crate::core::field_types::expr::types::{
    SwitchCondition, SwitchRule,
};
use crate::core::field_types::expr::utils::parse_field_value_by_type;
use crate::core::field_types::types::FieldDataType;

/// 求值切换表达式
pub fn evaluate_switch(
    default_value: &str,
    rules: &[SwitchRule],
    row_index: usize,
    total_packets: Option<u64>,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    // 将 row_index 转换为包索引（从1开始计数）
    let packet_index = row_index + 1;

    // 检查是否匹配任何规则
    for rule in rules {
        if matches_condition(
            &rule.condition,
            packet_index,
            total_packets,
        ) {
            return parse_field_value_by_type(
                data_type,
                &rule.value,
            );
        }
    }

    // 没有匹配的规则，使用默认值
    parse_field_value_by_type(data_type, default_value)
}

/// 检查包索引是否匹配切换条件
fn matches_condition(
    condition: &SwitchCondition,
    packet_index: usize,
    total_packets: Option<u64>,
) -> bool {
    match condition {
        SwitchCondition::Absolute(pos) => {
            packet_index == *pos
        }
        SwitchCondition::Relative(offset) => {
            if let Some(total) = total_packets {
                if total == 0 {
                    // 无限发送模式，相对位置无意义
                    false
                } else {
                    let target_pos = (total as i64)
                        + (*offset as i64)
                        + 1;
                    target_pos > 0
                        && (packet_index as i64)
                            == target_pos
                }
            } else {
                // 没有总包数信息，无法计算相对位置
                false
            }
        }
        SwitchCondition::Range(start, end) => {
            packet_index >= *start && packet_index <= *end
        }
    }
}
