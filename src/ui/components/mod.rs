//! GUI组件模块 - 包含各种UI组件的实现

pub mod app_renderer;
pub use app_renderer::UserAction;
pub mod path_selector;
pub mod receiver_config;
pub mod sender_config;
pub mod stats;

// 重新导出主要组件
pub use app_renderer::AppRenderer;
pub use path_selector::PathSelector;
pub use receiver_config::render_receiver_config;
pub use sender_config::render_sender_config;
pub use stats::render_stats;
