//! 应用渲染器模块
//!
//! 负责渲染发送器和接收器的主要界面

use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::core::network::sender::TransferState;
use crate::core::stats::collector::TransferStats;
use crate::ui::app_state::AppStateManager;
use crate::ui::config::{ReceiverConfig, SenderConfig};

/// 用户动作枚举
#[derive(Debug, Clone, PartialEq)]
pub enum UserAction {
    None,
    StartSender,
    StopSender,
    StartReceiver,
    StopReceiver,
}

/// 应用渲染器
pub struct AppRenderer;

impl AppRenderer {
    /// 渲染发送器界面，返回用户动作
    pub fn render_sender(
        ui: &mut egui::Ui,
        config: &mut SenderConfig,
        transfer_state: &TransferState,
        stats: &Arc<Mutex<TransferStats>>,
    ) -> UserAction {
        // 下半部分：传输统计 (固定高度)
        egui::TopBottomPanel::bottom("sender_stats_panel")
            .resizable(false)
            .exact_height(200.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0); // 补偿TopBottomPanel的默认内边距
                    ui.heading("Transfer Statistics");
                    ui.separator();
                    ui.add_space(8.0);
                    AppStateManager::render_sender_stats_safely(stats, ui);
                });
            });

        // 主内容区域：配置和控制区域 (占据剩余空间)
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.heading("Configuration Parameters");
                ui.separator();
                ui.add_space(8.0);

                // 配置区域
                crate::ui::components::render_sender_config(ui, config);

                ui.add_space(20.0);

                // 控制按钮
                Self::render_sender_controls(ui, transfer_state)
            }).inner
        }).inner
    }

    /// 渲染接收器界面，返回用户动作
    pub fn render_receiver(
        ui: &mut egui::Ui,
        config: &mut ReceiverConfig,
        transfer_state: &TransferState,
        stats: &Arc<Mutex<TransferStats>>,
    ) -> UserAction {
        // 下半部分：传输统计 (固定高度)
        egui::TopBottomPanel::bottom("receiver_stats_panel")
            .resizable(false)
            .exact_height(200.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0); // 补偿TopBottomPanel的默认内边距
                    ui.heading("Transfer Statistics");
                    ui.separator();
                    ui.add_space(8.0);
                    AppStateManager::render_receiver_stats_safely(stats, ui);
                });
            });

        // 主内容区域：配置和控制区域 (占据剩余空间)
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.heading("Configuration Parameters");
                ui.separator();
                ui.add_space(8.0);

                // 配置区域
                crate::ui::components::render_receiver_config(ui, config);

                ui.add_space(20.0);

                // 控制按钮
                Self::render_receiver_controls(ui, transfer_state)
            }).inner
        }).inner
    }

    /// 渲染发送器控制按钮，返回用户动作
    fn render_sender_controls(
        ui: &mut egui::Ui,
        transfer_state: &TransferState,
    ) -> UserAction {
        let can_start = matches!(
            transfer_state,
            TransferState::Idle
                | TransferState::Error(_)
                | TransferState::Completed
        );
        let can_stop = matches!(
            transfer_state,
            TransferState::Running
        );

        let mut action = UserAction::None;

        ui.horizontal(|ui| {
            if can_start
                && ui.button("Start Sending").clicked()
            {
                action = UserAction::StartSender;
            }
            if can_stop
                && ui.button("Stop Sending").clicked()
            {
                action = UserAction::StopSender;
            }

            match transfer_state {
                TransferState::Completed => {
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "Send Completed",
                    );
                }
                TransferState::Error(err) => {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Error: {}", err),
                    );
                }
                _ => {}
            }
        });

        action
    }

    /// 渲染接收器控制按钮，返回用户动作
    fn render_receiver_controls(
        ui: &mut egui::Ui,
        transfer_state: &TransferState,
    ) -> UserAction {
        let can_start = matches!(
            transfer_state,
            TransferState::Idle
                | TransferState::Error(_)
                | TransferState::Completed
        );
        let can_stop = matches!(
            transfer_state,
            TransferState::Running
        );

        let mut action = UserAction::None;

        ui.horizontal(|ui| {
            if can_start
                && ui.button("Start Receiving").clicked()
            {
                action = UserAction::StartReceiver;
            }
            if can_stop
                && ui.button("Stop Receiving").clicked()
            {
                action = UserAction::StopReceiver;
            }

            match transfer_state {
                TransferState::Completed => {
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "Receive Completed",
                    );
                }
                TransferState::Error(err) => {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Error: {}", err),
                    );
                }
                _ => {}
            }
        });

        action
    }
}
