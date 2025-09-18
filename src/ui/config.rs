//! GUI配置模块
//!
//! 定义GUI应用程序的配置结构体和枚举类型。

use crate::app::config::types::{DataFormat, NetworkType};

/// 当前选中的标签页
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedTab {
    Sender,
    Receiver,
}

impl Default for SelectedTab {
    fn default() -> Self {
        Self::Sender
    }
}

/// 发送器配置
#[derive(Debug, Clone)]
pub struct SenderConfig {
    pub data_format: DataFormat,
    pub pcap_path: String, // PCAP数据集路径（文件夹）
    pub csv_file: String,  // CSV文件路径（文件）
    pub csv_packet_interval: u64, // CSV发送周期（毫秒）
    pub address: String,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            data_format: DataFormat::Pcap,
            pcap_path: String::new(),
            csv_file: String::new(),
            csv_packet_interval: 1000, // 默认1秒（1000毫秒）
            address: "127.0.0.1".to_string(),
            port: 8080,
            network_type: NetworkType::Unicast,
            interface: None,
        }
    }
}

/// 接收器配置
#[derive(Debug, Clone)]
pub struct ReceiverConfig {
    pub output_path: String, // 改为String以匹配配置管理器
    pub dataset_name: String,
    pub address: String,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
}

impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            output_path: "./output".to_string(),
            dataset_name: "received_data".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080, // 修改为8080，与发送器端口匹配
            network_type: NetworkType::Unicast,
            interface: None,
        }
    }
}
