//! GUI模块 - 图形用户界面相关功能

pub mod app;
pub mod config;
pub mod font;
pub mod renderer;

// 移除未使用的导入

use crate::error::Result;

/// 启动 GUI 应用程序
pub fn run_gui() -> Result<()> {
    app::run_gui()
}
