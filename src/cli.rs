use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// 数据包传输测试工具
#[derive(Parser, Debug)]
#[command(name = "data-transfer")]
#[command(about = "基于pcapfile-io的高性能数据包传输测试工具")]
#[command(version = "0.1.0")]
pub struct Args {
    /// 强制使用命令行模式（默认使用 GUI）
    #[arg(long, help = "强制使用命令行界面，默认启动图形界面")]
    pub cli: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 发送pcap数据集
    Send {
        /// pcap数据集路径
        #[arg(short, long, value_name = "PATH")]
        dataset_path: PathBuf,

        /// 目标地址
        #[arg(short, long, value_name = "IP")]
        address: String,

        /// 目标端口
        #[arg(short, long, value_name = "PORT")]
        port: u16,

        /// 网络类型
        #[arg(short, long, value_enum, default_value = "unicast")]
        network_type: NetworkType,

        /// 网络接口（可选，对于多播/广播）
        #[arg(short, long, value_name = "INTERFACE")]
        interface: Option<String>,
    },

    /// 接收数据包并保存为pcap数据集
    Receive {
        /// 输出目录路径
        #[arg(short, long, value_name = "PATH")]
        output_path: PathBuf,

        /// 数据集名称
        #[arg(short, long, value_name = "NAME")]
        dataset_name: String,

        /// 监听地址
        #[arg(short, long, value_name = "IP")]
        address: String,

        /// 监听端口
        #[arg(short, long, value_name = "PORT")]
        port: u16,

        /// 网络类型
        #[arg(short, long, value_enum, default_value = "unicast")]
        network_type: NetworkType,

        /// 网络接口（可选，对于多播/广播）
        #[arg(short, long, value_name = "INTERFACE")]
        interface: Option<String>,

        /// 最大接收包数（可选，0表示无限制）
        #[arg(short, long, value_name = "COUNT")]
        max_packets: Option<usize>,
    },
}

/// 网络传输类型
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum NetworkType {
    /// 单播
    Unicast,
    /// 广播
    Broadcast,
    /// 组播
    Multicast,
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkType::Unicast => write!(f, "单播"),
            NetworkType::Broadcast => write!(f, "广播"),
            NetworkType::Multicast => write!(f, "组播"),
        }
    }
}
