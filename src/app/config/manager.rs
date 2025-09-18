//! 配置管理模块
//! 负责加载、保存和管理应用程序配置

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use super::paths::ConfigPaths;
use super::types::NetworkType;

/// 应用程序配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub sender: SenderConfig,
    pub receiver: ReceiverConfig,
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
    pub csv_file: String, // CSV文件路径
    pub csv_send_interval: u64, // CSV发送周期（毫秒）
    pub network: NetworkConfig,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            dataset_path: "./dataset".to_string(),
            csv_file: String::new(), // CSV文件路径默认为空
            csv_send_interval: 1000, // 默认1秒发送周期
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

/// 配置管理器
pub struct ConfigManager {
    config_paths: ConfigPaths,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    ///
    /// # 参数
    /// * `project_name` - 项目名称，用于构建配置目录路径
    ///
    /// # 返回
    /// 返回配置管理器实例，如果路径创建失败则返回错误
    ///
    /// # 示例
    /// ```
    /// let config_manager = ConfigManager::new("pcap-transfer")?;
    /// ```
    pub fn new(project_name: &str) -> Result<Self> {
        let config_paths = ConfigPaths::new(project_name)?;

        Ok(Self {
            config_paths,
            config: AppConfig::default(),
        })
    }

    /// 加载配置文件
    pub fn load(&mut self) -> Result<()> {
        // 确保配置目录存在
        self.config_paths.ensure_config_dir_exists()?;

        let config_file = self.config_paths.config_file();

        if config_file.exists() {
            let content = fs::read_to_string(config_file)
                .with_context(|| {
                format!(
                    "Failed to read config file: {:?}",
                    config_file
                )
            })?;

            self.config = toml::from_str(&content)
                .with_context(|| {
                    format!(
                        "Failed to parse config file: {:?}",
                        config_file
                    )
                })?;

            tracing::info!(
                "Config file loaded successfully: {:?}",
                config_file
            );
        } else {
            tracing::info!(
                "Config file does not exist, using default config: {:?}",
                config_file
            );
            self.save()?; // 创建默认配置文件
        }
        Ok(())
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        // 确保配置目录存在
        self.config_paths.ensure_config_dir_exists()?;

        let content = toml::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;

        let config_file = self.config_paths.config_file();
        fs::write(config_file, content).with_context(
            || {
                format!(
                    "Failed to write config file: {:?}",
                    config_file
                )
            },
        )?;

        tracing::info!(
            "Config file saved successfully: {:?}",
            config_file
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

    /// 更新CSV配置
    pub fn update_csv_config(&mut self, csv_file: String, csv_send_interval: u64) {
        self.config.sender.csv_file = csv_file;
        self.config.sender.csv_send_interval = csv_send_interval;
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
