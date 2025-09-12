//! 接收器配置组件

use super::super::config::ReceiverConfig;
use super::PathSelector;
use crate::app::config::types::NetworkType;
use egui;

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

/// 渲染接收器配置区域
pub fn render_receiver_config(
    ui: &mut egui::Ui,
    config: &mut ReceiverConfig,
) {
    egui::Grid::new("receiver_config")
        .num_columns(2)
        .min_col_width(80.0) // 标题列固定最小宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Output Path");
            ui.add(PathSelector::new(
                &mut config.output_path,
            ));
            ui.end_row();

            ui.label("Dataset Name");
            ui.add(
                egui::TextEdit::singleline(
                    &mut config.dataset_name,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Listen Address");
            ui.add(
                egui::TextEdit::singleline(
                    &mut config.address,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Listen Port");
            ui.add(
                egui::DragValue::new(&mut config.port)
                    .range(1.0..=65535.0),
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
