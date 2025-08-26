use chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::time::{sleep_until, Instant as TokioInstant};

/// 时序控制器（基于数据包时间戳）
pub struct TimingController {
    first_packet_time: Option<DateTime<Utc>>,
    real_start_time: Option<TokioInstant>,
}

impl TimingController {
    /// 创建新的时序控制器
    pub fn new() -> Self {
        Self {
            first_packet_time: None,
            real_start_time: None,
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

            // 完全按照原始时间戳进行精确等待
            if wait_duration > Duration::from_nanos(1) {
                sleep_until(now + wait_duration).await;
            }
        }
    }
}

impl Default for TimingController {
    fn default() -> Self {
        Self::new()
    }
}
