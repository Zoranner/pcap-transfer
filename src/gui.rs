use eframe::egui;
use pcapfile_io::{PcapReader, ReaderConfig};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

use crate::cli::NetworkType;
use crate::config::{AppConfig, OperationConfig};
use crate::error::Result;
use crate::network::UdpSocketFactory;
use crate::stats::TransferStats;
use crate::timing::TimingController;

/// GUI 应用程序状态
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    MainMenu,
    Sender,
    Receiver,
}

/// 发送器配置
#[derive(Debug, Clone)]
pub struct SenderConfig {
    pub dataset_path: PathBuf,
    pub dataset_path_str: String, // 用于 GUI 编辑
    pub address: String,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            dataset_path: PathBuf::new(),
            dataset_path_str: String::new(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            network_type: NetworkType::Unicast,
            interface: None,
        }
    }
}

/// 接收器配置
#[derive(Debug, Clone)]
pub struct ReceiverConfig {
    pub output_path: PathBuf,
    pub output_path_str: String, // 用于 GUI 编辑
    pub dataset_name: String,
    pub address: String,
    pub port: u16,
    pub network_type: NetworkType,
    pub interface: Option<String>,
    pub max_packets: Option<usize>,
}

impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            output_path_str: String::new(),
            dataset_name: "received_data".to_string(),
            address: "0.0.0.0".to_string(),
            port: 8080,
            network_type: NetworkType::Unicast,
            interface: None,
            max_packets: None,
        }
    }
}

/// 传输状态
#[derive(Debug, Clone)]
pub enum TransferState {
    Idle,
    Running,
    Completed,
    Error(String),
}

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
}

impl Default for DataTransferApp {
    fn default() -> Self {
        Self {
            mode: AppMode::Sender,
            sender_config: SenderConfig::default(),
            receiver_config: ReceiverConfig::default(),
            transfer_state: TransferState::Idle,
            stats: Arc::new(Mutex::new(
                TransferStats::default(),
            )),
            runtime_handle: None,
            shared_transfer_state: None,
        }
    }
}

