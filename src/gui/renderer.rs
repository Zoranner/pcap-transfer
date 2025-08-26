//! 渲染器模块 - 处理各种UI界面的渲染逻辑

use eframe::egui;

use super::config::{
    AppMode, ReceiverConfig, SenderConfig,
};
use crate::config::NetworkType;
use crate::sender::TransferState;
use crate::stats::TransferStats;

/// 渲染主菜单
pub fn render_main_menu(
    ui: &mut egui::Ui,
    mode: &mut AppMode,
) {
    ui.heading("Data Transfer - 数据包传输测试工具");
    ui.separator();

    ui.add_space(20.0);

    ui.horizontal(|ui| {
        if ui.button("发送数据包").clicked() {
            *mode = AppMode::Sender;
        }

        ui.add_space(20.0);

        if ui.button("接收数据包").clicked() {
            *mode = AppMode::Receiver;
        }
    });

    ui.add_space(20.0);
    ui.label("选择操作模式开始使用工具");
}

/// 渲染发送器配置区域
pub fn render_sender_config(
    ui: &mut egui::Ui,
    config: &mut SenderConfig,
) {
    egui::Grid::new("sender_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            ui.label("数据集路径:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(
                    &mut config.dataset_path,
                );
                if ui.button("浏览").clicked() {
                    if let Some(path) =
                        rfd::FileDialog::new().pick_folder()
                    {
                        config.dataset_path = path
                            .to_string_lossy()
                            .to_string();
                    }
                }
            });
            ui.end_row();

            ui.label("目标地址:");
            ui.text_edit_singleline(&mut config.address);
            ui.end_row();

            ui.label("目标端口:");
            ui.add(
                egui::DragValue::new(&mut config.port)
                    .range(1..=65535),
            );
            ui.end_row();

            ui.label("网络类型:");
            render_network_type_combo(
                ui,
                &mut config.network_type,
            );
            ui.end_row();
        });
}

/// 渲染接收器配置区域
pub fn render_receiver_config(
    ui: &mut egui::Ui,
    config: &mut ReceiverConfig,
) {
    egui::Grid::new("receiver_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            ui.label("输出路径:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(
                    &mut config.output_path,
                );
                if ui.button("浏览").clicked() {
                    if let Some(path) =
                        rfd::FileDialog::new().pick_folder()
                    {
                        config.output_path = path
                            .to_string_lossy()
                            .to_string();
                    }
                }
            });
            ui.end_row();

            ui.label("数据集名称:");
            ui.text_edit_singleline(
                &mut config.dataset_name,
            );
            ui.end_row();

            ui.label("监听地址:");
            ui.text_edit_singleline(&mut config.address);
            ui.end_row();

            ui.label("监听端口:");
            ui.add(
                egui::DragValue::new(&mut config.port)
                    .range(1..=65535),
            );
            ui.end_row();

            ui.label("网络类型:");
            render_network_type_combo(
                ui,
                &mut config.network_type,
            );
            ui.end_row();
        });
}

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
                NetworkType::Broadcast,
                "Broadcast",
            );
            ui.selectable_value(
                network_type,
                NetworkType::Multicast,
                "Multicast",
            );
        });
}

/// 渲染统计信息
pub fn render_stats(
    ui: &mut egui::Ui,
    stats: &TransferStats,
) {
    ui.group(|ui| {
        ui.heading("传输统计");

        egui::Grid::new("stats")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("已处理包数:");
                ui.label(
                    stats
                        .get_packets_processed()
                        .to_string(),
                );
                ui.end_row();

                ui.label("已传输字节:");
                ui.label(crate::utils::format_bytes(
                    stats.get_bytes_processed(),
                ));
                ui.end_row();

                ui.label("数据速率:");
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
                ui.end_row();

                ui.label("持续时间:");
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
                ui.end_row();

                ui.label("错误数:");
                ui.label(format!("{}", stats.get_errors()));
                ui.end_row();
            });
    });
}

/// 渲染页面头部（带返回按钮）
pub fn render_page_header(
    ui: &mut egui::Ui,
    title: &str,
    mode: &mut AppMode,
    transfer_state: &mut TransferState,
) {
    ui.horizontal(|ui| {
        if ui.button("← 返回").clicked() {
            *mode = AppMode::MainMenu;
            *transfer_state = TransferState::Idle;
        }
        ui.heading(title);
    });
    ui.separator();
}
