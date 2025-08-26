use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

// Removed display module dependency
// Removed indicatif and format_bytes dependencies

/// 传输统计信息
#[derive(Debug, Default)]
pub struct TransferStats {
    packets_processed: usize,
    bytes_processed: u64,
    errors: usize,
    end_time: Option<Instant>,
    // 基于数据包时间戳的统计
    first_packet_timestamp: Option<DateTime<Utc>>,
    last_packet_timestamp: Option<DateTime<Utc>>,
}

impl TransferStats {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self::default()
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

    // Removed unused get_duration method

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

    // Removed unused get_average_packet_size method

    // Removed update_progress_sender method (display dependency removed)

    // Removed update_progress_receiver method (display dependency removed)

    // Removed finish_progress method (display dependency removed)

    // Removed print_summary method (display dependency removed)

    // Removed unused packets_count method

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
