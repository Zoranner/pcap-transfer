// Windows GUI应用程序配置，隐藏控制台窗口
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod config;
mod config_manager;
mod error;
mod gui;
mod network;
mod receiver;
mod sender;
mod stats;
mod timing;
mod utils;

use error::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logging();

    // 启动 GUI
    gui::run_gui()
}

/// 初始化日志系统
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // 默认日志级别：debug模式下显示debug信息，release模式下显示info信息
            if cfg!(debug_assertions) {
                EnvFilter::new("pcap_transfer=debug,warn")
            } else {
                EnvFilter::new("pcap_transfer=info,warn")
            }
        });

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true),
        )
        .with(filter)
        .init();
}
