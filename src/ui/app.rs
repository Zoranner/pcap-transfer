//! GUI主应用程序模块

use eframe::egui;
use std::sync::{Arc, Mutex};
use tracing;

use crate::app::config::manager::ConfigManager;
use crate::app::config::validator::ConfigValidator;
use crate::app::error::types::{AppError, Result};
use crate::core::network::sender::TransferState;
use crate::core::services::transfer_service::TransferService;
use crate::core::stats::collector::TransferStats;

use super::app_state::AppStateManager;
use super::components::{AppRenderer, UserAction};
use super::config::{
    ReceiverConfig, SelectedTab, SenderConfig,
};
use super::fonts::loader;
use super::widgets;

/// GUI 应用程序
pub struct DataTransferApp {
    selected_tab: SelectedTab,
    sender_config: SenderConfig,
    receiver_config: ReceiverConfig,
    // 发送器状态
    sender_transfer_state: TransferState,
    sender_stats: Arc<Mutex<TransferStats>>,
    sender_shared_state: Option<Arc<Mutex<TransferState>>>,
    // 接收器状态
    receiver_transfer_state: TransferState,
    receiver_stats: Arc<Mutex<TransferStats>>,
    receiver_shared_state:
        Option<Arc<Mutex<TransferState>>>,
    // Tokio runtime handle
    runtime_handle: Option<tokio::runtime::Handle>,
    // 服务
    transfer_service: TransferService,
}

impl Default for DataTransferApp {
    fn default() -> Self {
        let mut config_manager =
            ConfigManager::new("pcap-transfer")
                .unwrap_or_else(|e| {
                    tracing::error!(
                "Failed to create config manager: {}",
                e
            );
                    panic!(
                "Unable to create config manager: {}",
                e
            );
                });

        // 尝试加载配置，如果失败则使用默认配置
        if let Err(e) = config_manager.load() {
            tracing::warn!("Failed to load config file, using default config: {}", e);
        }

        // 从配置管理器初始化GUI配置
        let config = config_manager.config();
        let sender_config = SenderConfig {
            dataset_path: config
                .sender
                .dataset_path
                .clone(),
            address: config.sender.network.address.clone(),
            port: config.sender.network.port,
            network_type: config_manager
                .get_sender_network_type(),
            interface: if config
                .sender
                .network
                .interface
                .is_empty()
            {
                None
            } else {
                Some(
                    config.sender.network.interface.clone(),
                )
            },
        };

        let receiver_config = ReceiverConfig {
            output_path: config
                .receiver
                .output_path
                .clone(),
            dataset_name: config
                .receiver
                .dataset_name
                .clone(),
            address: config
                .receiver
                .network
                .address
                .clone(),
            port: config.receiver.network.port,
            network_type: config_manager
                .get_receiver_network_type(),
            interface: if config
                .receiver
                .network
                .interface
                .is_empty()
            {
                None
            } else {
                Some(
                    config
                        .receiver
                        .network
                        .interface
                        .clone(),
                )
            },
        };

        let transfer_service =
            TransferService::new(config_manager);

        Self {
            selected_tab: SelectedTab::Sender,
            sender_config,
            receiver_config,
            sender_transfer_state: TransferState::Idle,
            sender_stats: Arc::new(Mutex::new(
                TransferStats::default(),
            )),
            sender_shared_state: None,
            receiver_transfer_state: TransferState::Idle,
            receiver_stats: Arc::new(Mutex::new(
                TransferStats::default(),
            )),
            receiver_shared_state: None,
            runtime_handle: None,
            transfer_service,
        }
    }
}

impl DataTransferApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置跨平台的中文字体支持
        loader::setup_fonts(&cc.egui_ctx);
        Self::default()
    }

    /// 启动发送器
    fn start_sender(&mut self) {
        if let Err(e) =
            ConfigValidator::validate_sender_config(
                &self.sender_config,
            )
        {
            self.sender_transfer_state =
                TransferState::Error(e.to_string());
            return;
        }

        if let Some(handle) = &self.runtime_handle {
            match self.transfer_service.start_sender(
                &self.sender_config,
                Arc::clone(&self.sender_stats),
                handle,
            ) {
                Ok(shared_state) => {
                    self.sender_shared_state =
                        Some(shared_state);
                    self.sender_transfer_state =
                        TransferState::Running;
                }
                Err(e) => {
                    self.sender_transfer_state =
                        TransferState::Error(e.to_string());
                }
            }
        } else {
            self.sender_transfer_state =
                TransferState::Error(
                    "Runtime handle not initialized"
                        .to_string(),
                );
        }
    }

    /// 启动接收器
    fn start_receiver(&mut self) {
        if let Err(e) =
            ConfigValidator::validate_receiver_config(
                &self.receiver_config,
            )
        {
            tracing::error!(
                "Receiver config validation failed: {}",
                e
            );
            self.receiver_transfer_state =
                TransferState::Error(e.to_string());
            return;
        }

        // 获取当前的 Tokio runtime handle
        let handle =
            match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => {
                    tracing::error!("Unable to get Tokio runtime handle");
                    self.receiver_transfer_state = TransferState::Error(
                    "Runtime handle not initialized".to_string(),
                );
                    return;
                }
            };
        self.runtime_handle = Some(handle.clone());

        match self.transfer_service.start_receiver(
            &self.receiver_config,
            Arc::clone(&self.receiver_stats),
            &handle,
        ) {
            Ok(shared_state) => {
                self.receiver_shared_state =
                    Some(shared_state);
                self.receiver_transfer_state =
                    TransferState::Running;
            }
            Err(e) => {
                self.receiver_transfer_state =
                    TransferState::Error(e.to_string());
            }
        }
    }

    /// 停止发送器
    fn stop_sender(&mut self) {
        TransferService::stop_transfer(
            &self.sender_shared_state,
        );
        self.sender_transfer_state = TransferState::Idle;
    }

    /// 停止接收器
    fn stop_receiver(&mut self) {
        TransferService::stop_transfer(
            &self.receiver_shared_state,
        );
        self.receiver_transfer_state = TransferState::Idle;
    }
}

