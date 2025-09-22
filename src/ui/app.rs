//! GUI主应用程序模块

use egui;
use std::sync::{Arc, Mutex};
use tracing;

use crate::app::config::manager::ConfigManager;
use crate::app::error::types::{AppError, Result};
use crate::core::network::TransferState;
use crate::core::services::transfer_service::TransferService;
use crate::core::stats::collector::TransferStats;
use crate::core::stats::message_stats::MessageStatsManager;

use super::app_state::AppStateManager;
use super::config::{MessageUIConfig, SenderConfig};
use super::fonts::loader;

/// GUI 应用程序
pub struct DataTransferApp {
    message_config: MessageUIConfig,
    // 传输状态
    transfer_state: TransferState,
    stats: Arc<Mutex<TransferStats>>,
    message_stats: Arc<Mutex<MessageStatsManager>>,
    shared_state: Option<Arc<Mutex<TransferState>>>,
    // Tokio runtime handle
    runtime_handle: Option<tokio::runtime::Handle>,
    // 服务
    transfer_service: TransferService,
}

impl Default for DataTransferApp {
    fn default() -> Self {
        let mut config_manager =
            ConfigManager::new()
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

        // 初始化消息配置
        let mut message_config = MessageUIConfig::default();

        // 从配置管理器加载消息配置
        let messages = config_manager.get_messages();
        tracing::info!(
            "Loaded {} messages from config",
            messages.len()
        );
        if !messages.is_empty() {
            use crate::app::config::message_types::MessageRuntimeState;
            message_config.messages = messages
                .iter()
                .map(|def| {
                    tracing::info!("Loading message: {} with {} fields", def.name, def.fields.len());
                    MessageRuntimeState::from_definition(def.clone())
                })
                .collect();
        } else {
            tracing::warn!(
                "No messages found in configuration"
            );
        }

        // 设置全局网络配置
        let config = config_manager.config();
        message_config.global_network.address =
            config.sender.network.address.clone();
        message_config.global_network.port =
            config.sender.network.port;
        message_config.global_network.network_type =
            config_manager.get_sender_network_type();
        message_config.global_network.interface =
            if config.sender.network.interface.is_empty() {
                None
            } else {
                Some(
                    config.sender.network.interface.clone(),
                )
            };

        let transfer_service =
            TransferService::new(config_manager);
        let message_stats =
            transfer_service.get_message_stats();

        Self {
            message_config,
            transfer_state: TransferState::Idle,
            stats: Arc::new(Mutex::new(
                TransferStats::default(),
            )),
            message_stats,
            shared_state: None,
            runtime_handle: None,
            transfer_service,
        }
    }
}

