//! 发送器配置组件

use super::super::config::SenderConfig;
use super::PathSelector;
use crate::app::config::types::NetworkType;
use eframe::egui;

/// 渲染网络类型选择组合框
fn render_network_type_combo(
    ui: &mut egui::Ui,
    network_type: &mut NetworkType,
) {
    egui::ComboBox::from_label("")
        .selected_text(format!("{:?}", network_type))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                network_type,
                NetworkType::Unicast,
                "Unicast",
            );
            ui.selectable_value(
                network_type,
                NetworkType::Multicast,
                "Multicast",
            );
            ui.selectable_value(
                network_type,
                NetworkType::Broadcast,
                "Broadcast",
            );
        });
}

/// 渲染发送器配置区域
pub fn render_sender_config(
    ui: &mut egui::Ui,
    config: &mut SenderConfig,
) {
    egui::Grid::new("sender_config")
        .num_columns(2)
        .min_col_width(80.0) // 标题列固定最小宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Save Path");
            ui.add(PathSelector::new(
                &mut config.dataset_path,
            ));
            ui.end_row();

            ui.label("Target Address");
            ui.text_edit_singleline(&mut config.address);
            ui.end_row();

            ui.label("Target Port");
            ui.add(
                egui::DragValue::new(&mut config.port)
                    .range(1..=65535),
            );
            ui.end_row();

            ui.label("Network Type");
            render_network_type_combo(
                ui,
                &mut config.network_type,
            );
            ui.end_row();
        });
}
