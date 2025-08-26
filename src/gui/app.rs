//! GUI主应用程序模块

use eframe::egui;
use std::sync::{Arc, Mutex};
use tracing;

use crate::config_manager::ConfigManager;
use crate::error::{AppError, Result};
use crate::receiver::run_receiver_with_gui_stats;
use crate::sender::{
    run_sender_with_gui_stats, TransferState,
};
use crate::stats::TransferStats;

use super::config::{
    AppMode, ReceiverConfig, SenderConfig,
};
use super::{font, renderer};

/// GUI 应用程序
pub struct DataTransferApp {
    mode: AppMode,
    sender_config: SenderConfig,
    receiver_config: ReceiverConfig,
    transfer_state: TransferState,
    stats: Arc<Mutex<TransferStats>>,
    // Tokio runtime handle
    runtime_handle: Option<tokio::runtime::Handle>,
    // 共享的传输状态引用
    shared_transfer_state:
        Option<Arc<Mutex<TransferState>>>,
    // 配置管理器
    config_manager: ConfigManager,
}

impl Default for DataTransferApp {
    fn default() -> Self {
        let mut config_manager =
            ConfigManager::new("config.toml");
        // 尝试加载配置，如果失败则使用默认配置
        if let Err(e) = config_manager.load() {
            tracing::warn!(
                "加载配置文件失败，使用默认配置: {}",
                e
            );
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

        Self {
            mode: AppMode::MainMenu,
            sender_config,
            receiver_config,
            transfer_state: TransferState::Idle,
            stats: Arc::new(Mutex::new(
                TransferStats::default(),
            )),
            runtime_handle: None,
            shared_transfer_state: None,
            config_manager,
        }
    }
}

impl DataTransferApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置跨平台的中文字体支持
        font::setup_fonts(&cc.egui_ctx);
        Self::default()
    }

    /// 渲染发送器界面
    fn render_sender(&mut self, ui: &mut egui::Ui) {
        renderer::render_page_header(
            ui,
            "发送数据包",
            &mut self.mode,
            &mut self.transfer_state,
        );

        // 配置区域
        renderer::render_sender_config(
            ui,
            &mut self.sender_config,
        );
        ui.add_space(20.0);

        // 控制按钮
        let transfer_state = self.transfer_state.clone();
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

        ui.horizontal(|ui| {
            if can_start && ui.button("开始发送").clicked()
            {
                self.start_sender();
            }
            if can_stop && ui.button("停止发送").clicked()
            {
                self.stop_transfer();
            }

            match &transfer_state {
                TransferState::Completed => {
                    ui.label("发送完成");
                }
                TransferState::Error(err) => {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("错误: {}", err),
                    );
                }
                _ => {}
            }
        });

        ui.add_space(20.0);

        // 统计信息
        self.render_stats_safely(ui);
    }

    /// 渲染接收器界面
    fn render_receiver(&mut self, ui: &mut egui::Ui) {
        renderer::render_page_header(
            ui,
            "接收数据包",
            &mut self.mode,
            &mut self.transfer_state,
        );

        // 配置区域
        renderer::render_receiver_config(
            ui,
            &mut self.receiver_config,
        );
        ui.add_space(20.0);

        // 控制按钮
        let transfer_state = self.transfer_state.clone();
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

        ui.horizontal(|ui| {
            if can_start && ui.button("开始接收").clicked()
            {
                self.start_receiver();
            }
            if can_stop && ui.button("停止接收").clicked()
            {
                self.stop_transfer();
            }

            match &transfer_state {
                TransferState::Completed => {
                    ui.label("接收完成");
                }
                TransferState::Error(err) => {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("错误: {}", err),
                    );
                }
                _ => {}
            }
        });

        ui.add_space(20.0);

        // 统计信息
        self.render_stats_safely(ui);
    }

    /// 安全地渲染统计信息
    fn render_stats_safely(&self, ui: &mut egui::Ui) {
        if let Ok(stats) = self.stats.lock() {
            renderer::render_stats(ui, &stats);
        } else {
            // 如果无法获取锁，显示默认统计信息
            let default_stats = TransferStats::default();
            renderer::render_stats(ui, &default_stats);
        }
    }

    /// 启动发送器
    fn start_sender(&mut self) {
        if let Err(e) = self.validate_sender_config() {
            self.transfer_state =
                TransferState::Error(e.to_string());
            return;
        }

        // 保存当前配置到配置管理器
        self.config_manager.update_sender_network_config(
            self.sender_config.address.clone(),
            self.sender_config.port,
            self.sender_config.network_type,
            self.sender_config.interface.clone(),
        );
        self.config_manager.update_sender_config(
            self.sender_config.dataset_path.clone(),
        );
        if let Err(e) = self.config_manager.save() {
            tracing::warn!("保存配置失败: {}", e);
        }

        let dataset_path = std::path::PathBuf::from(
            &self.sender_config.dataset_path,
        );
        let address = self.sender_config.address.clone();
        let port = self.sender_config.port;
        let network_type = self.sender_config.network_type;
        let interface =
            self.sender_config.interface.clone();
        let stats = Arc::clone(&self.stats);

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        } else {
            tracing::error!("无法获取统计信息锁");
            self.transfer_state = TransferState::Error(
                "统计信息初始化失败".to_string(),
            );
            return;
        }

        self.transfer_state = TransferState::Running;

        // 在后台运行发送任务
        if let Some(handle) = &self.runtime_handle {
            let transfer_state_ref = Arc::new(Mutex::new(
                TransferState::Running,
            ));
            let transfer_state_clone =
                Arc::clone(&transfer_state_ref);

            // 保存共享状态引用
            self.shared_transfer_state =
                Some(Arc::clone(&transfer_state_ref));

            handle.spawn(async move {
                match run_sender_with_gui_stats(
                    dataset_path,
                    address,
                    port,
                    network_type,
                    interface,
                    stats,
                    transfer_state_clone,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("发送任务完成");
                    }
                    Err(e) => {
                        tracing::error!(
                            "发送任务失败: {}",
                            e
                        );
                        if let Ok(mut state) =
                            transfer_state_ref.lock()
                        {
                            *state = TransferState::Error(
                                e.to_string(),
                            );
                        }
                    }
                }
            });
        } else {
            self.transfer_state = TransferState::Error(
                "运行时句柄未初始化".to_string(),
            );
        }
    }

    /// 启动接收器
    fn start_receiver(&mut self) {
        if let Err(e) = self.validate_receiver_config() {
            tracing::error!("接收器配置验证失败: {}", e);
            self.transfer_state =
                TransferState::Error(e.to_string());
            return;
        }

        // 保存当前配置到配置管理器
        self.config_manager.update_receiver_network_config(
            self.receiver_config.address.clone(),
            self.receiver_config.port,
            self.receiver_config.network_type,
            self.receiver_config.interface.clone(),
        );
        self.config_manager.update_receiver_config(
            self.receiver_config.output_path.clone(),
            self.receiver_config.dataset_name.clone(),
        );
        if let Err(e) = self.config_manager.save() {
            tracing::warn!("保存配置失败: {}", e);
        }

        // 获取当前的 Tokio runtime handle
        let handle =
            match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => {
                    tracing::error!(
                        "无法获取 Tokio runtime handle"
                    );
                    self.transfer_state =
                        TransferState::Error(
                            "运行时句柄未初始化"
                                .to_string(),
                        );
                    return;
                }
            };
        self.runtime_handle = Some(handle.clone());

        // 创建共享的传输状态
        let shared_state =
            Arc::new(Mutex::new(TransferState::Running));
        self.shared_transfer_state =
            Some(Arc::clone(&shared_state));
        self.transfer_state = TransferState::Running;

        let output_path = std::path::PathBuf::from(
            &self.receiver_config.output_path,
        );
        let dataset_name =
            self.receiver_config.dataset_name.clone();
        let address = self.receiver_config.address.clone();
        let port = self.receiver_config.port;
        let network_type =
            self.receiver_config.network_type;
        let interface =
            self.receiver_config.interface.clone();
        let stats = Arc::clone(&self.stats);

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        }

        self.transfer_state = TransferState::Running;

        // 在后台运行接收任务
        if let Some(handle) = &self.runtime_handle {
            let transfer_state_clone =
                Arc::clone(&shared_state);
            let transfer_state_for_error =
                Arc::clone(&shared_state);

            handle.spawn(async move {
                match run_receiver_with_gui_stats(
                    output_path,
                    dataset_name,
                    address,
                    port,
                    network_type,
                    interface,
                    stats,
                    transfer_state_clone,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("接收任务完成");
                    }
                    Err(e) => {
                        tracing::error!(
                            "接收任务失败: {}",
                            e
                        );
                        if let Ok(mut state) =
                            transfer_state_for_error.lock()
                        {
                            *state = TransferState::Error(
                                e.to_string(),
                            );
                        }
                    }
                }
            });
        } else {
            self.transfer_state = TransferState::Error(
                "运行时句柄未初始化".to_string(),
            );
        }
    }

    /// 停止传输
    fn stop_transfer(&mut self) {
        if let Some(shared_state) =
            &self.shared_transfer_state
        {
            if let Ok(mut state) = shared_state.lock() {
                *state = TransferState::Idle;
            }
        }
        self.transfer_state = TransferState::Idle;
    }

    /// 验证发送器配置
    fn validate_sender_config(&self) -> Result<()> {
        if self.sender_config.dataset_path.is_empty() {
            return Err(AppError::validation(
                "数据集路径",
                "路径不能为空",
            ));
        }

        let dataset_path = std::path::PathBuf::from(
            &self.sender_config.dataset_path,
        );
        if !dataset_path.exists() {
            return Err(AppError::validation(
                "数据集路径",
                "路径不存在",
            ));
        }

        if self.sender_config.address.is_empty() {
            return Err(AppError::validation(
                "目标地址",
                "地址不能为空",
            ));
        }

        Ok(())
    }

    /// 验证接收器配置
    fn validate_receiver_config(&self) -> Result<()> {
        if self.receiver_config.output_path.is_empty() {
            return Err(AppError::validation(
                "输出路径",
                "路径不能为空",
            ));
        }

        if self.receiver_config.dataset_name.is_empty() {
            return Err(AppError::validation(
                "数据集名称",
                "名称不能为空",
            ));
        }

        if self.receiver_config.address.is_empty() {
            return Err(AppError::validation(
                "监听地址",
                "地址不能为空",
            ));
        }

        Ok(())
    }
}

impl eframe::App for DataTransferApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        // 同步共享的传输状态
        if let Some(shared_state) =
            &self.shared_transfer_state
        {
            if let Ok(state) = shared_state.lock() {
                self.transfer_state = state.clone();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::MainMenu => {
                    renderer::render_main_menu(
                        ui,
                        &mut self.mode,
                    )
                }
                AppMode::Sender => self.render_sender(ui),
                AppMode::Receiver => {
                    self.render_receiver(ui)
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
        if let Err(e) = self.config_manager.save() {
            tracing::error!("保存配置文件失败: {}", e);
        }
    }
}

/// 启动 GUI 应用程序
pub fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title(
                "Data Transfer - 数据包传输测试工具",
            ),
        ..Default::default()
    };

    // 获取当前的 tokio runtime handle
    let runtime_handle = tokio::runtime::Handle::current();

    eframe::run_native(
        "Data Transfer",
        options,
        Box::new(move |cc| {
            let mut app = DataTransferApp::new(cc);
            app.runtime_handle = Some(runtime_handle);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| {
        tracing::error!("GUI启动失败: {}", e);
        AppError::Gui(e.to_string())
    })?;

    Ok(())
}
