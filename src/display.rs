use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

use crate::config::DisplayConfig;
use crate::utils::{format_bytes, format_rate};

/// 美化的命令行显示器
pub struct Display {
    config: DisplayConfig,
}

impl Display {
    pub fn new(config: DisplayConfig) -> Self {
        Self { config }
    }

    /// 打印欢迎信息
    pub fn print_welcome(&self) {
        if self.config.use_colors {
            println!(
                "{}",
                "Data Transfer - 数据包传输测试工具".bright_cyan().bold()
            );
            println!("{}", "=".repeat(60).bright_blue());
        } else {
            println!("Data Transfer - 数据包传输测试工具");
            println!("{}", "=".repeat(60));
        }
    }

    /// 打印成功信息
    pub fn print_success(&self, message: &str) {
        if self.config.use_colors {
            println!("\n[{}] {}", "SUCCESS".green().bold(), message.green());
        } else {
            println!("\n[SUCCESS] {message}");
        }
    }

    /// 打印信息
    pub fn print_info(&self, message: &str) {
        if self.config.use_colors {
            println!("\n[{}] {}", "INFO".blue().bold(), message);
        } else {
            println!("\n[INFO] {message}");
        }
    }

    /// 创建进度条
    pub fn create_progress_bar(&self, total: u64) -> Option<ProgressBar> {
        if !self.config.show_progress {
            return None;
        }

        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:30}] {pos:>7}/{len:7} ({percent:>3}%) {msg}")
                .unwrap()
                .progress_chars("██▌ "),
        );
        Some(pb)
    }

    /// 更新进度条
    pub fn update_progress(&self, pb: &Option<ProgressBar>, position: u64, message: &str) {
        if let Some(pb) = pb {
            pb.set_position(position);
            pb.set_message(message.to_string());
        }
    }

    /// 完成进度条
    pub fn finish_progress(&self, pb: &Option<ProgressBar>, message: &str) {
        if let Some(pb) = pb {
            pb.finish_with_message(message.to_string()); // 保留进度条并显示完成消息
            println!();
        }
    }

    /// 打印数据集信息
    pub fn print_dataset_info(
        &self,
        file_count: usize,
        total_packets: usize,
        total_size: u64,
        time_span: Option<f64>,
    ) {
        if self.config.use_colors {
            println!("\n{}", "数据集信息:".bright_cyan().bold());
        } else {
            println!("\n数据集信息:");
        }

        print!(
            "  文件数: {}, 数据包: {}, 大小: {}",
            file_count,
            total_packets,
            format_bytes(total_size)
        );

        if let Some(span) = time_span {
            println!(", 时长: {span:.3}s");
        } else {
            println!();
        }
    }

    /// 打印统计信息
    pub fn print_statistics(
        &self,
        title: &str,
        packets: usize,
        bytes: u64,
        errors: usize,
        duration: Duration,
        avg_packet_size: Option<u64>,
    ) {
        // 计算速率
        let rate_bps = if duration.as_secs_f64() > 0.0 {
            (bytes as f64 * 8.0) / duration.as_secs_f64()
        } else {
            0.0
        };

        if self.config.use_colors {
            println!("\n{}", title.bright_green().bold());
        } else {
            println!("\n{title}");
        }

        print!(
            "  数据包: {}, 字节数: {}, 错误: {}, 耗时: {:.2}s, 速率: {}",
            packets,
            format_bytes(bytes),
            errors,
            duration.as_secs_f64(),
            format_rate(rate_bps)
        );

        if let Some(avg_size) = avg_packet_size {
            println!(", 平均大小: {}", format_bytes(avg_size));
        } else {
            println!();
        }
    }

    /// 打印网络配置信息
    pub fn print_network_config(&self, mode: &str, address: &str, port: u16, network_type: &str) {
        if self.config.use_colors {
            println!("\n{}", "网络配置:".bright_cyan().bold());
        } else {
            println!("\n网络配置:");
        }
        println!("  模式: {mode}, 地址: {address}:{port}, 类型: {network_type}");
    }
}
