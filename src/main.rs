use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod network;
mod receiver;
mod sender;
mod utils;

use cli::{Args, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("设置默认订阅者失败");

    // 解析命令行参数
    let args = Args::parse();

    info!("启动data-transfer工具");

    // 根据子命令执行相应操作
    match args.command {
        Commands::Send {
            dataset_path,
            address,
            port,
            network_type,
            interface,
        } => {
            info!(
                "发送模式: 数据集={}, 地址={}:{}, 类型={:?}",
                dataset_path.display(),
                address,
                port,
                network_type
            );

            sender::run_sender(dataset_path, address, port, network_type, interface).await?;
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
            info!(
                "接收模式: 输出={}/{}, 地址={}:{}, 类型={:?}, 最大包数={:?}",
                output_path.display(),
                dataset_name,
                address,
                port,
                network_type,
                max_packets
            );

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

    info!("data-transfer工具执行完成");
    Ok(())
}
