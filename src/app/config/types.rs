use std::net::IpAddr;
use std::path::PathBuf;

use crate::app::error::types::Result;
use crate::utils::helpers::{
    ensure_output_directory, is_broadcast_address,
    is_multicast_address, validate_dataset_path,
    validate_ip_address, validate_port,
};

/// 网络类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    /// 单播
    Unicast,
    /// 广播
    Broadcast,
    /// 组播
    Multicast,
}

/// 发送器应用程序配置
#[derive(Debug, Clone)]
pub struct SenderAppConfig {
    pub network: NetworkConfig,
    pub dataset_path: PathBuf,
}

/// 接收器应用程序配置
#[derive(Debug, Clone)]
pub struct ReceiverAppConfig {
    pub network: NetworkConfig,
    pub output_path: PathBuf,
    pub dataset_name: String,
    pub buffer_size: usize,
}

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub address: IpAddr,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
}

// Removed unused DisplayConfig struct and its impl

impl NetworkConfig {
    /// 创建发送器网络配置
    pub fn for_sender(
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) -> Result<Self> {
        let validated_port = validate_port(port)?;
        let validated_ip = validate_network_config(
            &address,
            &network_type,
        )?;

        Ok(Self {
            address: validated_ip,
            port: validated_port,
            network_type,
            interface,
        })
    }

    /// 创建接收器网络配置
    pub fn for_receiver(
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) -> Result<Self> {
        let validated_port = validate_port(port)?;
        let validated_ip = validate_network_config(
            &address,
            &network_type,
        )?;

        Ok(Self {
            address: validated_ip,
            port: validated_port,
            network_type,
            interface,
        })
    }

    /// 检查配置是否有效
    pub fn validate(&self) -> Result<()> {
        validate_port(self.port)?;
        validate_network_config_by_ip(
            &self.address,
            &self.network_type,
        )?;
        Ok(())
    }
}

impl SenderAppConfig {
    /// 创建发送器配置
    pub fn new(
        dataset_path: PathBuf,
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
        validate_dataset_path(&dataset_path)?;

        Ok(Self {
            network,
            dataset_path,
        })
    }

    /// 验证整个配置
    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
        validate_dataset_path(&self.dataset_path)?;
        Ok(())
    }
}

impl ReceiverAppConfig {
    /// 创建接收器配置
    pub fn new(
        output_path: PathBuf,
        dataset_name: String,
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) -> Result<Self> {
        let network = NetworkConfig::for_receiver(
            address,
            port,
            network_type,
            interface,
        )?;
        ensure_output_directory(&output_path)?;
        let buffer_size = 1024 * 1024; // 默认 1MB 缓冲区

        Ok(Self {
            network,
            output_path,
            dataset_name,
            buffer_size,
        })
    }

    /// 验证整个配置
    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
        ensure_output_directory(&self.output_path)?;
        Ok(())
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
                    "地址{}可能不是有效的广播地址",
                    ip_addr
                );
            }
        }
        NetworkType::Multicast => {
            if !is_multicast_address(ip_addr) {
                return Err(crate::app::error::types::DataTransferError::validation(
                    "地址",
                    format!("地址{ip_addr}不是有效的组播地址"),
                ));
            }
        }
        NetworkType::Unicast => {
            if is_broadcast_address(ip_addr)
                || is_multicast_address(ip_addr)
            {
                tracing::warn!(
                    "单播模式使用了特殊地址: {}",
                    ip_addr
                );
            }
        }
    }
    Ok(())
}
