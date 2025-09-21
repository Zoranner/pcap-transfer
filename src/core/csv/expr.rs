//! CSV 默认值表达式解析与求值
//!
//! 支持的表达式类型：
//! - 字面量：`i32=100`、`hex=0xFF` 等
//! - 随机值：`rand(min,max)` 或 `rand()`（布尔类型）
//! - 循环值：`loop(值1,值2,值3,...)`
//!
//! 相关文档：doc/表格结构设计.md

use crate::app::error::types::{AppError, Result};
use crate::core::csv::types::{
    parse_hex_string, CsvDataType,
};
use rand::Rng;

/// 默认值表达式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultExpr {
    /// 直接字面量（按列的数据类型进行解析）
    Literal(String),
    /// 随机整数（根据具体的数据类型边界进行校验与截断）
    RandInt { min: i128, max: i128 },
    /// 随机无符号整数
    RandUint { min: u128, max: u128 },
    /// 随机浮点
    RandFloat { min: f64, max: f64 },
    /// 随机布尔
    RandBool,
    /// 随机十六进制，按 byte_size 生成小端序字节
    RandHex {
        min: u128,
        max: u128,
        byte_size: usize,
    },
    /// 循环取值（按行索引轮询），值以原始字符串形式存储
    Loop(Vec<String>),
}

impl DefaultExpr {
    /// 解析默认表达式字符串
    pub fn parse(
        expr: &str,
        data_type: &CsvDataType,
    ) -> std::result::Result<Self, String> {
        let expr = expr.trim();

        // rand()
        if let Some(inner) = expr.strip_prefix("rand(") {
            let inner =
                inner.strip_suffix(')').ok_or_else(
                    || "rand(...) missing ')'".to_string(),
                )?;
            return Self::parse_rand_expr(inner, data_type);
        }

        // loop()
        if let Some(inner) = expr.strip_prefix("loop(") {
            let inner =
                inner.strip_suffix(')').ok_or_else(
                    || "loop(...) missing ')'".to_string(),
                )?;
            return Self::parse_loop_expr(inner);
        }

        // 字面量默认值
        Ok(DefaultExpr::Literal(expr.to_string()))
    }

    /// 解析随机表达式
    fn parse_rand_expr(
        inner: &str,
        data_type: &CsvDataType,
    ) -> std::result::Result<Self, String> {
        match data_type {
            CsvDataType::Bool => {
                if !inner.trim().is_empty() {
                    return Err("bool rand() should not have arguments".to_string());
                }
                Ok(DefaultExpr::RandBool)
            }
            CsvDataType::F32 | CsvDataType::F64 => {
                let parts: Vec<&str> = inner
                    .split(',')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.len() != 2 {
                    return Err("rand(min,max) requires two float args".to_string());
                }
                let min: f64 =
                    parts[0].parse().map_err(|_| {
                        "Invalid float min".to_string()
                    })?;
                let max: f64 =
                    parts[1].parse().map_err(|_| {
                        "Invalid float max".to_string()
                    })?;
                if min > max {
                    return Err("rand: min must be <= max"
                        .to_string());
                }
                Ok(DefaultExpr::RandFloat { min, max })
            }
            CsvDataType::I8
            | CsvDataType::I16
            | CsvDataType::I32
            | CsvDataType::I64 => {
                let parts: Vec<&str> = inner
                    .split(',')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.len() != 2 {
                    return Err("rand(min,max) requires two integer args".to_string());
                }
                let min: i128 =
                    parts[0].parse().map_err(|_| {
                        "Invalid int min".to_string()
                    })?;
                let max: i128 =
                    parts[1].parse().map_err(|_| {
                        "Invalid int max".to_string()
                    })?;
                if min > max {
                    return Err("rand: min must be <= max"
                        .to_string());
                }
                Ok(DefaultExpr::RandInt { min, max })
            }
            CsvDataType::U8
            | CsvDataType::U16
            | CsvDataType::U32
            | CsvDataType::U64 => {
                let parts: Vec<&str> = inner
                    .split(',')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.len() != 2 {
                    return Err("rand(min,max) requires two integer args".to_string());
                }
                let min: u128 =
                    parts[0].parse().map_err(|_| {
                        "Invalid uint min".to_string()
                    })?;
                let max: u128 =
                    parts[1].parse().map_err(|_| {
                        "Invalid uint max".to_string()
                    })?;
                if min > max {
                    return Err("rand: min must be <= max"
                        .to_string());
                }
                Ok(DefaultExpr::RandUint { min, max })
            }
            CsvDataType::HexDynamic(size) => {
                let parts: Vec<&str> = inner
                    .split(',')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                if parts.len() != 2 {
                    return Err("rand(min,max) requires two hex args".to_string());
                }
                let min = parse_hex_u128(parts[0])?;
                let max = parse_hex_u128(parts[1])?;
                if min > max {
                    return Err("rand: min must be <= max"
                        .to_string());
                }
                let byte_size = *size;
                // 校验范围是否能放入 byte_size
                let max_allowed: u128 = if byte_size >= 16 {
                    u128::MAX
                } else {
                    (1u128 << (byte_size as u32 * 8)) - 1
                };
                if max > max_allowed {
                    return Err(format!("hex rand range exceeds size {} bytes", byte_size));
                }
                Ok(DefaultExpr::RandHex {
                    min,
                    max,
                    byte_size,
                })
            }
        }
    }

