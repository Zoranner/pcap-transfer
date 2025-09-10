//! UI模块 - 用户界面相关功能

pub mod app;
pub mod app_state;
pub mod components;
pub mod config;
pub mod fonts;
pub mod widgets;

use crate::app::error::types::Result;

/// 启动 GUI 应用程序
pub fn run_gui() -> Result<()> {
    app::run_gui()
}
