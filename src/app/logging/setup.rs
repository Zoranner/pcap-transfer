//! 日志系统设置

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// 初始化日志系统
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // 默认日志级别：debug模式下显示debug信息，release模式下显示info信息
            if cfg!(debug_assertions) {
                EnvFilter::new("pcap_transfer=debug,warn")
            } else {
                EnvFilter::new("pcap_transfer=info,warn")
            }
        });

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true),
        )
        .with(filter)
        .init();
}
