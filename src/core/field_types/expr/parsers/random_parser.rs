//! 随机表达式解析器

use crate::core::field_types::expr::types::DefaultExpr;
use crate::core::field_types::expr::utils::parse_hex_u128;
use crate::core::field_types::types::FieldDataType;

/// 解析随机表达式
pub fn parse_random_expr(
    inner: &str,
    data_type: &FieldDataType,
) -> std::result::Result<DefaultExpr, String> {
    match data_type {
        FieldDataType::Bool => {
            if !inner.trim().is_empty() {
                return Err(
                    "bool rand() should not have arguments"
                        .to_string(),
                );
            }
            Ok(DefaultExpr::RandBool)
        }
        FieldDataType::F32 | FieldDataType::F64 => {
            parse_float_random(inner)
        }
        FieldDataType::I8
        | FieldDataType::I16
        | FieldDataType::I32
        | FieldDataType::I64 => parse_int_random(inner),
        FieldDataType::U8
        | FieldDataType::U16
        | FieldDataType::U32
        | FieldDataType::U64 => parse_uint_random(inner),
        FieldDataType::HexDynamic(size) => {
            parse_hex_random(inner, *size)
        }
    }
}

/// 解析浮点随机表达式
fn parse_float_random(
    inner: &str,
) -> std::result::Result<DefaultExpr, String> {
    let parts: Vec<&str> = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 2 {
        return Err(
            "rand(min,max) requires two float args"
                .to_string(),
        );
    }
    let min: f64 = parts[0]
        .parse()
        .map_err(|_| "Invalid float min".to_string())?;
    let max: f64 = parts[1]
        .parse()
        .map_err(|_| "Invalid float max".to_string())?;
    if min > max {
        return Err("rand: min must be <= max".to_string());
    }
    Ok(DefaultExpr::RandFloat { min, max })
}

/// 解析有符号整数随机表达式
fn parse_int_random(
    inner: &str,
) -> std::result::Result<DefaultExpr, String> {
    let parts: Vec<&str> = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 2 {
        return Err(
            "rand(min,max) requires two integer args"
                .to_string(),
        );
    }
    let min: i128 = parts[0]
        .parse()
        .map_err(|_| "Invalid int min".to_string())?;
    let max: i128 = parts[1]
        .parse()
        .map_err(|_| "Invalid int max".to_string())?;
    if min > max {
        return Err("rand: min must be <= max".to_string());
    }
    Ok(DefaultExpr::RandInt { min, max })
}

/// 解析无符号整数随机表达式
fn parse_uint_random(
    inner: &str,
) -> std::result::Result<DefaultExpr, String> {
    let parts: Vec<&str> = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 2 {
        return Err(
            "rand(min,max) requires two integer args"
                .to_string(),
        );
    }
    let min: u128 = parts[0]
        .parse()
        .map_err(|_| "Invalid uint min".to_string())?;
    let max: u128 = parts[1]
        .parse()
        .map_err(|_| "Invalid uint max".to_string())?;
    if min > max {
        return Err("rand: min must be <= max".to_string());
    }
    Ok(DefaultExpr::RandUint { min, max })
}

/// 解析十六进制随机表达式
fn parse_hex_random(
    inner: &str,
    size: usize,
) -> std::result::Result<DefaultExpr, String> {
    let parts: Vec<&str> = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 2 {
        return Err("rand(min,max) requires two hex args"
            .to_string());
    }
    let min = parse_hex_u128(parts[0])?;
    let max = parse_hex_u128(parts[1])?;
    if min > max {
        return Err("rand: min must be <= max".to_string());
    }
    let byte_size = size;
    // 校验范围是否能放入 byte_size
    let max_allowed: u128 = if byte_size >= 16 {
        u128::MAX
    } else {
        (1u128 << (byte_size as u32 * 8)) - 1
    };
    if max > max_allowed {
        return Err(format!(
            "hex rand range exceeds size {} bytes",
            byte_size
        ));
    }
    Ok(DefaultExpr::RandHex {
        min,
        max,
        byte_size,
    })
}
