use std::net::IpAddr;

use crate::app::error::types::Result;
use crate::utils::helpers::{
    is_broadcast_address, is_multicast_address,
    validate_ip_address, validate_port,
};

/// 网络类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    /// 单播
    Unicast,
    /// 组播
    Multicast,
    /// 广播
    Broadcast,
}

/// 发送器应用程序配置
#[derive(Debug, Clone)]
pub struct SenderAppConfig {
    /// 网络配置
    pub network: NetworkConfig,
}

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// 目标IP地址
    pub address: IpAddr,
    /// 目标端口
    pub port: u16,
}

impl NetworkConfig {
    /// 创建发送器网络配置
    pub fn for_sender(
        address: String,
        port: u16,
        network_type: NetworkType,
        _interface: Option<String>,
    ) -> Result<Self> {
        let validated_port = validate_port(port)?;
        let validated_ip = validate_network_config(
            &address,
            &network_type,
        )?;

        Ok(Self {
            address: validated_ip,
            port: validated_port,
        })
    }
}

impl SenderAppConfig {
    /// 创建发送器配置
    pub fn new(
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) -> Result<Self> {
        let network = NetworkConfig::for_sender(
            address,
            port,
            network_type,
            interface,
        )?;

        Ok(Self { network })
    }
}

/// 验证网络配置（原网络模块函数的重构版本）
pub fn validate_network_config(
    address: &str,
    network_type: &NetworkType,
) -> Result<IpAddr> {
    let ip_addr = validate_ip_address(address)?;
    validate_network_config_by_ip(&ip_addr, network_type)?;
    Ok(ip_addr)
}

/// 按IP地址验证网络配置
fn validate_network_config_by_ip(
    ip_addr: &IpAddr,
    network_type: &NetworkType,
) -> Result<()> {
    match network_type {
        NetworkType::Broadcast => {
            if !is_broadcast_address(ip_addr) {
                tracing::warn!(
                    "Address {} may not be a valid broadcast address",
                    ip_addr
                );
            }
        }
        NetworkType::Multicast => {
            if !is_multicast_address(ip_addr) {
                return Err(crate::app::error::types::DataTransferError::validation(
                    "Address",
                    format!("Address {ip_addr} is not a valid multicast address"),
                ));
            }
        }
        NetworkType::Unicast => {
            if is_broadcast_address(ip_addr)
                || is_multicast_address(ip_addr)
            {
                tracing::warn!(
                    "Unicast mode using special address: {}",
                    ip_addr
                );
            }
        }
    }
    Ok(())
}
