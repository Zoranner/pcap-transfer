use chrono::{DateTime, Utc};
use indicatif::ProgressBar;
use std::time::{Duration, Instant};

use crate::display::Display;
use crate::utils::format_bytes;

/// 传输统计信息
#[derive(Debug, Default)]
pub struct TransferStats {
    packets_processed: usize,
    bytes_processed: u64,
    errors: usize,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    progress_bar: Option<ProgressBar>,
    // 基于数据包时间戳的统计
    first_packet_timestamp: Option<DateTime<Utc>>,
    last_packet_timestamp: Option<DateTime<Utc>>,
}

impl TransferStats {
    /// 创建新的统计实例
    pub fn new(progress_bar: Option<ProgressBar>) -> Self {
        Self {
            start_time: Some(Instant::now()),
            progress_bar,
            ..Default::default()
        }
    }

    /// 更新统计信息（带时间戳）
    pub fn update_with_timestamp(
        &mut self,
        bytes: usize,
        timestamp: DateTime<Utc>,
    ) {
        self.packets_processed += 1;
        self.bytes_processed += bytes as u64;

        // 更新时间戳范围
        if self.first_packet_timestamp.is_none() {
            self.first_packet_timestamp = Some(timestamp);
        }
        self.last_packet_timestamp = Some(timestamp);
    }

    /// 增加错误计数
    pub fn add_error(&mut self) {
        self.errors += 1;
    }

    /// 标记传输完成
    pub fn finish(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(Instant::now());
        }
    }

    /// 获取运行时长
    pub fn get_duration(&self) -> Duration {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => {
                end.duration_since(start)
            }
            (Some(start), None) => start.elapsed(),
            _ => Duration::default(),
        }
    }

    /// 获取基于数据包时间戳的持续时间
    pub fn get_packet_duration(&self) -> Option<Duration> {
        match (
            self.first_packet_timestamp,
            self.last_packet_timestamp,
        ) {
            (Some(first), Some(last)) => {
                let duration_chrono =
                    last.signed_duration_since(first);
                duration_chrono.to_std().ok()
            }
            _ => None,
        }
    }

    /// 计算数据速率（bps）- 基于数据包时间戳
    pub fn get_packet_rate_bps(&self) -> Option<f64> {
        if let Some(duration) = self.get_packet_duration() {
            let duration_secs = duration.as_secs_f64();
            if duration_secs > 0.0 {
                Some(
                    (self.bytes_processed as f64 * 8.0)
                        / duration_secs,
                )
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }

    /// 计算平均包大小
    pub fn get_average_packet_size(&self) -> Option<u64> {
        if self.packets_processed > 0 {
            Some(
                self.bytes_processed
                    / self.packets_processed as u64,
            )
        } else {
            None
        }
    }

    /// 更新进度条（发送模式）
    pub fn update_progress_sender(
        &mut self,
        display: &Display,
    ) {
        // 每100个数据包或达到100%时更新进度条
        if self.packets_processed % 100 == 0
            || self.packets_processed == 1
        {
            let rate_bps =
                self.get_packet_rate_bps().unwrap_or(0.0);
            let message = format!(
                "{:.1} Mbps, {}",
                rate_bps / 1_000_000.0,
                format_bytes(self.bytes_processed)
            );
            display.update_progress(
                &self.progress_bar,
                self.packets_processed as u64,
                &message,
            );
        }
    }

    /// 更新进度条（接收模式）
    pub fn update_progress_receiver(
        &mut self,
        _display: &Display,
    ) {
        if let Some(pb) = &self.progress_bar {
            let rate_bps =
                self.get_packet_rate_bps().unwrap_or(0.0);
            let message = format!(
                "{:.1} Mbps, {}",
                rate_bps / 1_000_000.0,
                format_bytes(self.bytes_processed)
            );
            pb.set_position(self.packets_processed as u64);
            pb.set_message(message);
        }
    }

    /// 完成进度条
    pub fn finish_progress(&self, display: &Display) {
        let final_message = format!(
            "{:.1} Mbps, {}",
            self.get_packet_rate_bps().unwrap_or(0.0)
                / 1_000_000.0,
            format_bytes(self.bytes_processed)
        );
        display.update_progress(
            &self.progress_bar,
            self.packets_processed as u64,
            &final_message,
        );
        display.finish_progress(&self.progress_bar, "完成");
    }

    /// 打印最终统计信息
    pub fn print_summary(
        &self,
        display: &Display,
        title: &str,
    ) {
        let runtime_duration = self.get_duration();
        let packet_duration = self.get_packet_duration();
        let avg_packet_size =
            self.get_average_packet_size();

        display.print_statistics_with_packet_duration(
            title,
            self.packets_processed,
            self.bytes_processed,
            self.errors,
            runtime_duration,
            packet_duration,
            avg_packet_size,
        );
    }

    /// 获取包数量
    pub fn packets_count(&self) -> usize {
        self.packets_processed
    }

    /// 获取已处理的包数量（GUI 用）
    pub fn get_packets_processed(&self) -> usize {
        self.packets_processed
    }

    /// 获取已处理的字节数（GUI 用）
    pub fn get_bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// 获取错误数量（GUI 用）
    pub fn get_errors(&self) -> usize {
        self.errors
    }
}
