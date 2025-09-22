//! 表达式工具函数
//!
//! 提供各种表达式解析和求值所需的通用工具函数

use crate::app::error::types::{AppError, Result};
use crate::core::field_types::parser::parse_hex_string;
use crate::core::field_types::types::FieldDataType;

/// 解析十六进制字符串为 u128
pub fn parse_hex_u128(
    hex_str: &str,
) -> std::result::Result<u128, String> {
    let bytes = parse_hex_string(hex_str)?;
    let mut val: u128 = 0;
    // 以小端序组装（与字节生成一致）
    for (i, b) in bytes.iter().enumerate() {
        val |= (*b as u128) << (8 * i as u32);
    }
    Ok(val)
}

/// 解析数值字面量
pub fn parse_numeric_literal<T: std::str::FromStr>(
    raw: &str,
    ty_name: &str,
) -> Result<T> {
    raw.trim().parse::<T>().map_err(|_| {
        AppError::validation(
            ty_name,
            format!("Invalid {} value '{}'", ty_name, raw),
        )
    })
}

/// 解析布尔字面量
pub fn parse_bool_literal(raw: &str) -> Result<bool> {
    match raw.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(AppError::validation(
            "bool",
            format!("Invalid bool value '{}'", raw),
        )),
    }
}

/// 根据数据类型解析字段值
pub fn parse_field_value_by_type(
    data_type: &FieldDataType,
    value: &str,
) -> Result<Vec<u8>> {
    let v = value.trim();
    match data_type {
        FieldDataType::I8 => {
            let val: i8 = parse_numeric_literal(v, "i8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I16 => {
            let val: i16 = parse_numeric_literal(v, "i16")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I32 => {
            let val: i32 = parse_numeric_literal(v, "i32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::I64 => {
            let val: i64 = parse_numeric_literal(v, "i64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U8 => {
            let val: u8 = parse_numeric_literal(v, "u8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U16 => {
            let val: u16 = parse_numeric_literal(v, "u16")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U32 => {
            let val: u32 = parse_numeric_literal(v, "u32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::U64 => {
            let val: u64 = parse_numeric_literal(v, "u64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::F32 => {
            let val: f32 = parse_numeric_literal(v, "f32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::F64 => {
            let val: f64 = parse_numeric_literal(v, "f64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        FieldDataType::Bool => {
            let val = parse_bool_literal(v)?;
            Ok(vec![if val { 1 } else { 0 }])
        }
        FieldDataType::HexDynamic(size) => {
            // 解析十六进制值
            let bytes =
                parse_hex_string(v).map_err(|e| {
                    AppError::validation("hex", e)
                })?;
            if bytes.len() != *size {
                return Err(AppError::validation(
                    "hex",
                    format!("Hex value length {} doesn't match expected size {}", bytes.len(), size)
                ));
            }
            Ok(bytes)
        }
    }
}
