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
