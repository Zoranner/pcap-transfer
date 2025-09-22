//! 字段数据类型定义

use crate::core::field_types::expr::DefaultExpr;
use std::fmt;

/// 字段数据类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FieldDataType {
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

impl FieldDataType {
    /// 基于字段类型字符串解析出基础类型与默认表达式
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
            "i8" => FieldDataType::I8,
            "i16" => FieldDataType::I16,
            "i32" => FieldDataType::I32,
            "i64" => FieldDataType::I64,
            "u8" => FieldDataType::U8,
            "u16" => FieldDataType::U16,
            "u32" => FieldDataType::U32,
            "u64" => FieldDataType::U64,
            "f32" => FieldDataType::F32,
            "f64" => FieldDataType::F64,
            "bool" => FieldDataType::Bool,
            b if b.starts_with("hex_") => {
                let size_str = &b[4..];
                let size: usize =
                    size_str.parse().map_err(|_| {
                        format!(
                            "Invalid hex size: {}",
                            size_str
                        )
                    })?;
                FieldDataType::HexDynamic(size)
            }
            "hex" => FieldDataType::HexDynamic(1),
            _ => {
                return Err(format!(
                    "Unknown data type: {}",
                    base
                ))
            }
        };

        // 解析默认表达式（若存在）
        let default_expr = if let Some(expr) = expr_opt {
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
                FieldDataType::HexDynamic(size),
                Some(DefaultExpr::Literal(s)),
            ) => {
                // 尝试按 hex 解析以推导长度；如 header 为 hex 而非 hex_N，让长度以字面量为准
                if *size == 1 {
                    if let Ok(bytes) = crate::core::field_types::parser::parse_hex_string(s) {
                        FieldDataType::HexDynamic(bytes.len())
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
            FieldDataType::I8 => vec![0],
            FieldDataType::I16 => vec![0, 0],
            FieldDataType::I32 => vec![0, 0, 0, 0],
            FieldDataType::I64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            FieldDataType::U8 => vec![0],
            FieldDataType::U16 => vec![0, 0],
            FieldDataType::U32 => vec![0, 0, 0, 0],
            FieldDataType::U64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            FieldDataType::F32 => vec![0, 0, 0, 0],
            FieldDataType::F64 => {
                vec![0, 0, 0, 0, 0, 0, 0, 0]
            }
            FieldDataType::Bool => vec![0],
            FieldDataType::HexDynamic(size) => {
                vec![0; *size]
            }
        }
    }
}

impl fmt::Display for FieldDataType {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            FieldDataType::I8 => write!(f, "i8"),
            FieldDataType::I16 => write!(f, "i16"),
            FieldDataType::I32 => write!(f, "i32"),
            FieldDataType::I64 => write!(f, "i64"),
            FieldDataType::U8 => write!(f, "u8"),
            FieldDataType::U16 => write!(f, "u16"),
            FieldDataType::U32 => write!(f, "u32"),
            FieldDataType::U64 => write!(f, "u64"),
            FieldDataType::F32 => write!(f, "f32"),
            FieldDataType::F64 => write!(f, "f64"),
            FieldDataType::Bool => write!(f, "bool"),
            FieldDataType::HexDynamic(size) => {
                write!(f, "hex_{}", size)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_types() {
        let (i32_type, _) =
            FieldDataType::parse_type_and_default("i32")
                .unwrap();
        assert_eq!(i32_type, FieldDataType::I32);

        let (f32_type, _) =
            FieldDataType::parse_type_and_default("f32")
                .unwrap();
        assert_eq!(f32_type, FieldDataType::F32);

        let (bool_type, _) =
            FieldDataType::parse_type_and_default("bool")
                .unwrap();
        assert_eq!(bool_type, FieldDataType::Bool);

        let (hex_dynamic, _) =
            FieldDataType::parse_type_and_default("hex_4")
                .unwrap();
        assert_eq!(
            hex_dynamic,
            FieldDataType::HexDynamic(4)
        );
    }
}