impl DataTransferApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置跨平台的中文字体支持
        Self::setup_fonts(&cc.egui_ctx);
        Self::default()
    }

    /// 设置跨平台的中文字体支持
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 根据不同操作系统加载合适的中文字体
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用微软雅黑
            if let Ok(font_data) = std::fs::read(
                "C:\\Windows\\Fonts\\msyh.ttc",
            ) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 尝试常见的中文字体路径
            let font_paths = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            ];

            for font_path in &font_paths {
                if let Ok(font_data) =
                    std::fs::read(font_path)
                {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        egui::FontData::from_owned(
                            font_data,
                        ),
                    );
                    break;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 使用系统中文字体
            let font_paths = [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Helvetica.ttc",
            ];

            for font_path in &font_paths {
                if let Ok(font_data) =
                    std::fs::read(font_path)
                {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        egui::FontData::from_owned(
                            font_data,
                        ),
                    );
                    break;
                }
            }
        }

        // 如果成功加载了中文字体，将其设置为默认字体
        if fonts.font_data.contains_key("chinese_font") {
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "chinese_font".to_owned());

            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("chinese_font".to_owned());
        }

        ctx.set_fonts(fonts);

        // 设置更大的字体大小以便更好地显示中文
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(
                14.0,
                egui::FontFamily::Proportional,
            ),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(
                14.0,
                egui::FontFamily::Proportional,
            ),
        );
        ctx.set_style(style);
    }

    /// 渲染主菜单
    fn render_main_menu(&mut self, ui: &mut egui::Ui) {
        ui.heading("Data Transfer - 数据包传输测试工具");
        ui.separator();

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if ui.button("📤 发送数据包").clicked() {
                self.mode = AppMode::Sender;
            }

            ui.add_space(20.0);

            if ui.button("📥 接收数据包").clicked() {
                self.mode = AppMode::Receiver;
            }
        });

        ui.add_space(20.0);
        ui.label("选择操作模式开始使用工具");
    }

    /// 渲染发送器界面
    fn render_sender(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← 返回").clicked() {
                self.mode = AppMode::MainMenu;
                self.transfer_state = TransferState::Idle;
            }
            ui.heading("发送数据包");
        });
        ui.separator();

        // 配置区域
        egui::Grid::new("sender_config")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("数据集路径:");
                ui.horizontal(|ui| {
                    if ui
                        .text_edit_singleline(
                            &mut self
                                .sender_config
                                .dataset_path_str,
                        )
                        .changed()
                    {
                        self.sender_config.dataset_path =
                            PathBuf::from(
                                &self
                                    .sender_config
                                    .dataset_path_str,
                            );
                    }
                    if ui.button("浏览").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new()
                                .pick_folder()
                        {
                            self.sender_config
                                .dataset_path =
                                path.clone();
                            self.sender_config
                                .dataset_path_str = path
                                .to_string_lossy()
                                .to_string();
                        }
                    }
                });
                ui.end_row();

                ui.label("目标地址:");
                ui.text_edit_singleline(
                    &mut self.sender_config.address,
                );
                ui.end_row();

                ui.label("目标端口:");
                ui.add(
                    egui::DragValue::new(
                        &mut self.sender_config.port,
                    )
                    .range(1..=65535),
                );
                ui.end_row();

                ui.label("网络类型:");
                egui::ComboBox::from_label("")
                    .selected_text(format!(
                        "{:?}",
                        self.sender_config.network_type
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self
                                .sender_config
                                .network_type,
                            NetworkType::Unicast,
                            "Unicast",
                        );
                        ui.selectable_value(
                            &mut self
                                .sender_config
                                .network_type,
                            NetworkType::Broadcast,
                            "Broadcast",
                        );
                        ui.selectable_value(
                            &mut self
                                .sender_config
                                .network_type,
                            NetworkType::Multicast,
                            "Multicast",
                        );
                    });
                ui.end_row();
            });

        ui.add_space(20.0);

        // 控制按钮
        match self.transfer_state {
            TransferState::Idle => {
                if ui.button("开始发送").clicked() {
                    self.start_sender();
                }
            }
            TransferState::Running => {
                if ui.button("停止发送").clicked() {
                    self.stop_transfer();
                }
            }
            TransferState::Completed => {
                ui.label("✅ 发送完成");
                if ui.button("重新开始").clicked() {
                    self.transfer_state =
                        TransferState::Idle;
                }
            }
            TransferState::Error(ref err) => {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("❌ 错误: {}", err),
                );
                if ui.button("重试").clicked() {
                    self.transfer_state =
                        TransferState::Idle;
                }
            }
        }

        ui.add_space(20.0);

        // 统计信息
        self.render_stats(ui);
    }

    /// 渲染接收器界面
    fn render_receiver(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← 返回").clicked() {
                self.mode = AppMode::MainMenu;
                self.transfer_state = TransferState::Idle;
            }
            ui.heading("接收数据包");
        });
        ui.separator();

        // 配置区域
        egui::Grid::new("receiver_config")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("输出路径:");
                ui.horizontal(|ui| {
                    if ui
                        .text_edit_singleline(
                            &mut self
                                .receiver_config
                                .output_path_str,
                        )
                        .changed()
                    {
                        self.receiver_config.output_path =
                            PathBuf::from(
                                &self
                                    .receiver_config
                                    .output_path_str,
                            );
                    }
                    if ui.button("浏览").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new()
                                .pick_folder()
                        {
                            self.receiver_config
                                .output_path = path.clone();
                            self.receiver_config
                                .output_path_str = path
                                .to_string_lossy()
                                .to_string();
                        }
                    }
                });
                ui.end_row();

                ui.label("数据集名称:");
                ui.text_edit_singleline(
                    &mut self.receiver_config.dataset_name,
                );
                ui.end_row();

                ui.label("监听地址:");
                ui.text_edit_singleline(
                    &mut self.receiver_config.address,
                );
                ui.end_row();

                ui.label("监听端口:");
                ui.add(
                    egui::DragValue::new(
                        &mut self.receiver_config.port,
                    )
                    .range(1..=65535),
                );
                ui.end_row();

                ui.label("网络类型:");
                egui::ComboBox::from_label("")
                    .selected_text(format!(
                        "{:?}",
                        self.receiver_config.network_type
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self
                                .receiver_config
                                .network_type,
                            NetworkType::Unicast,
                            "Unicast",
                        );
                        ui.selectable_value(
                            &mut self
                                .receiver_config
                                .network_type,
                            NetworkType::Broadcast,
                            "Broadcast",
                        );
                        ui.selectable_value(
                            &mut self
                                .receiver_config
                                .network_type,
                            NetworkType::Multicast,
                            "Multicast",
                        );
                    });
                ui.end_row();
            });

        ui.add_space(20.0);

        // 控制按钮
        match self.transfer_state {
            TransferState::Idle => {
                if ui.button("开始接收").clicked() {
                    self.start_receiver();
                }
            }
            TransferState::Running => {
                if ui.button("停止接收").clicked() {
                    self.stop_transfer();
                }
            }
            TransferState::Completed => {
                ui.label("✅ 接收完成");
                if ui.button("重新开始").clicked() {
                    self.transfer_state =
                        TransferState::Idle;
                }
            }
            TransferState::Error(ref err) => {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("❌ 错误: {}", err),
                );
                if ui.button("重试").clicked() {
                    self.transfer_state =
                        TransferState::Idle;
                }
            }
        }

        ui.add_space(20.0);

        // 统计信息
        self.render_stats(ui);
    }

    /// 渲染统计信息
    fn render_stats(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("传输统计");

            if let Ok(stats) = self.stats.lock() {
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
                        ui.label(
                            crate::utils::format_bytes(
                                stats.get_bytes_processed(),
                            ),
                        );
                        ui.end_row();

                        ui.label("传输速率:");
                        ui.label(format!(
                            "{}/s",
                            crate::utils::format_bytes(
                                stats.get_rate_bps() as u64
                                    / 8
                            )
                        ));
                        ui.end_row();

                        ui.label("运行时间:");
                        ui.label(format!(
                            "{:.1}s",
                            stats
                                .get_duration()
                                .as_secs_f64()
                        ));
                        ui.end_row();

                        ui.label("错误数:");
                        ui.label(format!(
                            "{}",
                            stats.get_errors()
                        ));
                        ui.end_row();
                    });
            }
        });
    }

    /// 启动发送器
    fn start_sender(&mut self) {
        if self.sender_config.dataset_path_str.is_empty() {
            self.transfer_state = TransferState::Error(
                "请选择数据集路径".to_string(),
            );
            return;
        }

        // 更新 dataset_path 从字符串
        self.sender_config.dataset_path = PathBuf::from(
            &self.sender_config.dataset_path_str,
        );

        let dataset_path =
            self.sender_config.dataset_path.clone();
        let address = self.sender_config.address.clone();
        let port = self.sender_config.port;
        let network_type =
            self.sender_config.network_type.clone();
        let interface =
            self.sender_config.interface.clone();
        let stats = self.stats.clone();

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        }

        self.transfer_state = TransferState::Running;

        // 在后台运行发送任务
        if let Some(handle) = &self.runtime_handle {
            let transfer_state_ref =
                std::sync::Arc::new(std::sync::Mutex::new(
                    TransferState::Running,
                ));
            let transfer_state_clone =
                transfer_state_ref.clone();

            // 保存共享状态引用
            self.shared_transfer_state =
                Some(transfer_state_ref.clone());

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
                        println!("发送任务完成");
                    }
                    Err(e) => {
                        eprintln!("发送错误: {}", e);
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
                "运行时未初始化".to_string(),
            );
        }
    }

    /// 启动接收器
    fn start_receiver(&mut self) {
        if self.receiver_config.output_path_str.is_empty() {
            self.transfer_state = TransferState::Error(
                "请选择输出路径".to_string(),
            );
            return;
        }

        if self.receiver_config.dataset_name.is_empty() {
            self.transfer_state = TransferState::Error(
                "请输入数据集名称".to_string(),
            );
            return;
        }

        // 更新 output_path 从字符串
        self.receiver_config.output_path = PathBuf::from(
            &self.receiver_config.output_path_str,
        );

        let output_path =
            self.receiver_config.output_path.clone();
        let dataset_name =
            self.receiver_config.dataset_name.clone();
        let address = self.receiver_config.address.clone();
        let port = self.receiver_config.port;
        let network_type =
            self.receiver_config.network_type.clone();
        let interface =
            self.receiver_config.interface.clone();
        let max_packets = self.receiver_config.max_packets;
        let stats = self.stats.clone();

        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = TransferStats::default();
        }

        self.transfer_state = TransferState::Running;

        // 在后台运行接收任务
        if let Some(handle) = &self.runtime_handle {
            handle.spawn(async move {
                match crate::receiver::run_receiver(
                    output_path,
                    dataset_name,
                    address,
                    port,
                    network_type,
                    interface,
                    max_packets,
                )
                .await
                {
                    Ok(_) => {
                        // 接收完成 - 在实际应用中可以通过共享状态通知GUI
                        println!("接收任务完成");
                    }
                    Err(e) => {
                        eprintln!("接收错误: {}", e);
                    }
                }
            });
        }
    }

    /// 停止传输
    fn stop_transfer(&mut self) {
        self.transfer_state = TransferState::Idle;
        // TODO: 实现停止逻辑
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
                    self.render_main_menu(ui)
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
}

