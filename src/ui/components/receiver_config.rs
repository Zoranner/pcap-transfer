//! 接收器配置组件

use super::super::config::ReceiverConfig;
use super::PathSelector;
use crate::app::config::types::NetworkType;
use egui;

/// 渲染网络类型选择组合框
fn render_network_type_combo(
    ui: &mut egui::Ui,
    network_type: &mut NetworkType,
    enabled: bool,
) {
    ui.add_enabled_ui(enabled, |ui| {
        egui::ComboBox::from_id_source("receiver_network_type_combo")
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
    });
}

/// 渲染接收器配置区域
pub fn render_receiver_config(
    ui: &mut egui::Ui,
    config: &mut ReceiverConfig,
    enabled: bool,
) {
    egui::Grid::new("receiver_config_grid")
        .num_columns(2)
        .min_col_width(80.0) // 标题列固定最小宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Output Path");
            ui.add_enabled(
                enabled,
                PathSelector::new(&mut config.output_path),
            );
            ui.end_row();

            ui.label("PCAP Name");
            ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(
                    &mut config.dataset_name,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Listen Address");
            ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(
                    &mut config.address,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Listen Port");
            ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut config.port)
                    .range(1..=65535),
            );
            ui.end_row();

            ui.label("Network Type");
            render_network_type_combo(
                ui,
                &mut config.network_type,
                enabled,
            );
            ui.end_row();
        });
}
