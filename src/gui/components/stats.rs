//! 统计信息组件

use crate::stats::TransferStats;
use eframe::egui;

/// 渲染统计信息
pub fn render_stats(
    ui: &mut egui::Ui,
    stats: &TransferStats,
) {
    egui::Grid::new("stats")
        .num_columns(2)
        .min_col_width(80.0) // 标题列固定最小宽度
        .max_col_width(ui.available_width() - 100.0) // 确保Grid撑满可用宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("传输包数");
            // 让数值列占用剩余空间
            ui.with_layout(
                egui::Layout::left_to_right(
                    egui::Align::Center,
                ),
                |ui| {
                    ui.label(
                        stats
                            .get_packets_processed()
                            .to_string(),
                    );
                    ui.allocate_response(
                        egui::Vec2::new(
                            ui.available_width(),
                            0.0,
                        ),
                        egui::Sense::hover(),
                    );
                },
            );
            ui.end_row();

            ui.label("传输字节");
            ui.with_layout(
                egui::Layout::left_to_right(
                    egui::Align::Center,
                ),
                |ui| {
                    ui.label(crate::utils::format_bytes(
                        stats.get_bytes_processed(),
                    ));
                    ui.allocate_response(
                        egui::Vec2::new(
                            ui.available_width(),
                            0.0,
                        ),
                        egui::Sense::hover(),
                    );
                },
            );
            ui.end_row();

            ui.label("数据速率");
            ui.with_layout(
                egui::Layout::left_to_right(
                    egui::Align::Center,
                ),
                |ui| {
                    if let Some(packet_rate) =
                        stats.get_packet_rate_bps()
                    {
                        ui.label(format!(
                            "{}/s",
                            crate::utils::format_bytes(
                                packet_rate as u64 / 8
                            )
                        ));
                    } else {
                        ui.label("未知".to_string());
                    }
                    ui.allocate_response(
                        egui::Vec2::new(
                            ui.available_width(),
                            0.0,
                        ),
                        egui::Sense::hover(),
                    );
                },
            );
            ui.end_row();

            ui.label("持续时间");
            ui.with_layout(
                egui::Layout::left_to_right(
                    egui::Align::Center,
                ),
                |ui| {
                    if let Some(packet_duration) =
                        stats.get_packet_duration()
                    {
                        ui.label(format!(
                            "{:.3}s",
                            packet_duration.as_secs_f64()
                        ));
                    } else {
                        ui.label("未知".to_string());
                    }
                    ui.allocate_response(
                        egui::Vec2::new(
                            ui.available_width(),
                            0.0,
                        ),
                        egui::Sense::hover(),
                    );
                },
            );
            ui.end_row();

            ui.label("错误数量");
            ui.with_layout(
                egui::Layout::left_to_right(
                    egui::Align::Center,
                ),
                |ui| {
                    ui.label(format!(
                        "{}",
                        stats.get_errors()
                    ));
                    ui.allocate_response(
                        egui::Vec2::new(
                            ui.available_width(),
                            0.0,
                        ),
                        egui::Sense::hover(),
                    );
                },
            );
            ui.end_row();
        });
}
