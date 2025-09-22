//! 字面量表达式求值器

use crate::app::error::types::{AppError, Result};
use crate::core::field_types::expr::utils::{
    parse_bool_literal, parse_numeric_literal,
};
use crate::core::field_types::parser::parse_hex_string;
use crate::core::field_types::types::FieldDataType;

/// 求值字面量表达式
pub fn evaluate_literal(
    literal: &str,
    data_type: &FieldDataType,
) -> Result<Vec<u8>> {
    match data_type {
        FieldDataType::I8 => {
            let val =
                parse_numeric_literal::<i8>(literal, "i8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I16 => {
            let val = parse_numeric_literal::<i16>(
                literal, "i16",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I32 => {
            let val = parse_numeric_literal::<i32>(
                literal, "i32",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I64 => {
            let val = parse_numeric_literal::<i64>(
                literal, "i64",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U8 => {
            let val =
                parse_numeric_literal::<u8>(literal, "u8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U16 => {
            let val = parse_numeric_literal::<u16>(
                literal, "u16",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U32 => {
            let val = parse_numeric_literal::<u32>(
                literal, "u32",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U64 => {
            let val = parse_numeric_literal::<u64>(
                literal, "u64",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::F32 => {
            let val = parse_numeric_literal::<f32>(
                literal, "f32",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::F64 => {
            let val = parse_numeric_literal::<f64>(
                literal, "f64",
            )?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::Bool => {
            let val = parse_bool_literal(literal)?;
            Ok(vec![if val { 1 } else { 0 }])
        }
        FieldDataType::HexDynamic(size) => {
            let bytes =
                parse_hex_string(literal).map_err(|e| {
                    AppError::validation("hex", e)
                })?;
            if bytes.len() != *size {
                return Err(AppError::validation("hex", format!("Hex value length {} doesn't match expected size {}", bytes.len(), size)));
            }
            Ok(bytes)
        }
    }
}