impl DataTransferApp {
    /// 创建新的应用程序实例
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置跨平台的中文字体支持
        loader::setup_fonts(&cc.egui_ctx);
        Self::default()
    }

    /// 启动消息发送器
    fn start_message_sender(&mut self) {
        // 验证配置
        if self.message_config.messages.is_empty() {
            self.transfer_state = TransferState::Error(
                "No messages configured".to_string(),
            );
            return;
        }

        // 在启动发送前，先同步界面的字段值到配置管理器
        self.sync_field_values_to_config();

        // 检查是否有启用的报文
        let enabled_messages: Vec<_> = self
            .message_config
            .messages
            .iter()
            .filter(|msg| msg.definition.enabled)
            .collect();

        if enabled_messages.is_empty() {
            self.transfer_state = TransferState::Error(
                "No enabled messages found".to_string(),
            );
            return;
        }

        // 验证可编辑字段是否都有值
        for message in &enabled_messages {
            for field in message.get_editable_fields() {
                if field.current_value.trim().is_empty() {
                    self.transfer_state = TransferState::Error(
                        format!("Field '{}' in message '{}' requires a value",
                               field.name, message.definition.name)
                );
                    return;
                }
            }
        }

        // 获取当前的 Tokio runtime handle
        let handle =
            match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => {
                    tracing::error!("Unable to get Tokio runtime handle");
                    self.transfer_state = TransferState::Error(
                    "Runtime handle not initialized".to_string(),
                );
                    return;
                }
            };
        self.runtime_handle = Some(handle.clone());

        // 创建发送器配置
        let sender_config = SenderConfig {
            address: self
                .message_config
                .global_network
                .address
                .clone(),
            port: self.message_config.global_network.port,
            network_type: self
                .message_config
                .global_network
                .network_type,
            interface: self
                .message_config
                .global_network
                .interface
                .clone(),
        };

        // 启动发送器
        match self.transfer_service.start_sender(
            &sender_config,
            Arc::clone(&self.stats),
            &handle,
        ) {
            Ok(shared_state) => {
                self.shared_state = Some(shared_state);
                tracing::info!("Starting message sender with {} enabled messages", enabled_messages.len());

                // 标记消息为发送状态
                for message in
                    &mut self.message_config.messages
                {
                    if message.definition.enabled {
                        message.is_sending = true;
                    }
                }
            }
            Err(e) => {
                self.transfer_state =
                    TransferState::Error(e.to_string());
                tracing::error!(
                    "Failed to start sender: {}",
                    e
                );
            }
        }
    }

    /// 停止发送器
    fn stop_sender(&mut self) {
        tracing::info!("Stopping message sender...");
        TransferService::stop_transfer(&self.shared_state);
        self.transfer_state = TransferState::Idle;

        // 结束所有消息的统计
        if let Ok(mut stats) = self.message_stats.lock() {
            stats.finish_all();
        }

        // 重置消息发送状态
        for message in &mut self.message_config.messages {
            message.is_sending = false;
        }

        tracing::info!("Message sender stopped");
    }

    /// 同步界面字段值到配置管理器（发送前调用）
    fn sync_field_values_to_config(&mut self) {
        // 创建更新后的消息定义，包含界面中的字段值
        let mut updated_messages = Vec::new();

        for runtime_msg in &self.message_config.messages {
            let mut definition =
                runtime_msg.definition.clone();

            // 更新字段定义，将界面的字段值同步到定义中
            for (i, field_def) in
                definition.fields.iter_mut().enumerate()
            {
                if let Some(field_value) =
                    runtime_msg.field_values.get(i)
                {
                    // 如果字段有用户输入的值，更新字段类型定义
                    if field_value.editable
                        && !field_value
                            .current_value
                            .trim()
                            .is_empty()
                    {
                        // 对于可编辑字段，如果用户输入了值，将其作为固定值保存
                        if !field_value
                            .has_function_expression()
                        {
                            field_def.field_type = format!(
                                "{}={}",
                                field_value
                                    .field_type
                                    .split('=')
                                    .next()
                                    .unwrap_or(
                                        &field_value
                                            .field_type
                                    ),
                                field_value
                                    .current_value
                                    .trim()
                            );
                        }
                    }
                }
            }

            updated_messages.push(definition);
        }

        // 更新配置管理器中的消息配置
        self.transfer_service
            .config_manager
            .update_messages(updated_messages);
    }

    /// 保存当前GUI配置到配置管理器
    fn save_current_config(&mut self) {
        // 先同步字段值
        self.sync_field_values_to_config();

        // 保存全局网络配置到发送器配置
        let sender_config = SenderConfig {
            address: self
                .message_config
                .global_network
                .address
                .clone(),
            port: self.message_config.global_network.port,
            network_type: self
                .message_config
                .global_network
                .network_type,
            interface: self
                .message_config
                .global_network
                .interface
                .clone(),
        };
        self.transfer_service
            .config_manager
            .update_sender_config(&sender_config);

        // 保存到文件
        if let Err(e) =
            self.transfer_service.config_manager.save()
        {
            tracing::error!("Failed to save config: {}", e);
        } else {
            tracing::info!(
                "Configuration saved successfully"
            );
        }
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
            &mut self.transfer_state,
            &self.shared_state,
        );

        // 底部状态栏
        egui::TopBottomPanel::bottom("status_panel")
            .resizable(false)
            .exact_height(30.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            // 状态信息
                            match &self.transfer_state {
                                TransferState::Idle => {
                                    ui.colored_label(
                                        egui::Color32::GRAY,
                                        "状态: 空闲",
                                    );
                                }
                                TransferState::Running => {
                                    ui.colored_label(
                                        egui::Color32::GREEN,
                                        "状态: 发送中",
                                    );
                                }
                                TransferState::Error(err) => {
                                    ui.colored_label(
                                        egui::Color32::RED,
                                        format!(
                                            "状态: 错误 - {}",
                                            err
                                        ),
                                    );
                                }
                            }
                        },
                    );

                    ui.with_layout(
                        egui::Layout::right_to_left(
                            egui::Align::Center,
                        ),
                        |ui| {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                format!(
                                    "v{}",
                                    env!(
                                        "CARGO_PKG_VERSION"
                                    )
                                ),
                            );
                        },
                    );
                });
            });

        // 右侧统计信息面板 - 固定宽度，不可拖动
        egui::SidePanel::right("stats_panel")
            .resizable(false)
            .exact_width(300.0)
            .show(ctx, |ui| {
                crate::ui::components::render_stats_panel(
                    ui,
                    &self.message_stats,
                );
            });

        // 主内容区域 - 配置界面
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // 全局网络配置标题
                ui.heading("网络配置");
                ui.separator();
                ui.add_space(5.0);

                // 全局网络配置（固定在顶部，不在滚动区域内）

                let config_enabled = !matches!(self.transfer_state, TransferState::Running);

                crate::ui::components::render_global_network_config(
                    ui,
                    &mut self.message_config.global_network,
                    config_enabled,
                );

                ui.add_space(10.0);

                // 报文配置标题
                ui.heading("报文配置");
                ui.separator();
                ui.add_space(5.0);

                // 报文配置（滚动区域）
                let available_height = ui.available_height() - 60.0; // 为控制按钮预留空间
                crate::ui::components::render_all_messages_config(
                    ui,
                    &mut self.message_config.messages,
                    config_enabled,
                    available_height,
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 控制按钮区域（固定在配置区最下方）
                ui.horizontal(|ui| {
                    let is_running = matches!(self.transfer_state, TransferState::Running);
                    let button_text = if is_running { "停止发送" } else { "开始发送" };

                    if ui.button(button_text).clicked() {
                        if is_running {
                            self.stop_sender();
                        } else {
                            self.start_message_sender();
                        }
                    }
                });

                ui.add_space(10.0)
            });
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
        // 应用退出时保存当前配置
        self.save_current_config();

        // 保存到配置文件
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
    let title = "回放复盘工具";

    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_min_inner_size([800.0, 600.0])
        .with_resizable(true)
        .with_title(title);

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
        title,
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
