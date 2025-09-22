//! 配置路径管理模块
//! 负责处理程序根目录下的配置文件路径

use crate::app::error::types::{DataTransferError, Result};
use std::path::{Path, PathBuf};

/// 配置路径管理器
pub struct ConfigPaths {
    config_file: PathBuf,
}

impl ConfigPaths {
    /// 创建新的配置路径管理器
    ///
    /// # 返回
    /// 返回配置路径管理器实例
    ///
    /// # 示例
    /// ```
    /// use param_sender::app::config::paths::ConfigPaths;
    /// let paths = ConfigPaths::new().unwrap();
    /// assert!(paths.config_file().to_string_lossy().contains("config.toml"));
    /// ```
    pub fn new() -> Result<Self> {
        let config_file = Self::get_root_config_file()?;

        Ok(Self {
            config_file,
        })
    }

    /// 获取配置文件路径
    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    /// 获取程序根目录的配置文件路径
    ///
    /// # 返回
    /// 返回程序根目录下的 config.toml 文件路径
    fn get_root_config_file() -> Result<PathBuf> {
        // 首先尝试从当前工作目录查找
        let current_dir = std::env::current_dir()
            .map_err(|e| DataTransferError::config(
                format!("Failed to get current directory: {}", e)
            ))?;
        
        let config_file = current_dir.join("config.toml");
        
        // 如果工作目录没有，尝试从可执行文件目录查找
        if !config_file.exists() {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let exe_config_file = exe_dir.join("config.toml");
                    if exe_config_file.exists() {
                        tracing::info!("Found config file in executable directory: {:?}", exe_config_file);
                        return Ok(exe_config_file);
                    }
                }
            }
        }
        
        tracing::info!("Using config file path: {:?}", config_file);
        Ok(config_file)
    }
}
