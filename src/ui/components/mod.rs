//! GUI组件模块 - 包含各种UI组件的实现

pub mod message_config;
pub mod sender_config;
pub mod stats_panel;

// 重新导出主要组件
pub use message_config::{
    render_all_messages_config,
    render_global_network_config, GlobalNetworkConfig,
};
pub use stats_panel::render_stats_panel;
