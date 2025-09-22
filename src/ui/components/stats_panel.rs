//! 统计信息面板组件

use crate::core::stats::message_stats::MessageStatsManager;
use egui;

/// 渲染统计信息面板
pub fn render_stats_panel(
    ui: &mut egui::Ui,
    message_stats: &std::sync::Arc<
        std::sync::Mutex<MessageStatsManager>,
    >,
) {
    ui.vertical(|ui| {
        ui.add_space(5.0);
        ui.heading("统计信息");
        ui.separator();
        ui.add_space(5.0);

        // 添加滚动区域以支持多个消息类型
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 20.0)
            .show(ui, |ui| {
                if let Ok(stats) = message_stats.lock() {
                    let all_stats =
                        stats.get_all_message_stats();

                    if all_stats.is_empty() {
                        ui.label("暂无统计数据");
                        return;
                    }

                    // 按消息名称排序
                    let mut message_names: Vec<_> =
                        all_stats.keys().collect();
                    message_names.sort();

                    // 显示每个消息类型的统计
                    for (i, message_name) in
                        message_names.iter().enumerate()
                    {
                        if let Some(msg_stats) =
                            all_stats.get(*message_name)
                        {
                            render_message_stats_group(
                                ui,
                                message_name,
                                msg_stats,
                                i,
                            );
                            ui.add_space(8.0); // 消息类型之间的间距
                        }
                    }

                    // 全局汇总（放在最后）
                    ui.separator();
                    ui.add_space(5.0);
                    render_global_summary(ui, &stats);
                } else {
                    ui.label("无法获取统计数据");
                }
            });
    });
}

/// 渲染单个消息类型的统计信息组
fn render_message_stats_group(
    ui: &mut egui::Ui,
    message_name: &str,
    msg_stats: &crate::core::stats::message_stats::MessageStats,
    index: usize,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.strong(message_name);
            ui.separator();

            // 统计信息网格
            egui::Grid::new(format!(
                "msg_stats_grid_{}",
                index
            ))
            .num_columns(2)
            .min_col_width(80.0) // 标题列固定最小宽度
            .max_col_width(ui.available_width() - 100.0) // 确保Grid撑满可用宽度
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                render_stats_row(
                    ui,
                    "包数量",
                    &format!("{}", msg_stats.packets_sent),
                );
                render_stats_row(
                    ui,
                    "字节数",
                    &crate::utils::helpers::format_bytes(
                        msg_stats.bytes_sent,
                    ),
                );
                render_stats_row(
                    ui,
                    "错误数",
                    &format!("{}", msg_stats.errors),
                );

                // 发送速率
                let rate_text = if let Some(rate) =
                    msg_stats.get_packet_rate()
                {
                    format!("{:.1} pps", rate)
                } else {
                    "0 pps".to_string()
                };
                render_stats_row(
                    ui,
                    "发送速率",
                    &rate_text,
                );

                // 字节速率
                let bytes_rate_text = if let Some(rate) =
                    msg_stats.get_bytes_rate()
                {
                    format!(
                        "{}/s",
                        crate::utils::helpers::format_bytes(
                            rate as u64
                        )
                    )
                } else {
                    "0 B/s".to_string()
                };
                render_stats_row(
                    ui,
                    "字节速率",
                    &bytes_rate_text,
                );

                // 运行时长
                let duration_text = if let Some(duration) =
                    msg_stats.get_duration()
                {
                    format!(
                        "{:.1}s",
                        duration.as_secs_f64()
                    )
                } else {
                    "未开始".to_string()
                };
                render_stats_row(
                    ui,
                    "运行时长",
                    &duration_text,
                );
            });
        });
    });
}

/// 渲染全局汇总统计信息
fn render_global_summary(
    ui: &mut egui::Ui,
    stats: &MessageStatsManager,
) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.strong("全局汇总");
            ui.separator();

            let global = stats.get_global_summary();
            egui::Grid::new("global_stats_grid")
                .num_columns(2)
                .min_col_width(80.0) // 标题列固定最小宽度
                .max_col_width(ui.available_width() - 100.0) // 确保Grid撑满可用宽度
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    render_stats_row(ui, "消息类型", &format!("{}", global.message_count));
                    render_stats_row(ui, "总包数", &format!("{}", global.total_packets));
                    render_stats_row(ui, "总字节", &crate::utils::helpers::format_bytes(global.total_bytes));
                    render_stats_row(ui, "总错误", &format!("{}", global.total_errors));
                    // 全局速率
                    let rate_text = if let Some(rate) = global.global_packet_rate {
                        format!("{:.1} pps", rate)
                    } else {
                        "0 pps".to_string()
                    };
                    render_stats_row(ui, "全局速率", &rate_text);
                });
        });
    });
}

/// 渲染统计信息行（标签 + 数值）
fn render_stats_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
) {
    ui.label(label);
    ui.with_layout(
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(value);
            ui.allocate_response(
                egui::Vec2::new(ui.available_width(), 0.0),
                egui::Sense::hover(),
            );
        },
    );
    ui.end_row();
}
