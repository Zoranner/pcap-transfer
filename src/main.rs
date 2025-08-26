// Windows GUI应用程序配置，隐藏控制台窗口
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod config;
mod error;
mod gui;
mod network;
mod receiver;
mod sender;
mod stats;
mod timing;
mod utils;

use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 直接启动 GUI
    gui::run_gui()
}
