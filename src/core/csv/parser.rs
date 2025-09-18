//! CSV数据解析器

use crate::app::error::types::{AppError, Result};
use crate::core::csv::types::{
    CsvColumn, CsvDataType, CsvPacket,
};
use chrono::Utc;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// CSV解析器
pub struct CsvParser {
    columns: Vec<CsvColumn>,
    data_rows: Vec<Vec<String>>,
}

impl CsvParser {
    /// 从文件创建CSV解析器
    pub fn from_file<P: AsRef<Path>>(
        file_path: P,
    ) -> Result<Self> {
        let file = File::open(&file_path).map_err(|e| {
            AppError::config(format!(
                "Failed to open CSV file: {}",
                e
            ))
        })?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // 读取第一行（列名）
        let name_line = lines
            .next()
            .ok_or_else(|| {
                AppError::validation("CSV", "File is empty")
            })?
            .map_err(|e| {
                AppError::config(format!(
                    "Failed to read CSV file: {}",
                    e
                ))
            })?;

        // 读取第二行（数据类型定义）
        let type_line = lines.next()
            .ok_or_else(|| AppError::validation("CSV", "File must have at least 2 lines (names and types)"))?
            .map_err(|e| AppError::config(format!("Failed to read CSV file: {}", e)))?;

        let column_names = Self::parse_csv_line(&name_line);
        let data_types = Self::parse_csv_line(&type_line);

        if column_names.len() != data_types.len() {
            return Err(AppError::validation(
                "CSV",
                "Column names and data types count mismatch"
            ));
        }

        // 解析数据类型
        let mut columns = Vec::new();
        for (i, (name, type_str)) in column_names
            .iter()
            .zip(data_types.iter())
            .enumerate()
        {
            let data_type =
                CsvDataType::from_string(type_str)
                    .map_err(|e| {
                        AppError::validation(
                            &format!("Column {}", i + 1),
                            e,
                        )
                    })?;

            columns.push(CsvColumn {
                name: name.clone(),
                data_type,
            });
        }

        // 读取数据行
        let mut data_rows = Vec::new();
        for (line_num, line) in lines.enumerate() {
            let line = line.map_err(|e| {
                AppError::config(format!(
                    "Failed to read CSV line {}: {}",
                    line_num + 2,
                    e
                ))
            })?;
            let row = Self::parse_csv_line(&line);

            if row.len() != columns.len() {
                return Err(AppError::validation(
                    &format!("CSV Line {}", line_num + 2),
                    format!(
                        "Expected {} columns, got {}",
                        columns.len(),
                        row.len()
                    ),
                ));
            }

            data_rows.push(row);
        }

        Ok(Self { columns, data_rows })
    }