    /// 解析循环表达式
    fn parse_loop_expr(
        inner: &str,
    ) -> std::result::Result<Self, String> {
        let items: Vec<String> = inner
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        if items.len() < 2 {
            return Err(
                "loop requires at least two values"
                    .to_string(),
            );
        }
        Ok(DefaultExpr::Loop(items))
    }

    /// 求值默认表达式，返回字节数组
    pub fn evaluate(
        &self,
        data_type: &CsvDataType,
        row_index: usize,
    ) -> Result<Vec<u8>> {
        match (self, data_type) {
            // 字面量求值
            (DefaultExpr::Literal(s), CsvDataType::I8) => {
                let val =
                    parse_numeric_literal::<i8>(s, "i8")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::I16) => {
                let val =
                    parse_numeric_literal::<i16>(s, "i16")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::I32) => {
                let val =
                    parse_numeric_literal::<i32>(s, "i32")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::I64) => {
                let val =
                    parse_numeric_literal::<i64>(s, "i64")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::U8) => {
                let val =
                    parse_numeric_literal::<u8>(s, "u8")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::U16) => {
                let val =
                    parse_numeric_literal::<u16>(s, "u16")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::U32) => {
                let val =
                    parse_numeric_literal::<u32>(s, "u32")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::U64) => {
                let val =
                    parse_numeric_literal::<u64>(s, "u64")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::F32) => {
                let val =
                    parse_numeric_literal::<f32>(s, "f32")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (DefaultExpr::Literal(s), CsvDataType::F64) => {
                let val =
                    parse_numeric_literal::<f64>(s, "f64")?;
                Ok(val.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::Literal(s),
                CsvDataType::Bool,
            ) => {
                let val = parse_bool_literal(s)?;
                Ok(vec![if val { 1 } else { 0 }])
            }
            (
                DefaultExpr::Literal(s),
                CsvDataType::HexDynamic(size),
            ) => {
                let bytes =
                    parse_hex_string(s).map_err(|e| {
                        AppError::validation("hex", e)
                    })?;
                if bytes.len() != *size {
                    return Err(AppError::validation("hex", format!("Hex value length {} doesn't match expected size {}", bytes.len(), size)));
                }
                Ok(bytes)
            }

            // 随机值求值
            (
                DefaultExpr::RandInt { min, max },
                CsvDataType::I8,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as i64)..=(*max as i64),
                ) as i8;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandInt { min, max },
                CsvDataType::I16,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as i64)..=(*max as i64),
                ) as i16;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandInt { min, max },
                CsvDataType::I32,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as i64)..=(*max as i64),
                ) as i32;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandInt { min, max },
                CsvDataType::I64,
            ) => {
                let mut rng = rand::thread_rng();
                let v =
                    rng.gen_range((*min)..=(*max)) as i64;
                Ok(v.to_le_bytes().to_vec())
            }

            (
                DefaultExpr::RandUint { min, max },
                CsvDataType::U8,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as u64)..=(*max as u64),
                ) as u8;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandUint { min, max },
                CsvDataType::U16,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as u64)..=(*max as u64),
                ) as u16;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandUint { min, max },
                CsvDataType::U32,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(
                    (*min as u64)..=(*max as u64),
                ) as u32;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandUint { min, max },
                CsvDataType::U64,
            ) => {
                let mut rng = rand::thread_rng();
                let v =
                    rng.gen_range((*min)..=(*max)) as u64;
                Ok(v.to_le_bytes().to_vec())
            }

            (
                DefaultExpr::RandFloat { min, max },
                CsvDataType::F32,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(*min..=*max) as f32;
                Ok(v.to_le_bytes().to_vec())
            }
            (
                DefaultExpr::RandFloat { min, max },
                CsvDataType::F64,
            ) => {
                let mut rng = rand::thread_rng();
                let v = rng.gen_range(*min..=*max);
                Ok(v.to_le_bytes().to_vec())
            }

            (DefaultExpr::RandBool, CsvDataType::Bool) => {
                let mut rng = rand::thread_rng();
                let v: bool = rng.gen();
                Ok(vec![if v { 1 } else { 0 }])
            }

            (
                DefaultExpr::RandHex {
                    min,
                    max,
                    byte_size,
                },
                CsvDataType::HexDynamic(size),
            ) => {
                let mut rng = rand::thread_rng();
                let v = if *min == *max {
                    *min
                } else {
                    rng.gen_range(*min..=*max)
                };
                if byte_size != size {
                    return Err(AppError::validation("hex", "hex rand byte_size mismatch with type"));
                }
                let mut bytes = vec![0u8; *size];
                for (i, byte) in
                    bytes.iter_mut().enumerate().take(*size)
                {
                    *byte = ((v >> (8 * i)) & 0xFF) as u8;
                }
                Ok(bytes)
            }

            // 循环值求值
            (DefaultExpr::Loop(items), ty) => {
                let idx = row_index % items.len();
                parse_cell_value_by_type(ty, &items[idx])
            }

            // 兜底：类型与表达式不匹配
            _ => Ok(data_type.default_value()),
        }
    }
}

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

