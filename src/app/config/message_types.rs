//! 多报文配置数据结构定义

use crate::app::error::types::Result;
use crate::core::field_types::{
    DefaultExpr, FieldDataType,
};
use serde::{Deserialize, Serialize};

/// 发送策略
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq,
)]
#[serde(rename_all = "lowercase")]
pub enum SendStrategy {
    /// 顺序发送
    Sequential,
    /// 并行发送
    Parallel,
}

impl Default for SendStrategy {
    fn default() -> Self {
        Self::Sequential
    }
}

/// 网络配置
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Default,
)]
pub struct MessageNetworkConfig {
    /// 目标地址
    pub address: Option<String>,
    /// 目标端口
    pub port: Option<u16>,
    /// 网络类型
    pub network_type: Option<String>,
    /// 网络接口
    pub interface: Option<String>,
}

/// 字段定义
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq,
)]
pub struct MessageField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    #[serde(rename = "type")]
    pub field_type: String,
    /// 是否可编辑
    #[serde(default = "default_editable")]
    pub editable: bool,
}

fn default_editable() -> bool {
    true
}

/// 报文定义
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq,
)]
pub struct MessageDefinition {
    /// 消息名称
    pub name: String,
    /// 发送间隔（毫秒）
    pub interval: u64,
    /// 是否启用
    pub enabled: bool,
    /// 发送包数量（0表示无限）
    pub packet_count: u64,
    /// 网络配置
    pub network: Option<MessageNetworkConfig>,
    /// 字段列表
    pub fields: Vec<MessageField>,
}

impl Default for MessageDefinition {
    fn default() -> Self {
        Self {
            name: "new_message".to_string(),
            interval: 1000,
            enabled: true,
            packet_count: 0,
            network: None,
            fields: Vec::new(),
        }
    }
}

/// 运行时字段值（用于界面显示和编辑）
#[derive(Debug, Clone)]
pub struct FieldValue {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: String,
    /// 当前用户输入的值
    pub current_value: String,
    /// 是否可编辑（在界面显示）
    pub editable: bool,
    /// 解析后的字段类型（用于数据包生成）
    pub parsed_type: Option<FieldDataType>,
    /// 默认表达式（用于数据包生成）
    pub default_expr: Option<DefaultExpr>,
}

impl FieldValue {
    /// 从字段定义创建字段值
    pub fn from_field(field: &MessageField) -> Self {
        // 解析字段类型和默认表达式
        let (parsed_type, default_expr) =
            match FieldDataType::parse_type_and_default(
                &field.field_type,
            ) {
                Ok((data_type, expr)) => {
                    (Some(data_type), expr)
                }
                Err(e) => {
                    tracing::warn!("Failed to parse field type '{}': {}", field.field_type, e);
                    (None, None)
                }
            };

        // 确定当前值
        let current_value = if let Some(eq_pos) =
            field.field_type.find('=')
        {
            // 固定值字段：使用 = 后的值作为默认值
            field.field_type[eq_pos + 1..].to_string()
        } else if let Some(ref data_type) = parsed_type {
            // 动态值字段：根据解析的类型提供默认值
            Self::get_type_default_value_from_parsed(
                data_type,
            )
        } else {
            // 解析失败，使用传统方法
            Self::get_type_default_value(&field.field_type)
        };

        Self {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            current_value,
            editable: field.editable,
            parsed_type,
            default_expr,
        }
    }

    /// 根据数据类型获取默认值（传统方法）
    fn get_type_default_value(field_type: &str) -> String {
        match field_type {
            "i8" | "i16" | "i32" | "i64" => "0".to_string(),
            "u8" | "u16" | "u32" | "u64" => "0".to_string(),
            "f32" => "0.0".to_string(),
            "f64" => "0.0".to_string(),
            "bool" => "false".to_string(),
            "hex" => "0x00".to_string(),
            _ if field_type.starts_with("hex_") => {
                "0x00".to_string()
            }
            _ => String::new(),
        }
    }

    /// 根据解析的数据类型获取默认值
    fn get_type_default_value_from_parsed(
        data_type: &FieldDataType,
    ) -> String {
        match data_type {
            FieldDataType::I8
            | FieldDataType::I16
            | FieldDataType::I32
            | FieldDataType::I64 => "0".to_string(),
            FieldDataType::U8
            | FieldDataType::U16
            | FieldDataType::U32
            | FieldDataType::U64 => "0".to_string(),
            FieldDataType::F32 => "0.0".to_string(),
            FieldDataType::F64 => "0.0".to_string(),
            FieldDataType::Bool => "false".to_string(),
            FieldDataType::HexDynamic(_) => {
                "0x00".to_string()
            }
        }
    }

    /// 检查字段类型是否包含函数表达式
    pub fn has_function_expression(&self) -> bool {
        self.field_type.contains("rand(")
            || self.field_type.contains("loop(")
            || self.field_type.contains("switch(")
    }

    /// 检查字段是否应该允许用户输入（考虑函数表达式）
    pub fn should_allow_input(&self) -> bool {
        self.editable && !self.has_function_expression()
    }

    /// 生成字段的字节数据（用于数据包生成）
    pub fn to_bytes(
        &self,
        packet_index: usize,
        total_packets: Option<u64>,
    ) -> Result<Vec<u8>> {
        use crate::core::field_types::parse_field_value_by_type;

        if let Some(ref data_type) = self.parsed_type {
            let value = self.current_value.trim();

            // 如果用户输入了值且允许输入，优先使用用户输入
            if !value.is_empty()
                && self.should_allow_input()
            {
                return parse_field_value_by_type(
                    data_type, value,
                );
            }

            // 如果有默认表达式，使用表达式求值
            if let Some(ref expr) = self.default_expr {
                return expr.evaluate(
                    data_type,
                    packet_index,
                    total_packets,
                );
            }

            // 最后使用类型默认值
            Ok(data_type.default_value())
        } else {
            // 解析失败的情况下，返回空字节数组
            tracing::warn!("Field '{}' has no parsed type, returning empty bytes", self.name);
            Ok(Vec::new())
        }
    }
}

/// 运行时报文状态（用于界面显示）
/// 消息运行时状态
#[derive(Debug, Clone)]
pub struct MessageRuntimeState {
    /// 消息定义
    pub definition: MessageDefinition,
    /// 字段值列表
    pub field_values: Vec<FieldValue>,
    /// 是否正在发送
    pub is_sending: bool,
}

impl MessageRuntimeState {
    /// 从报文定义创建运行时状态
    pub fn from_definition(
        definition: MessageDefinition,
    ) -> Self {
        let field_values = definition
            .fields
            .iter()
            .map(FieldValue::from_field)
            .collect();

        Self {
            definition,
            field_values,
            is_sending: false,
        }
    }

    /// 获取可编辑字段（需要在界面显示的字段）
    pub fn get_editable_fields(&self) -> Vec<&FieldValue> {
        self.field_values
            .iter()
            .filter(|field| field.editable)
            .collect()
    }

    /// 获取可编辑字段的可变引用
    pub fn get_editable_fields_mut(
        &mut self,
    ) -> Vec<&mut FieldValue> {
        self.field_values
            .iter_mut()
            .filter(|field| field.editable)
            .collect()
    }

    /// 生成消息数据包
    pub fn generate_packet(
        &self,
        packet_index: usize,
        total_packets: Option<u64>,
    ) -> Result<Vec<u8>> {
        let mut packet_data = Vec::new();

        for field in &self.field_values {
            let field_bytes = field
                .to_bytes(packet_index, total_packets)?;
            packet_data.extend_from_slice(&field_bytes);
        }

        Ok(packet_data)
    }
}
