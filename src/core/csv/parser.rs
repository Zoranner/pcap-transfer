//! CSV数据解析器

use crate::app::error::types::{AppError, Result};
use crate::core::csv::expr::parse_cell_value_by_type;
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

        // 解析数据类型与默认表达式
        let mut columns = Vec::new();
        for (i, (_name, type_str)) in column_names
            .iter()
            .zip(data_types.iter())
            .enumerate()
        {
            let (data_type, default_expr) =
                CsvDataType::parse_type_and_default(
                    type_str,
                )
                .map_err(|e| {
                    AppError::validation(
                        format!("Column {}", i + 1),
                        e,
                    )
                })?;

            columns.push(CsvColumn {
                data_type,
                default_expr,
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
                    format!("CSV Line {}", line_num + 2),
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

        for (col_index, (column, value)) in
            self.columns.iter().zip(row.iter()).enumerate()
        {
            let bytes = self.value_bytes_with_default(
                column, value, row_index, col_index,
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

    /// 根据优先级规则获取字段值的字节表示
    fn value_bytes_with_default(
        &self,
        column: &CsvColumn,
        cell_value: &str,
        row_index: usize,
        _col_index: usize,
    ) -> Result<Vec<u8>> {
        let data_type = &column.data_type;
        let cell_trimmed = cell_value.trim();

        // 规则：单元格优先
        if !cell_trimmed.is_empty() {
            return parse_cell_value_by_type(
                data_type,
                cell_trimmed,
            );
        }

        // 无单元格值，尝试默认表达式
        if let Some(expr) = &column.default_expr {
            return expr.evaluate(data_type, row_index);
        }

        // 无默认表达式，使用类型默认值
        Ok(data_type.default_value())
    }
}
