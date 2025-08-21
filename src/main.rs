mod cli;
mod config;
mod display;
mod error;
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

    // 根据子命令执行相应操作
    match args.command {
        Commands::Send {
            dataset_path,
            address,
            port,
            network_type,
            interface,
        } => {
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
