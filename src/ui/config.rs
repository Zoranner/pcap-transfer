//! GUI配置模块
//!
//! 定义GUI应用程序的配置结构体和枚举类型。

use crate::app::config::message_types::MessageRuntimeState;
use crate::app::config::types::NetworkType;
use crate::ui::components::GlobalNetworkConfig;

/// 发送器配置
#[derive(Debug, Clone)]
pub struct SenderConfig {
    /// 目标地址
    pub address: String,
    /// 目标端口
    pub port: u16,
    /// 网络类型
    pub network_type: NetworkType,
    /// 网络接口
    pub interface: Option<String>,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
            network_type: NetworkType::Unicast,
            interface: None,
        }
    }
}

/// 消息配置（用于UI状态管理）
#[derive(Debug, Clone, Default)]
pub struct MessageUIConfig {
    /// 消息运行时状态列表
    pub messages: Vec<MessageRuntimeState>,
    /// 全局网络配置
    pub global_network: GlobalNetworkConfig,
}
