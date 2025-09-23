//! 配置管理模块
//! 负责加载、保存和管理应用程序配置

use serde::{Deserialize, Serialize};
use std::fs;
use tracing;

use crate::app::error::types::{DataTransferError, Result};

use super::paths::ConfigPaths;
use super::types::NetworkType;

/// 应用程序配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// 发送器配置
    pub sender: SenderConfig,
    /// 消息定义列表
    pub messages:
        Vec<super::message_types::MessageDefinition>,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 目标地址
    pub address: String,
    /// 目标端口
    pub port: u16,
    /// 网络类型
    pub network_type: String,
    /// 网络接口
    pub interface: String,
}

/// 发送器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderConfig {
    /// 发送策略：sequential 或 parallel
    pub strategy: String,
    /// 网络配置
    pub network: NetworkConfig,
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

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            strategy: "sequential".to_string(),
            network: NetworkConfig::default(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_paths: ConfigPaths,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let config_paths = ConfigPaths::new()?;
        let config = AppConfig::default();

        Ok(Self {
            config_paths,
            config,
        })
    }

    /// 加载配置文件
    pub fn load(&mut self) -> Result<()> {
        let config_file = self.config_paths.config_file();

        if config_file.exists() {
            let content = fs::read_to_string(config_file)
                .map_err(|e| {
                DataTransferError::config(format!(
                    "Failed to read config file {:?}: {}",
                    config_file, e
                ))
            })?;

            // 添加调试输出
            tracing::info!(
                "Config file content:\n{}",
                content
            );

            match toml::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    self.config = config;
                    tracing::info!("Successfully parsed config with {} messages", self.config.messages.len());
                }
                Err(e) => {
                    tracing::error!(
                        "TOML parsing error: {}",
                        e
                    );
                    tracing::warn!("Config file parse failed, using default config");
                    // 解析失败时使用默认配置，但不返回错误
                    self.config = AppConfig::default();
                }
            }

            tracing::info!(
                "Config file loaded successfully: {:?}",
                config_file
            );
            tracing::info!(
                "Loaded {} messages from config",
                self.config.messages.len()
            );
        } else {
            tracing::info!(
                "Config file does not exist, creating default config: {:?}",
                config_file
            );
            // 使用默认配置并保存
            self.config = AppConfig::default();
            if let Err(e) = self.save() {
                tracing::warn!("Failed to save default config file: {}", e);
                // 保存失败不应该阻止程序运行
            }
        }
        Ok(())
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| {
                DataTransferError::config(format!(
                    "Failed to serialize config: {}",
                    e
                ))
            })?;

        let config_file = self.config_paths.config_file();
        fs::write(config_file, &content).map_err(|e| {
            DataTransferError::config(format!(
                "Failed to write config file {:?}: {}",
                config_file, e
            ))
        })?;

        tracing::info!(
            "Config file saved successfully: {:?}",
            config_file
        );
        Ok(())
    }

    /// 只保存网络配置，不影响报文配置
    pub fn save_network_only(&self) -> Result<()> {
        let config_file = self.config_paths.config_file();
        
        // 读取现有的配置文件内容
        let mut file_config = if config_file.exists() {
            let content = fs::read_to_string(config_file)
                .map_err(|e| {
                    DataTransferError::config(format!(
                        "Failed to read existing config file {:?}: {}",
                        config_file, e
                    ))
                })?;
            
            // 解析现有配置
            toml::from_str::<AppConfig>(&content)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to parse existing config, using current messages: {}", e);
                    // 如果解析失败，使用当前的消息配置作为备份
                    AppConfig {
                        sender: SenderConfig::default(),
                        messages: self.config.messages.clone(),
                    }
                })
        } else {
            // 如果文件不存在，使用当前的消息配置
            AppConfig {
                sender: SenderConfig::default(),
                messages: self.config.messages.clone(),
            }
        };
        
        // 只更新网络配置部分，保持报文配置不变
        file_config.sender = self.config.sender.clone();
        
        // 序列化并保存
        let content = toml::to_string_pretty(&file_config)
            .map_err(|e| {
                DataTransferError::config(format!(
                    "Failed to serialize network config: {}",
                    e
                ))
            })?;
        
        fs::write(config_file, &content).map_err(|e| {
            DataTransferError::config(format!(
                "Failed to write network config to file {:?}: {}",
                config_file, e
            ))
        })?;
        
        tracing::info!(
            "Network configuration saved successfully: {:?}",
            config_file
        );
        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
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

    /// 获取消息配置列表
    pub fn get_messages(
        &self,
    ) -> &Vec<super::message_types::MessageDefinition> {
        &self.config.messages
    }

    /// 更新消息配置列表
    pub fn update_messages(
        &mut self,
        messages: Vec<
            super::message_types::MessageDefinition,
        >,
    ) {
        self.config.messages = messages;
    }

    /// 更新发送器配置（统一接口）
    pub fn update_sender_config(
        &mut self,
        config: &crate::ui::config::SenderConfig,
    ) {
        // 更新网络配置
        self.update_sender_network_config(
            config.address.clone(),
            config.port,
            config.network_type,
            config.interface.clone(),
        );
    }

    /// 更新发送器网络配置（内部方法）
    fn update_sender_network_config(
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
                NetworkType::Multicast => {
                    "multicast".to_string()
                }
                NetworkType::Broadcast => {
                    "broadcast".to_string()
                }
            };

        self.config.sender.network.interface =
            interface.unwrap_or_default();
    }
}
