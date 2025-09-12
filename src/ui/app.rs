//! GUI主应用程序模块

use egui;
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
    pub fn new(ctx: &egui::Context) -> Self {
        // 配置跨平台的中文字体支持
        loader::setup_fonts(ctx);
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

impl DataTransferApp {
    /// 更新应用状态和渲染UI
    pub fn update(&mut self, ctx: &egui::Context) {
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
        ctx.request_repaint();
    }

    /// 应用退出时保存配置
    pub fn on_exit(&mut self) {
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
    use egui_sdl2_gl::{
        gl, sdl2, translate_virtual_key_code,
    };
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;

    // 初始化 SDL2
    let sdl_context = sdl2::init().map_err(|e| {
        tracing::error!("Failed to initialize SDL2: {}", e);
        AppError::Gui(format!(
            "SDL2 initialization failed: {}",
            e
        ))
    })?;

    let video_subsystem =
        sdl_context.video().map_err(|e| {
            tracing::error!(
                "Failed to initialize SDL2 video: {}",
                e
            );
            AppError::Gui(format!(
                "SDL2 video initialization failed: {}",
                e
            ))
        })?;

    // 创建窗口
    let mut window = video_subsystem
        .window("Pcap Transfer", 400, 500)
        .opengl()
        .build()
        .map_err(|e| {
            tracing::error!(
                "Failed to create window: {}",
                e
            );
            AppError::Gui(format!(
                "Window creation failed: {}",
                e
            ))
        })?;

    // 锁定窗口尺寸，防止用户调整大小和最大化
    let (initial_w, initial_h) = window.size();
    window.set_resizable(false);
    let _ = window.set_minimum_size(initial_w, initial_h);
    let _ = window.set_maximum_size(initial_w, initial_h);

    // 创建 OpenGL 上下文
    let _gl_context =
        window.gl_create_context().map_err(|e| {
            tracing::error!(
                "Failed to create OpenGL context: {}",
                e
            );
            AppError::Gui(format!(
                "OpenGL context creation failed: {}",
                e
            ))
        })?;

    // 加载 OpenGL 函数
    gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const _
    });

    // 创建 egui 上下文
    let egui_ctx = egui::Context::default();

    // 创建 painter
    let mut painter = egui_sdl2_gl::painter::Painter::new(
        &window,
        1.0,
        egui_sdl2_gl::ShaderVersion::Default,
    );

    // 获取当前的 tokio runtime handle
    let runtime_handle = tokio::runtime::Handle::current();

    // 创建应用实例
    let mut app = DataTransferApp::new(&egui_ctx);
    app.runtime_handle = Some(runtime_handle);

    // 获取初始窗口大小
    let (window_width, window_height) = window.size();

    // 创建输入状态
    let mut input_state = egui::RawInput::default();
    input_state.screen_rect =
        Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                window_width as f32,
                window_height as f32,
            ),
        ));

    // 事件循环
    let mut event_pump =
        sdl_context.event_pump().map_err(|e| {
            tracing::error!(
                "Failed to create event pump: {}",
                e
            );
            AppError::Gui(format!(
                "Event pump creation failed: {}",
                e
            ))
        })?;

    let start_time = std::time::Instant::now();

    'running: loop {
        // 重置输入事件
        input_state.events.clear();

        // 设置时间信息
        input_state.time =
            Some(start_time.elapsed().as_secs_f64());

        // 处理 SDL2 事件
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    app.on_exit();
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    app.on_exit();
                    break 'running;
                }
                Event::Window {
                    win_event:
                        sdl2::event::WindowEvent::Resized(
                            width,
                            height,
                        ),
                    ..
                } => {
                    // 更新屏幕尺寸
                    input_state.screen_rect =
                        Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(
                                width as f32,
                                height as f32,
                            ),
                        ));

                    // 更新 OpenGL 视口
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn,
                    x,
                    y,
                    ..
                } => {
                    let pos =
                        egui::Pos2::new(x as f32, y as f32);
                    let button = match mouse_btn {
                        sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
                        sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
                        sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
                        _ => continue,
                    };
                    input_state.events.push(
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: true,
                            modifiers:
                                egui::Modifiers::default(),
                        },
                    );
                }
                Event::MouseButtonUp {
                    mouse_btn,
                    x,
                    y,
                    ..
                } => {
                    let pos =
                        egui::Pos2::new(x as f32, y as f32);
                    let button = match mouse_btn {
                        sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
                        sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
                        sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
                        _ => continue,
                    };
                    input_state.events.push(
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed: false,
                            modifiers:
                                egui::Modifiers::default(),
                        },
                    );
                }
                Event::MouseMotion { x, y, .. } => {
                    input_state.events.push(
                        egui::Event::PointerMoved(
                            egui::Pos2::new(
                                x as f32, y as f32,
                            ),
                        ),
                    );
                }
                Event::TextInput { text, .. } => {
                    input_state
                        .events
                        .push(egui::Event::Text(text));
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) =
                        translate_virtual_key_code(keycode)
                    {
                        input_state.events.push(egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::default(),
                        });
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) =
                        translate_virtual_key_code(keycode)
                    {
                        input_state.events.push(egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers: egui::Modifiers::default(),
                        });
                    }
                }
                _ => {}
            }
        }

        let full_output =
            egui_ctx.run(input_state.take(), |ctx| {
                // 更新应用
                app.update(ctx);
            });

        // 清除屏幕
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // 渲染 egui
        let clipped_primitives = egui_ctx.tessellate(
            full_output.shapes,
            full_output.pixels_per_point,
        );
        painter.paint_jobs(
            None,
            full_output.textures_delta,
            clipped_primitives,
        );

        // 交换窗口缓冲区
        window.gl_swap_window();

        // 控制帧率
        std::thread::sleep(std::time::Duration::new(
            0,
            1_000_000_000u32 / 60,
        ));
    }

    Ok(())
}