/// 根据数据类型解析单元格值
pub fn parse_cell_value_by_type(
    data_type: &CsvDataType,
    value: &str,
) -> Result<Vec<u8>> {
    let v = value.trim();
    match data_type {
        CsvDataType::I8 => {
            let val: i8 = parse_numeric_literal(v, "i8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::I16 => {
            let val: i16 = parse_numeric_literal(v, "i16")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::I32 => {
            let val: i32 = parse_numeric_literal(v, "i32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::I64 => {
            let val: i64 = parse_numeric_literal(v, "i64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::U8 => {
            let val: u8 = parse_numeric_literal(v, "u8")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::U16 => {
            let val: u16 = parse_numeric_literal(v, "u16")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::U32 => {
            let val: u32 = parse_numeric_literal(v, "u32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::U64 => {
            let val: u64 = parse_numeric_literal(v, "u64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::F32 => {
            let val: f32 = parse_numeric_literal(v, "f32")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::F64 => {
            let val: f64 = parse_numeric_literal(v, "f64")?;
            Ok(val.to_le_bytes().to_vec())
        }
        CsvDataType::Bool => {
            let val = parse_bool_literal(v)?;
            Ok(vec![if val { 1 } else { 0 }])
        }
        CsvDataType::HexDynamic(size) => {
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
