// Windows GUI应用程序配置，隐藏控制台窗口
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod cli;
mod config;
mod display;
mod error;
mod gui;
mod network;
mod receiver;
mod sender;
mod stats;
mod timing;
mod utils;

use clap::Parser;
use cli::{Args, Commands};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 如果没有指定 --cli 且没有子命令，启动 GUI
    if !args.cli && args.command.is_none() {
        return gui::run_gui();
    }

    // 如果指定了 --cli 但没有子命令，显示帮助信息
    if args.cli && args.command.is_none() {
        eprintln!("错误: 使用命令行模式时必须指定子命令 (send 或 receive)");
        eprintln!("使用 --help 查看详细帮助信息");
        std::process::exit(1);
    }

    // 处理命令行模式的子命令
    match args.command.unwrap() {
        Commands::Send {
            dataset_path,
            address,
            port,
            network_type,
            interface,
        } => {
            sender::run_sender(
                dataset_path,
                address,
                port,
                network_type,
                interface,
            )
            .await?;
        }
        Commands::Receive {
            output_path,
            dataset_name,
            address,
            port,
            network_type,
            interface,
            max_packets,
        } => {
            receiver::run_receiver(
                output_path,
                dataset_name,
                address,
                port,
                network_type,
                interface,
                max_packets,
            )
            .await?;
        }
    }

    Ok(())
}
