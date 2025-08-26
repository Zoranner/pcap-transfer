use std::net::IpAddr;
use std::path::PathBuf;

use crate::cli::NetworkType;
use crate::error::Result;
use crate::utils::{
    ensure_output_directory, is_broadcast_address,
    is_multicast_address, validate_dataset_path,
    validate_ip_address, validate_port,
};

/// 应用程序配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub operation: OperationConfig,
    #[allow(dead_code)]
    pub display: DisplayConfig,
}

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub address: IpAddr,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
}

/// 操作配置（发送或接收）
#[derive(Debug, Clone)]
pub enum OperationConfig {
    Send {
        #[allow(dead_code)]
        dataset_path: PathBuf,
        timing_enabled: bool,
        max_delay_threshold_ms: u64,
    },
    Receive {
        #[allow(dead_code)]
        output_path: PathBuf,
        #[allow(dead_code)]
        dataset_name: String,
        max_packets: Option<usize>,
        buffer_size: usize,
    },
}

/// 显示配置
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub show_progress: bool,
    pub use_colors: bool,
    #[allow(dead_code)]
    pub update_interval_packets: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_progress: true,
            use_colors: true,
            update_interval_packets: 100,
        }
    }
}

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

    /// 获取网络类型的中文描述
    pub fn get_type_description(&self) -> &'static str {
        match self.network_type {
            NetworkType::Unicast => "单播",
            NetworkType::Broadcast => "广播",
            NetworkType::Multicast => "组播",
        }
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

impl OperationConfig {
    /// 创建发送配置
    pub fn for_sender(
        dataset_path: PathBuf,
    ) -> Result<Self> {
        validate_dataset_path(&dataset_path)?;
        Ok(Self::Send {
            dataset_path,
            timing_enabled: true,
            max_delay_threshold_ms: 100,
        })
    }

    /// 创建接收配置
    pub fn for_receiver(
        output_path: PathBuf,
        dataset_name: String,
        max_packets: Option<usize>,
    ) -> Result<Self> {
        ensure_output_directory(&output_path)?;
        Ok(Self::Receive {
            output_path,
            dataset_name,
            max_packets,
            buffer_size: 65536, // 64KB 默认缓冲区
        })
    }
}

impl AppConfig {
    /// 创建发送器配置
    pub fn for_sender(
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
        let operation =
            OperationConfig::for_sender(dataset_path)?;
        let display = DisplayConfig::default();

        Ok(Self {
            network,
            operation,
            display,
        })
    }

    /// 创建接收器配置
    pub fn for_receiver(
        output_path: PathBuf,
        dataset_name: String,
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
        max_packets: Option<usize>,
    ) -> Result<Self> {
        let network = NetworkConfig::for_receiver(
            address,
            port,
            network_type,
            interface,
        )?;
        let operation = OperationConfig::for_receiver(
            output_path,
            dataset_name,
            max_packets,
        )?;
        let display = DisplayConfig::default();

        Ok(Self {
            network,
            operation,
            display,
        })
    }

    /// 验证整个配置
    pub fn validate(&self) -> Result<()> {
        self.network.validate()?;
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
                return Err(crate::error::DataTransferError::validation(
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
