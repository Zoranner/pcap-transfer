//! CSV数据类型定义

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
    /// 十六进制值（固定值）
    HexFixed(Vec<u8>),
    /// 十六进制值（动态值）
    HexDynamic(usize),
}

impl CsvDataType {
    /// 从字符串解析数据类型
    pub fn from_string(
        type_str: &str,
    ) -> Result<Self, String> {
        match type_str.trim() {
            "i8" => Ok(CsvDataType::I8),
            "i16" => Ok(CsvDataType::I16),
            "i32" => Ok(CsvDataType::I32),
            "i64" => Ok(CsvDataType::I64),
            "u8" => Ok(CsvDataType::U8),
            "u16" => Ok(CsvDataType::U16),
            "u32" => Ok(CsvDataType::U32),
            "u64" => Ok(CsvDataType::U64),
            "f32" => Ok(CsvDataType::F32),
            "f64" => Ok(CsvDataType::F64),
            "bool" => Ok(CsvDataType::Bool),
            s if s.starts_with("hex=") => {
                let hex_value = &s[4..];
                let bytes = parse_hex_string(hex_value)?;
                Ok(CsvDataType::HexFixed(bytes))
            }
            s if s.starts_with("hex_")
                && s.contains('=') =>
            {
                let parts: Vec<&str> =
                    s.split('=').collect();
                if parts.len() != 2 {
                    return Err(format!(
                        "Invalid hex format: {}",
                        s
                    ));
                }
                let size_part = parts[0];
                let hex_value = parts[1];

                let size_str = &size_part[4..];
                let size: usize =
                    size_str.parse().map_err(|_| {
                        format!(
                            "Invalid hex size: {}",
                            size_str
                        )
                    })?;

                let bytes = parse_hex_string(hex_value)?;
                if bytes.len() != size {
                    return Err(format!("Hex value length {} doesn't match specified size {}", bytes.len(), size));
                }
                Ok(CsvDataType::HexFixed(bytes))
            }
            s if s.starts_with("hex_") => {
                let size_str = &s[4..];
                let size: usize =
                    size_str.parse().map_err(|_| {
                        format!(
                            "Invalid hex size: {}",
                            size_str
                        )
                    })?;
                Ok(CsvDataType::HexDynamic(size))
            }
            "hex" => Ok(CsvDataType::HexDynamic(1)),
            _ => Err(format!(
                "Unknown data type: {}",
                type_str
            )),
        }
    }

    /// 获取数据类型的字节大小
    #[allow(dead_code)]
    pub fn byte_size(&self) -> usize {
        match self {
            CsvDataType::I8 | CsvDataType::U8 => 1,
            CsvDataType::I16 | CsvDataType::U16 => 2,
            CsvDataType::I32
            | CsvDataType::U32
            | CsvDataType::F32 => 4,
            CsvDataType::I64
            | CsvDataType::U64
            | CsvDataType::F64 => 8,
            CsvDataType::Bool => 1,
            CsvDataType::HexFixed(bytes) => bytes.len(),
            CsvDataType::HexDynamic(size) => *size,
        }
    }

    /// 获取默认值
    #[allow(dead_code)]
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
            CsvDataType::HexFixed(bytes) => bytes.clone(),
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
            CsvDataType::HexFixed(bytes) => {
                let hex_str = bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join("");
                write!(f, "hex=0x{}", hex_str)
            }
            CsvDataType::HexDynamic(size) => {
                write!(f, "hex_{}", size)
            }
        }
    }
}

/// CSV列定义
#[derive(Debug, Clone)]
pub struct CsvColumn {
    #[allow(dead_code)]
    pub name: String,
    pub data_type: CsvDataType,
}

/// CSV数据包定义
#[derive(Debug, Clone)]
pub struct CsvPacket {
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 解析十六进制字符串
fn parse_hex_string(
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
        assert_eq!(
            CsvDataType::from_string("i32").unwrap(),
            CsvDataType::I32
        );
        assert_eq!(
            CsvDataType::from_string("f32").unwrap(),
            CsvDataType::F32
        );
        assert_eq!(
            CsvDataType::from_string("bool").unwrap(),
            CsvDataType::Bool
        );

        let hex_fixed =
            CsvDataType::from_string("hex=0xFF").unwrap();
        assert_eq!(
            hex_fixed,
            CsvDataType::HexFixed(vec![0xFF])
        );

        let hex_dynamic =
            CsvDataType::from_string("hex_4").unwrap();
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
