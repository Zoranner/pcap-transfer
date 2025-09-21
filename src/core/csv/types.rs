//! CSV数据类型定义

use crate::core::csv::expr::DefaultExpr;
use std::fmt;

/// CSV数据类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum CsvDataType {
    /// 8位有符号整数
    I8,
    /// 16位有符号整数
    I16,
    /// 32位有符号整数
    I32,
    /// 64位有符号整数
    I64,
    /// 8位无符号整数
    U8,
    /// 16位无符号整数
    U16,
    /// 32位无符号整数
    U32,
    /// 64位无符号整数
    U64,
    /// 32位浮点数
    F32,
    /// 64位浮点数
    F64,
    /// 布尔值
    Bool,
    /// 十六进制值（固定长度）
    HexDynamic(usize),
}

impl CsvDataType {
    /// 基于列头定义解析出基础类型与默认表达式
    pub fn parse_type_and_default(
        type_str: &str,
    ) -> Result<(Self, Option<DefaultExpr>), String> {
        let s = type_str.trim();
        // 拆分 base=expr 形式；注意 hex_N=... 作为 base 是允许的
        let (base, expr_opt) =
            if let Some(eq_idx) = s.find('=') {
                let (b, rest) = s.split_at(eq_idx);
                let expr = &rest[1..];
                (b.trim(), Some(expr.trim()))
            } else {
                (s, None)
            };

        // 先解析基础类型（不考虑 = 右侧的内容）
        let base_ty = match base {
            "i8" => CsvDataType::I8,
            "i16" => CsvDataType::I16,
            "i32" => CsvDataType::I32,
            "i64" => CsvDataType::I64,
            "u8" => CsvDataType::U8,
            "u16" => CsvDataType::U16,
            "u32" => CsvDataType::U32,
            "u64" => CsvDataType::U64,
            "f32" => CsvDataType::F32,
            "f64" => CsvDataType::F64,
            "bool" => CsvDataType::Bool,
            b if b.starts_with("hex_") => {
                let size_str = &b[4..];
                let size: usize =
                    size_str.parse().map_err(|_| {
                        format!(
                            "Invalid hex size: {}",
                            size_str
                        )
                    })?;
                CsvDataType::HexDynamic(size)
            }
            "hex" => CsvDataType::HexDynamic(1),
            _ => {
                return Err(format!(
                    "Unknown data type: {}",
                    base
                ))
            }
        };

        // 解析默认表达式（若存在）
        let default_expr = if let Some(expr) = expr_opt {
            // DefaultExpr::parse 返回 Result<_, String>，此处转换为 String 仍然符合当前错误类型
            Some(
                DefaultExpr::parse(expr, &base_ty)
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };

        // 若是 hex 且给了字面量默认值，需要推导长度
        let final_ty = match (&base_ty, &default_expr) {
            (
                CsvDataType::HexDynamic(size),
                Some(DefaultExpr::Literal(s)),
            ) => {
                // 尝试按 hex 解析以推导长度；如 header 为 hex 而非 hex_N，让长度以字面量为准
                if *size == 1 {
                    if let Ok(bytes) = parse_hex_string(s) {
                        CsvDataType::HexDynamic(bytes.len())
                    } else {
                        base_ty.clone()
                    }
                } else {
                    base_ty.clone()
                }
            }
            _ => base_ty.clone(),
        };

        Ok((final_ty, default_expr))
    }

    /// 获取默认值
    pub fn default_value(&self) -> Vec<u8> {
        match self {
            CsvDataType::I8 => vec![0],
            CsvDataType::I16 => vec![0, 0],
            CsvDataType::I32 => vec![0, 0, 0, 0],
            CsvDataType::I64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            CsvDataType::U8 => vec![0],
            CsvDataType::U16 => vec![0, 0],
            CsvDataType::U32 => vec![0, 0, 0, 0],
            CsvDataType::U64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            CsvDataType::F32 => vec![0, 0, 0, 0],
            CsvDataType::F64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            CsvDataType::Bool => vec![0],
            CsvDataType::HexDynamic(size) => vec![0; *size],
        }
    }
}

impl fmt::Display for CsvDataType {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            CsvDataType::I8 => write!(f, "i8"),
            CsvDataType::I16 => write!(f, "i16"),
            CsvDataType::I32 => write!(f, "i32"),
            CsvDataType::I64 => write!(f, "i64"),
            CsvDataType::U8 => write!(f, "u8"),
            CsvDataType::U16 => write!(f, "u16"),
            CsvDataType::U32 => write!(f, "u32"),
            CsvDataType::U64 => write!(f, "u64"),
            CsvDataType::F32 => write!(f, "f32"),
            CsvDataType::F64 => write!(f, "f64"),
            CsvDataType::Bool => write!(f, "bool"),
            CsvDataType::HexDynamic(size) => {
                write!(f, "hex_{}", size)
            }
        }
    }
}

/// CSV列定义
#[derive(Debug, Clone)]
pub struct CsvColumn {
    pub data_type: CsvDataType,
    pub default_expr: Option<DefaultExpr>,
}

/// CSV数据包定义
#[derive(Debug, Clone)]
pub struct CsvPacket {
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 解析十六进制字符串
pub fn parse_hex_string(
    hex_str: &str,
) -> Result<Vec<u8>, String> {
    let hex_str = hex_str.trim();

    // 移除0x前缀
    let hex_str = if hex_str.starts_with("0x")
        || hex_str.starts_with("0X")
    {
        &hex_str[2..]
    } else {
        hex_str
    };

    if hex_str.is_empty() {
        return Ok(vec![0]);
    }

    // 确保长度为偶数
    let hex_str = if hex_str.len() % 2 == 1 {
        format!("0{}", hex_str)
    } else {
        hex_str.to_string()
    };

    let mut bytes = Vec::new();
    for chunk in hex_str.as_bytes().chunks(2) {
        let hex_pair = std::str::from_utf8(chunk)
            .map_err(|_| "Invalid UTF-8 in hex string")?;
        let byte = u8::from_str_radix(hex_pair, 16)
            .map_err(|_| {
                format!("Invalid hex value: {}", hex_pair)
            })?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_types() {
        let (i32_type, _) =
            CsvDataType::parse_type_and_default("i32")
                .unwrap();
        assert_eq!(i32_type, CsvDataType::I32);

        let (f32_type, _) =
            CsvDataType::parse_type_and_default("f32")
                .unwrap();
        assert_eq!(f32_type, CsvDataType::F32);

        let (bool_type, _) =
            CsvDataType::parse_type_and_default("bool")
                .unwrap();
        assert_eq!(bool_type, CsvDataType::Bool);

        let (hex_dynamic, _) =
            CsvDataType::parse_type_and_default("hex_4")
                .unwrap();
        assert_eq!(hex_dynamic, CsvDataType::HexDynamic(4));
    }

    #[test]
    fn test_parse_hex_string() {
        assert_eq!(
            parse_hex_string("0xFF").unwrap(),
            vec![0xFF]
        );
        assert_eq!(
            parse_hex_string("FF").unwrap(),
            vec![0xFF]
        );
        assert_eq!(
            parse_hex_string("1234").unwrap(),
            vec![0x12, 0x34]
        );
        assert_eq!(
            parse_hex_string("F").unwrap(),
            vec![0x0F]
        );
    }
}
