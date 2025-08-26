//! 配置管理模块
//! 负责加载、保存和管理应用程序配置

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::NetworkType;

/// 应用程序配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub sender: SenderConfig,
    pub receiver: ReceiverConfig,
    pub logging: LoggingConfig,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub address: String,
    pub port: u16,
    pub network_type: String, // 在TOML中存储为字符串
    pub interface: String,
}

/// 发送器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderConfig {
    pub dataset_path: String,
    pub network: NetworkConfig,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            dataset_path: "./dataset".to_string(),
            network: NetworkConfig::default(),
        }
    }
}

/// 接收器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiverConfig {
    pub output_path: String,
    pub dataset_name: String,
    pub buffer_size: usize,
    pub network: NetworkConfig,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
            network_type: "unicast".to_string(),
            interface: String::new(),
        }
    }
}

impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            output_path: "./output".to_string(),
            dataset_name: "received_data".to_string(),
            buffer_size: 1048576,
            network: NetworkConfig::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_enabled: false,
            file_path: "./logs/app.log".to_string(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            config: AppConfig::default(),
        }
    }

    /// 加载配置文件
    pub fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content =
                fs::read_to_string(&self.config_path)
                    .with_context(|| {
                        format!(
                            "无法读取配置文件: {:?}",
                            self.config_path
                        )
                    })?;

            self.config = toml::from_str(&content)
                .with_context(|| {
                    format!(
                        "无法解析配置文件: {:?}",
                        self.config_path
                    )
                })?;

            tracing::info!(
                "配置文件加载成功: {:?}",
                self.config_path
            );
        } else {
            tracing::info!(
                "配置文件不存在，使用默认配置: {:?}",
                self.config_path
            );
            self.save()?; // 创建默认配置文件
        }
        Ok(())
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).with_context(
                || {
                    format!(
                        "无法创建配置目录: {:?}",
                        parent
                    )
                },
            )?;
        }

        let content = toml::to_string_pretty(&self.config)
            .context("无法序列化配置")?;

        fs::write(&self.config_path, content)
            .with_context(|| {
                format!(
                    "无法写入配置文件: {:?}",
                    self.config_path
                )
            })?;

        tracing::info!(
            "配置文件保存成功: {:?}",
            self.config_path
        );
        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 更新发送器网络配置
    pub fn update_sender_network_config(
        &mut self,
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) {
        self.config.sender.network.address = address;
        self.config.sender.network.port = port;
        self.config.sender.network.network_type =
            match network_type {
                NetworkType::Unicast => {
                    "unicast".to_string()
                }
                NetworkType::Broadcast => {
                    "broadcast".to_string()
                }
                NetworkType::Multicast => {
                    "multicast".to_string()
                }
            };
        self.config.sender.network.interface =
            interface.unwrap_or_default();
    }

    /// 更新接收器网络配置
    pub fn update_receiver_network_config(
        &mut self,
        address: String,
        port: u16,
        network_type: NetworkType,
        interface: Option<String>,
    ) {
        self.config.receiver.network.address = address;
        self.config.receiver.network.port = port;
        self.config.receiver.network.network_type =
            match network_type {
                NetworkType::Unicast => {
                    "unicast".to_string()
                }
                NetworkType::Broadcast => {
                    "broadcast".to_string()
                }
                NetworkType::Multicast => {
                    "multicast".to_string()
                }
            };
        self.config.receiver.network.interface =
            interface.unwrap_or_default();
    }

    /// 获取发送器网络类型
    pub fn get_sender_network_type(&self) -> NetworkType {
        match self
            .config
            .sender
            .network
            .network_type
            .as_str()
        {
            "broadcast" => NetworkType::Broadcast,
            "multicast" => NetworkType::Multicast,
            _ => NetworkType::Unicast,
        }
    }

    /// 获取接收器网络类型
    pub fn get_receiver_network_type(&self) -> NetworkType {
        match self
            .config
            .receiver
            .network
            .network_type
            .as_str()
        {
            "broadcast" => NetworkType::Broadcast,
            "multicast" => NetworkType::Multicast,
            _ => NetworkType::Unicast,
        }
    }

    /// 更新发送器配置
    pub fn update_sender_config(
        &mut self,
        dataset_path: String,
    ) {
        self.config.sender.dataset_path = dataset_path;
    }

    /// 更新接收器配置
    pub fn update_receiver_config(
        &mut self,
        output_path: String,
        dataset_name: String,
    ) {
        self.config.receiver.output_path = output_path;
        self.config.receiver.dataset_name = dataset_name;
    }
}
