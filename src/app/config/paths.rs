//! 跨平台配置路径管理模块
//! 负责处理不同操作系统下的配置文件路径

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// 配置路径管理器
pub struct ConfigPaths {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigPaths {
    /// 创建新的配置路径管理器
    ///
    /// # 参数
    /// * `project_name` - 项目名称，将作为子目录名
    ///
    /// # 返回
    /// 返回配置路径管理器实例
    ///
    /// # 示例
    /// ```
    /// use pcap_transfer::app::config::paths::ConfigPaths;
    /// let paths = ConfigPaths::new("pcap-transfer").unwrap();
    /// assert!(paths.config_file().to_string_lossy().contains("config.toml"));
    /// ```
    pub fn new(project_name: &str) -> Result<Self> {
        let config_dir =
            Self::get_config_directory(project_name)?;
        let config_file = config_dir.join("config.toml");

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    /// 获取配置文件路径
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// 确保配置目录存在
    pub fn ensure_config_dir_exists(&self) -> Result<()> {
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir)
                .with_context(|| {
                    format!(
                        "Failed to create config directory: {:?}",
                        self.config_dir
                    )
                })?;

            tracing::info!(
                "Created config directory: {:?}",
                self.config_dir
            );
        }
        Ok(())
    }

    /// 获取跨平台的配置目录路径
    ///
    /// # 平台特定行为
    /// * **Windows**: `%APPDATA%\KimoTech\[project_name]`
    /// * **macOS**: `~/Library/Application Support/KimoTech/[project_name]`
    /// * **Linux**: `~/.config/KimoTech/[project_name]`
    /// * **其他**: `~/.config/KimoTech/[project_name]` (与 Linux 相同)
    fn get_config_directory(
        project_name: &str,
    ) -> Result<PathBuf> {
        let base_dir = dirs::config_dir()
            .or_else(|| {
                dirs::home_dir()
                    .map(|home| home.join(".config"))
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Unable to determine user config directory")
            })?;

        // 构建完整的配置目录路径
        let config_dir = match std::env::consts::OS {
            "windows" => {
                // Windows: %APPDATA%\KimoTech\[project_name]
                base_dir.join("KimoTech").join(project_name)
            }
            "macos" => {
                // macOS: ~/Library/Application Support/KimoTech/[project_name]
                base_dir.join("KimoTech").join(project_name)
            }
            _ => {
                // Linux 和其他平台: ~/.config/KimoTech/[project_name]
                base_dir.join("KimoTech").join(project_name)
            }
        };

        tracing::info!(
            "Config directory path: {:?} (platform: {})",
            config_dir,
            std::env::consts::OS
        );

        Ok(config_dir)
    }
}
