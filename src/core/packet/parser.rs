//! 数据包解析器实现

use crate::app::config::message_types::MessageRuntimeState;
use crate::core::field_types::FieldDataType;

/// 数据包解析器
pub struct PacketParser;

impl PacketParser {
    /// 解析数据包内容并格式化输出
    pub fn parse_and_format_packet(
        runtime_msg: &MessageRuntimeState,
        packet_data: &[u8],
        packet_number: u64,
    ) -> String {
        let mut output = String::new();
        
        // 添加包头信息
        output.push_str("\n");
        output.push_str(&format!(
            "Packet parsing result - {} #{} ({} bytes)\n",
            runtime_msg.definition.name,
            packet_number,
            packet_data.len()
        ));
        
        let mut offset = 0;
        
        // 遍历每个字段进行解析
        for field_value in &runtime_msg.field_values {
            if let Some(ref data_type) = field_value.parsed_type {
                let field_size = Self::get_field_size(data_type);
                
                if offset + field_size <= packet_data.len() {
                    let field_bytes = &packet_data[offset..offset + field_size];
                    let parsed_value = Self::parse_field_bytes(data_type, field_bytes);
                    let hex_str = Self::format_hex_bytes(field_bytes);
                    
                    output.push_str(&format!(
                        "  {} ({}) = {} [{}]\n",
                        field_value.name,
                        field_value.field_type,
                        parsed_value,
                        hex_str
                    ));
                    
                    offset += field_size;
                } else {
                    output.push_str(&format!(
                        "  {} ({}) = <数据不足> []\n",
                        field_value.name,
                        field_value.field_type
                    ));
                }
            } else {
                output.push_str(&format!(
                    "  {} ({}) = <解析失败> []\n",
                    field_value.name,
                    field_value.field_type
                ));
            }
        }
        
        output
    }
    
    /// 获取字段大小（字节数）
    fn get_field_size(data_type: &FieldDataType) -> usize {
        match data_type {
            FieldDataType::I8 | FieldDataType::U8 | FieldDataType::Bool => 1,
            FieldDataType::I16 | FieldDataType::U16 => 2,
            FieldDataType::I32 | FieldDataType::U32 | FieldDataType::F32 => 4,
            FieldDataType::I64 | FieldDataType::U64 | FieldDataType::F64 => 8,
            FieldDataType::HexDynamic(size) => *size,
        }
    }
    
    /// 解析字段字节为可读值
    fn parse_field_bytes(data_type: &FieldDataType, bytes: &[u8]) -> String {
        match data_type {
            FieldDataType::I8 => {
                if bytes.len() >= 1 {
                    format!("{}", i8::from_le_bytes([bytes[0]]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::I16 => {
                if bytes.len() >= 2 {
                    format!("{}", i16::from_le_bytes([bytes[0], bytes[1]]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::I32 => {
                if bytes.len() >= 4 {
                    format!("{}", i32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::I64 => {
                if bytes.len() >= 8 {
                    format!("{}", i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::U8 => {
                if bytes.len() >= 1 {
                    format!("{}", bytes[0])
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::U16 => {
                if bytes.len() >= 2 {
                    format!("{}", u16::from_le_bytes([bytes[0], bytes[1]]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::U32 => {
                if bytes.len() >= 4 {
                    format!("{}", u32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::U64 => {
                if bytes.len() >= 8 {
                    format!("{}", u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ]))
                } else {
                    "0".to_string()
                }
            }
            FieldDataType::F32 => {
                if bytes.len() >= 4 {
                    format!("{:.1}", f32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ]))
                } else {
                    "0.0".to_string()
                }
            }
            FieldDataType::F64 => {
                if bytes.len() >= 8 {
                    format!("{:.1}", f64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ]))
                } else {
                    "0.0".to_string()
                }
            }
            FieldDataType::Bool => {
                if bytes.len() >= 1 {
                    format!("{}", bytes[0] != 0)
                } else {
                    "false".to_string()
                }
            }
            FieldDataType::HexDynamic(_) => {
                // 对于 hex 类型，显示为十六进制字符串
                Self::format_hex_string(bytes)
            }
        }
    }
    
    /// 格式化字节为十六进制字符串（用于字段值显示）
    fn format_hex_string(bytes: &[u8]) -> String {
        format!("0x{}", bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("")
        )
    }
    
    /// 格式化字节为十六进制数组（用于 [XX XX XX] 格式）
    fn format_hex_bytes(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
