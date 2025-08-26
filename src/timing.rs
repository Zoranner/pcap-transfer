use chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::time::{sleep_until, Instant as TokioInstant};
use tracing::warn;

/// 时序控制器（基于数据包时间戳）
pub struct TimingController {
    first_packet_time: Option<DateTime<Utc>>,
    real_start_time: Option<TokioInstant>,
    max_delay_threshold_ms: u64,
}

impl TimingController {
    /// 创建新的时序控制器
    pub fn new() -> Self {
        Self {
            first_packet_time: None,
            real_start_time: None,
            max_delay_threshold_ms: 100, // 默认100ms延迟阈值
        }
    }

    /// 创建带自定义延迟阈值的时序控制器
    pub fn with_delay_threshold(max_delay_ms: u64) -> Self {
        Self {
            first_packet_time: None,
            real_start_time: None,
            max_delay_threshold_ms: max_delay_ms,
        }
    }

    /// 等待到指定的数据包时间
    pub async fn wait_for_packet_time(
        &mut self,
        packet_time: DateTime<Utc>,
    ) {
        if self.first_packet_time.is_none() {
            // 第一个数据包，记录基准时间
            self.first_packet_time = Some(packet_time);
            self.real_start_time =
                Some(TokioInstant::now());
            return;
        }

        let first_time = self.first_packet_time.unwrap();
        let real_start = self.real_start_time.unwrap();

        // 计算数据包相对于第一个包的时间差
        let packet_offset = packet_time
            .signed_duration_since(first_time)
            .to_std()
            .unwrap_or_default();

        // 计算应该发送的实际时间
        let target_time = real_start + packet_offset;
        let now = TokioInstant::now();

        if target_time > now {
            let wait_duration =
                target_time.duration_since(now);

            // 如果设置了延迟阈值（非0），则限制最大等待时间
            let actual_wait =
                if self.max_delay_threshold_ms > 0 {
                    let max_wait = Duration::from_millis(
                        self.max_delay_threshold_ms,
                    );
                    wait_duration.min(max_wait)
                } else {
                    // 延迟阈值为0时，不限制等待时间
                    wait_duration
                };

            if actual_wait > Duration::from_nanos(1) {
                sleep_until(now + actual_wait).await;
            }
        } else if now.duration_since(real_start)
            > packet_offset
                + Duration::from_millis(
                    self.max_delay_threshold_ms,
                )
        {
            // 如果延迟超过阈值，给出警告
            warn!(
                "数据包发送延迟: 预期={:.3}s, 实际={:.3}s",
                packet_offset.as_secs_f64(),
                now.duration_since(real_start)
                    .as_secs_f64()
            );
        }
    }
}

impl Default for TimingController {
    fn default() -> Self {
        Self::new()
    }
}
