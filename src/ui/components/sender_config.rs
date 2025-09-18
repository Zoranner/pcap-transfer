//! 发送器配置组件

use super::super::config::SenderConfig;
use super::PathSelector;
use crate::app::config::types::{DataFormat, NetworkType};
use egui;

/// CSV文件选择组件
struct CsvFileSelector<'a> {
    path: &'a mut String,
}

impl<'a> CsvFileSelector<'a> {
    fn new(path: &'a mut String) -> Self {
        Self { path }
    }
}

impl<'a> egui::Widget for CsvFileSelector<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let button_width = 60.0;
        let spacing = ui.spacing().item_spacing.x;
        let actual_width = ui.available_width();
        let input_width =
            actual_width - button_width - spacing;

        ui.horizontal_centered(|ui| {
            let text_edit =
                egui::TextEdit::singleline(self.path)
                    .desired_width(input_width)
                    .hint_text("Select CSV file...");

            let text_response = ui.add(text_edit);

            if ui.button("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CSV files", &["csv"])
                    .pick_file()
                {
                    *self.path =
                        path.to_string_lossy().to_string();
                }
            }

            text_response
        })
        .inner
    }
}

/// 渲染数据格式选择组合框
fn render_data_format_combo(
    ui: &mut egui::Ui,
    data_format: &mut DataFormat,
    enabled: bool,
) {
    ui.add_enabled_ui(enabled, |ui| {
        egui::ComboBox::from_id_salt(
            "sender_data_format_combo",
        )
        .selected_text(format!("{}", data_format))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                data_format,
                DataFormat::Pcap,
                format!("{}", DataFormat::Pcap),
            );
            ui.selectable_value(
                data_format,
                DataFormat::Csv,
                format!("{}", DataFormat::Csv),
            );
        });
    });
}

/// 渲染网络类型选择组合框
fn render_network_type_combo(
    ui: &mut egui::Ui,
    network_type: &mut NetworkType,
    enabled: bool,
) {
    ui.add_enabled_ui(enabled, |ui| {
        egui::ComboBox::from_id_salt(
            "sender_network_type_combo",
        )
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

/// 渲染发送器配置区域
pub fn render_sender_config(
    ui: &mut egui::Ui,
    config: &mut SenderConfig,
    enabled: bool,
) {
    egui::Grid::new("sender_config_grid")
        .num_columns(2)
        .min_col_width(80.0) // 标题列固定最小宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Data Format");
            render_data_format_combo(
                ui,
                &mut config.data_format,
                enabled,
            );
            ui.end_row();

            // 根据数据格式显示对应的路径输入框
            match config.data_format {
                DataFormat::Pcap => {
                    ui.label("PCAP Path");
                    ui.add_enabled(
                        enabled,
                        PathSelector::new(
                            &mut config.pcap_path,
                        ),
                    );
                    ui.end_row();
                }
                DataFormat::Csv => {
                    ui.label("CSV File");
                    ui.add_enabled(
                        enabled,
                        CsvFileSelector::new(
                            &mut config.csv_file,
                        ),
                    );
                    ui.end_row();

                    ui.label("Packet Interval");
                    ui.add_enabled(
                        enabled,
                        egui::DragValue::new(
                            &mut config.csv_packet_interval,
                        )
                        .range(1..=60000) // 1毫秒到60秒
                        .suffix(" ms"),
                    );
                    ui.end_row();
                }
            }

            ui.label("Target Address");
            ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(
                    &mut config.address,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Target Port");
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