impl eframe::App for DataTransferApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        // 同步共享的传输状态
        AppStateManager::sync_sender_state(
            &mut self.sender_transfer_state,
            &self.sender_shared_state,
        );
        AppStateManager::sync_receiver_state(
            &mut self.receiver_transfer_state,
            &self.receiver_shared_state,
        );

        // 标签按钮区域
        egui::TopBottomPanel::top("tab_buttons")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    // 发送器状态标签按钮
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width() * 0.5, 40.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if widgets::status::StatusTabButton::new(
                                "Sender",
                                self.sender_transfer_state.clone(),
                                self.selected_tab == SelectedTab::Sender,
                            )
                            .show(ui)
                            .clicked()
                            {
                                self.selected_tab = SelectedTab::Sender;
                            }
                        },
                    );

                    // 接收器状态标签按钮
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), 40.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if widgets::status::StatusTabButton::new(
                                "Receiver",
                                self.receiver_transfer_state.clone(),
                                self.selected_tab == SelectedTab::Receiver,
                            )
                            .show(ui)
                            .clicked()
                            {
                                self.selected_tab = SelectedTab::Receiver;
                            }
                        },
                    );
                });
                ui.add_space(8.0);
            });

        // 主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 根据选中的标签页渲染对应内容
            match self.selected_tab {
                SelectedTab::Sender => {
                    let action = AppRenderer::render_sender(
                        ui,
                        &mut self.sender_config,
                        &self.sender_transfer_state,
                        &self.sender_stats,
                    );

                    // 处理用户动作
                    match action {
                        UserAction::StartSender => {
                            self.start_sender();
                        }
                        UserAction::StopSender => {
                            self.stop_sender();
                        }
                        _ => {}
                    }
                }
                SelectedTab::Receiver => {
                    let action =
                        AppRenderer::render_receiver(
                            ui,
                            &mut self.receiver_config,
                            &self.receiver_transfer_state,
                            &self.receiver_stats,
                        );

                    // 处理用户动作
                    match action {
                        UserAction::StartReceiver => {
                            self.start_receiver();
                        }
                        UserAction::StopReceiver => {
                            self.stop_receiver();
                        }
                        _ => {}
                    }
                }
            }
        });

        // 定期刷新界面以更新统计信息
        ctx.request_repaint_after(
            std::time::Duration::from_millis(100),
        );
    }

    fn on_exit(
        &mut self,
        _gl: Option<&eframe::glow::Context>,
    ) {
        // 应用退出时保存配置
        if let Err(e) =
            self.transfer_service.config_manager.save()
        {
            tracing::error!(
                "Failed to save config file: {}",
                e
            );
        }
    }
}

/// 启动 GUI 应用程序
pub fn run_gui() -> Result<()> {
    // 加载应用程序图标
    let icon_data =
        match super::icons::loader::create_icon_data() {
            Ok(icon) => Some(icon),
            Err(e) => {
                eprintln!("Warning: Failed to load application icon: {}", e);
                None
            }
        };

    let mut viewport_builder =
        egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_min_inner_size([400.0, 500.0])
            .with_max_inner_size([400.0, 500.0])
            .with_resizable(true)
            .with_title("Pcap Transfer");

    // 如果图标加载成功，则设置图标
    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        // 添加额外的窗口控制选项
        hardware_acceleration:
            eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    // 获取当前的 tokio runtime handle
    let runtime_handle = tokio::runtime::Handle::current();

    eframe::run_native(
        "Pcap Transfer",
        options,
        Box::new(move |cc| {
            let mut app = DataTransferApp::new(cc);
            app.runtime_handle = Some(runtime_handle);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| {
        tracing::error!("GUI startup failed: {}", e);
        AppError::Gui(e.to_string())
    })?;

    Ok(())
}
