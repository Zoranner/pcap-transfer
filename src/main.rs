//! Param Sender - 高性能数据包传输测试工具
//!
//! 基于 Rust 和 egui 的图形界面数据包传输测试工具

// Windows GUI应用程序配置，隐藏控制台窗口
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod app;
mod core;
mod ui;
mod utils;

use app::error::types::Result;
use app::logging::setup::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动 GUI
    ui::run_gui()
}
