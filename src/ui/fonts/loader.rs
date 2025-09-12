//! 字体设置模块 - 处理跨平台中文字体支持

use egui;

/// 设置跨平台的中文字体支持
pub fn setup_fonts(_ctx: &egui::Context) {
    // 暂时使用默认字体，egui 0.13 的字体 API 与 0.29 不同
    // 后续可以根据需要添加自定义字体支持
    tracing::info!("Using default egui fonts");
}
