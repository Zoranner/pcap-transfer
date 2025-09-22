//! 按消息类型分开的统计系统

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 单个消息类型的统计信息
#[derive(Debug, Clone)]
pub struct MessageStats {
    /// 已发送的包数
    pub packets_sent: u64,
    /// 已发送的字节数
    pub bytes_sent: u64,
    /// 错误数量
    pub errors: u64,
    /// 开始时间
    pub start_time: Option<Instant>,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 最后发送时间
    pub last_sent_time: Option<Instant>,
    /// 第一个数据包时间戳
    pub first_packet_timestamp: Option<DateTime<Utc>>,
    /// 最后一个数据包时间戳
    pub last_packet_timestamp: Option<DateTime<Utc>>,
}

impl MessageStats {
    /// 创建新的消息统计
    pub fn new(_message_name: String) -> Self {
        Self {
            packets_sent: 0,
            bytes_sent: 0,
            errors: 0,
            start_time: Some(Instant::now()),
            end_time: None,
            last_sent_time: None,
            first_packet_timestamp: None,
            last_packet_timestamp: None,
        }
    }

    /// 记录成功发送的数据包
    pub fn record_sent_packet(
        &mut self,
        bytes: usize,
        timestamp: DateTime<Utc>,
    ) {
        self.packets_sent += 1;
        self.bytes_sent += bytes as u64;
        self.last_sent_time = Some(Instant::now());

        // 更新时间戳范围
        if self.first_packet_timestamp.is_none() {
            self.first_packet_timestamp = Some(timestamp);
        }
        self.last_packet_timestamp = Some(timestamp);
    }

    /// 记录发送错误
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// 结束统计（停止发送时调用）
    pub fn finish(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(Instant::now());
        }
    }

    /// 获取发送速率（包/秒）
    pub fn get_packet_rate(&self) -> Option<f64> {
        if let (Some(start), Some(last)) =
            (self.start_time, self.last_sent_time)
        {
            let duration = last.duration_since(start);
            if duration.as_secs() > 0
                && self.packets_sent > 0
            {
                Some(
                    self.packets_sent as f64
                        / duration.as_secs_f64(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 获取字节传输速率（字节/秒）
    pub fn get_bytes_rate(&self) -> Option<f64> {
        if let (Some(start), Some(last)) =
            (self.start_time, self.last_sent_time)
        {
            let duration = last.duration_since(start);
            if duration.as_secs() > 0 && self.bytes_sent > 0
            {
                Some(
                    self.bytes_sent as f64
                        / duration.as_secs_f64(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 获取运行时长
    pub fn get_duration(&self) -> Option<Duration> {
        if let Some(start) = self.start_time {
            if let Some(end) = self.end_time {
                // 如果已结束，使用结束时间
                Some(end.duration_since(start))
            } else {
                // 如果还在运行，使用当前时间
                Some(start.elapsed())
            }
        } else {
            None
        }
    }
}

/// 全局消息统计管理器
#[derive(Debug, Clone)]
pub struct MessageStatsManager {
    message_stats: HashMap<String, MessageStats>,
    global_start_time: Option<Instant>,
}

impl Default for MessageStatsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageStatsManager {
    /// 创建新的统计管理器
    pub fn new() -> Self {
        Self {
            message_stats: HashMap::new(),
            global_start_time: Some(Instant::now()),
        }
    }

    /// 确保消息统计存在
    fn ensure_message_stats(&mut self, message_name: &str) {
        if !self.message_stats.contains_key(message_name) {
            self.message_stats.insert(
                message_name.to_string(),
                MessageStats::new(message_name.to_string()),
            );
        }
    }

    /// 记录消息发送成功
    pub fn record_message_sent(
        &mut self,
        message_name: &str,
        bytes: usize,
        timestamp: DateTime<Utc>,
    ) {
        self.ensure_message_stats(message_name);
        if let Some(stats) =
            self.message_stats.get_mut(message_name)
        {
            stats.record_sent_packet(bytes, timestamp);
        }
    }

    /// 记录消息发送错误
    pub fn record_message_error(
        &mut self,
        message_name: &str,
    ) {
        self.ensure_message_stats(message_name);
        if let Some(stats) =
            self.message_stats.get_mut(message_name)
        {
            stats.record_error();
        }
    }

    /// 结束所有消息的统计
    pub fn finish_all(&mut self) {
        for stats in self.message_stats.values_mut() {
            stats.finish();
        }
    }

    /// 获取所有消息的统计信息
    pub fn get_all_message_stats(
        &self,
    ) -> &HashMap<String, MessageStats> {
        &self.message_stats
    }

    /// 获取全局统计汇总
    pub fn get_global_summary(&self) -> GlobalStatsSummary {
        let total_packets: u64 = self
            .message_stats
            .values()
            .map(|s| s.packets_sent)
            .sum();
        let total_bytes: u64 = self
            .message_stats
            .values()
            .map(|s| s.bytes_sent)
            .sum();
        let total_errors: u64 = self
            .message_stats
            .values()
            .map(|s| s.errors)
            .sum();
        let message_count = self.message_stats.len();

        let global_duration = self
            .global_start_time
            .map(|start| start.elapsed());

        let global_packet_rate = if let Some(duration) =
            global_duration
        {
            if duration.as_secs() > 0 && total_packets > 0 {
                Some(
                    total_packets as f64
                        / duration.as_secs_f64(),
                )
            } else {
                None
            }
        } else {
            None
        };

        let _global_bytes_rate = if let Some(duration) =
            global_duration
        {
            if duration.as_secs() > 0 && total_bytes > 0 {
                Some(
                    total_bytes as f64
                        / duration.as_secs_f64(),
                )
            } else {
                None
            }
        } else {
            None
        };

        GlobalStatsSummary {
            total_packets,
            total_bytes,
            total_errors,
            message_count,
            global_packet_rate,
        }
    }

    /// 重置所有统计信息
    pub fn reset(&mut self) {
        self.message_stats.clear();
        self.global_start_time = Some(Instant::now());
    }
}

/// 全局统计汇总
#[derive(Debug, Clone)]
pub struct GlobalStatsSummary {
    /// 总包数
    pub total_packets: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 总错误数
    pub total_errors: u64,
    /// 消息类型数量
    pub message_count: usize,
    /// 全局包速率
    pub global_packet_rate: Option<f64>,
}
