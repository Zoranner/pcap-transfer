//! 循环表达式求值器

use crate::app::error::types::Result;
use crate::core::field_types::expr::utils::parse_field_value_by_type;
use crate::core::field_types::types::FieldDataType;

/// 求值循环表达式
pub fn evaluate_loop(
    items: &[String],
    row_index: usize,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    let idx = row_index % items.len();
    parse_field_value_by_type(data_type, &items[idx])
}