    /// 解析CSV行
    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        // 检查是否是转义的引号
                        if chars.peek() == Some(&'"') {
                            chars.next(); // 跳过下一个引号
                            current.push('"');
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                ',' if !in_quotes => {
                    result.push(current.trim().to_string());
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        result.push(current.trim().to_string());
        result
    }

    /// 获取列定义
    #[allow(dead_code)]
    pub fn columns(&self) -> &[CsvColumn] {
        &self.columns
    }

    /// 获取数据行数
    pub fn row_count(&self) -> usize {
        self.data_rows.len()
    }

    /// 生成指定行的UDP数据包
    pub fn generate_packet(
        &self,
        row_index: usize,
    ) -> Result<CsvPacket> {
        if row_index >= self.data_rows.len() {
            return Err(AppError::validation(
                "Row Index",
                format!(
                    "Row index {} out of range (max: {})",
                    row_index,
                    self.data_rows.len() - 1
                ),
            ));
        }

        let row = &self.data_rows[row_index];
        let mut packet_data = Vec::new();

        for (column, value) in
            self.columns.iter().zip(row.iter())
        {
            let bytes = self.parse_value_to_bytes(
                &column.data_type,
                value,
            )?;
            packet_data.extend_from_slice(&bytes);
        }

        // 使用当前时间作为时间戳
        let timestamp = Utc::now();

        Ok(CsvPacket {
            data: packet_data,
            timestamp,
        })
    }

    /// 将值解析为字节数组
    fn parse_value_to_bytes(
        &self,
        data_type: &CsvDataType,
        value: &str,
    ) -> Result<Vec<u8>> {
        let trimmed_value = value.trim();

        match data_type {
            CsvDataType::I8 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<i8>()
                        .map_err(|e| AppError::validation("i8", format!("Invalid i8 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::I16 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<i16>()
                        .map_err(|e| AppError::validation("i16", format!("Invalid i16 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::I32 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<i32>()
                        .map_err(|e| AppError::validation("i32", format!("Invalid i32 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::I64 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<i64>()
                        .map_err(|e| AppError::validation("i64", format!("Invalid i64 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::U8 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<u8>()
                        .map_err(|e| AppError::validation("u8", format!("Invalid u8 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::U16 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<u16>()
                        .map_err(|e| AppError::validation("u16", format!("Invalid u16 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::U32 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<u32>()
                        .map_err(|e| AppError::validation("u32", format!("Invalid u32 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::U64 => {
                let val = if trimmed_value.is_empty() {
                    0
                } else {
                    trimmed_value.parse::<u64>()
                        .map_err(|e| AppError::validation("u64", format!("Invalid u64 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::F32 => {
                let val = if trimmed_value.is_empty() {
                    0.0
                } else {
                    trimmed_value.parse::<f32>()
                        .map_err(|e| AppError::validation("f32", format!("Invalid f32 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::F64 => {
                let val = if trimmed_value.is_empty() {
                    0.0
                } else {
                    trimmed_value.parse::<f64>()
                        .map_err(|e| AppError::validation("f64", format!("Invalid f64 value '{}': {}", value, e)))?
                };
                Ok(val.to_le_bytes().to_vec())
            }
            CsvDataType::Bool => {
                let val = if trimmed_value.is_empty() {
                    false
                } else {
                    match trimmed_value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => true,
                        "false" | "0" | "no" | "off" => false,
                        _ => return Err(AppError::validation("bool", format!("Invalid bool value '{}'", value)))
                    }
                };
                Ok(vec![if val { 1 } else { 0 }])
            }
            CsvDataType::HexFixed(bytes) => {
                // 固定值，忽略输入值
                Ok(bytes.clone())
            }
            CsvDataType::HexDynamic(size) => {
                if trimmed_value.is_empty() {
                    Ok(vec![0; *size])
                } else {
                    // 解析十六进制值
                    let hex_str = trimmed_value.trim();
                    let hex_str = if hex_str
                        .starts_with("0x")
                        || hex_str.starts_with("0X")
                    {
                        &hex_str[2..]
                    } else {
                        hex_str
                    };

                    let mut bytes = Vec::new();
                    let hex_str = if hex_str.len() % 2 == 1
                    {
                        format!("0{}", hex_str)
                    } else {
                        hex_str.to_string()
                    };

                    for chunk in
                        hex_str.as_bytes().chunks(2)
                    {
                        let hex_pair = std::str::from_utf8(chunk)
                            .map_err(|_| AppError::validation("hex", "Invalid UTF-8 in hex string"))?;
                        let byte = u8::from_str_radix(
                            hex_pair, 16,
                        )
                        .map_err(|_| {
                            AppError::validation(
                                "hex",
                                format!(
                                    "Invalid hex value: {}",
                                    hex_pair
                                ),
                            )
                        })?;
                        bytes.push(byte);
                    }

                    // 确保长度匹配
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_csv_parser() {
        // 创建临时文件
        let temp_path =
            std::env::temp_dir().join("test.csv");
        let mut file = File::create(&temp_path).unwrap();
        writeln!(file, "id,age,score").unwrap(); // 列名
        writeln!(file, "i32,f32,i16").unwrap(); // 数据类型
        writeln!(file, "1,25.5,100").unwrap(); // 数据行1
        writeln!(file, "2,30.0,95").unwrap(); // 数据行2
        file.flush().unwrap();

        let parser =
            CsvParser::from_file(&temp_path).unwrap();
        assert_eq!(parser.columns().len(), 3);
        assert_eq!(parser.row_count(), 2);

        let packet = parser.generate_packet(0).unwrap();
        assert_eq!(packet.data.len(), 10); // i32(4) + f32(4) + i16(2) = 10 bytes

        // 清理临时文件
        std::fs::remove_file(&temp_path).unwrap();
    }
}