/// 启动 GUI 应用程序
/// 专为GUI设计的发送器函数，支持统计信息共享
async fn run_sender_with_gui_stats(
    dataset_path: PathBuf,
    address: String,
    port: u16,
    network_type: NetworkType,
    interface: Option<String>,
    stats: Arc<Mutex<TransferStats>>,
    transfer_state: Arc<Mutex<TransferState>>,
) -> Result<()> {
    // 创建配置
    let config = AppConfig::for_sender(
        dataset_path.clone(),
        address.clone(),
        port,
        network_type.clone(),
        interface,
    )?;

    // 验证配置
    config.validate()?;

    // 创建UDP发送器
    let socket =
        UdpSocketFactory::create_sender(&config.network)
            .await?;

    // 创建pcap读取器
    let dataset_name = dataset_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dataset");

    let mut reader = PcapReader::new_with_config(
        dataset_path.parent().unwrap_or(&dataset_path),
        dataset_name,
        ReaderConfig::default(),
    )?;

    // 获取数据集信息
    let _dataset_info = reader.get_dataset_info()?;

    // 初始化时序控制器
    let mut timing_controller =
        if let OperationConfig::Send {
            timing_enabled,
            max_delay_threshold_ms,
            ..
        } = &config.operation
        {
            if *timing_enabled {
                Some(
                    TimingController::with_delay_threshold(
                        *max_delay_threshold_ms,
                    ),
                )
            } else {
                None
            }
        } else {
            None
        };

    // 重置并初始化统计信息
    if let Ok(mut stats_guard) = stats.lock() {
        *stats_guard = TransferStats::new(None); // GUI不需要进度条
    }

    // 读取并发送数据包
    while let Some(packet) = reader.read_packet()? {
        let packet_data = &packet.data;
        let packet_time = packet.capture_time();

        // 时序控制（如果启用）
        if let Some(controller) = &mut timing_controller {
            controller
                .wait_for_packet_time(packet_time)
                .await;
        }

        // 发送数据包
        match socket
            .send_to(
                packet_data,
                format!(
                    "{}:{}",
                    config.network.address,
                    config.network.port
                ),
            )
            .await
        {
            Ok(bytes_sent) => {
                debug!(
                    "发送数据包: {} 字节, 时间戳: {}",
                    bytes_sent,
                    packet_time.format("%H:%M:%S%.9f")
                );
                // 更新共享的统计信息
                if let Ok(mut stats_guard) = stats.lock() {
                    stats_guard.update(bytes_sent);
                }
            }
            Err(e) => {
                error!("发送数据包失败: {}", e);
                if let Ok(mut stats_guard) = stats.lock() {
                    stats_guard.add_error();
                }
            }
        }
    }

    // 标记统计信息完成并更新传输状态为完成
    if let Ok(mut stats_guard) = stats.lock() {
        stats_guard.finish();
    }
    if let Ok(mut state) = transfer_state.lock() {
        *state = TransferState::Completed;
    }

    Ok(())
}

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
        crate::error::AppError::Gui(e.to_string())
    })?;

    Ok(())
}
