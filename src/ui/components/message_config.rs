//! 多报文配置界面组件

use crate::app::config::message_types::MessageRuntimeState;
use crate::app::config::types::NetworkType;
use egui;

/// 全局网络配置结构（用于UI）
#[derive(Debug, Clone)]
pub struct GlobalNetworkConfig {
    /// 目标地址
    pub address: String,
    /// 目标端口
    pub port: u16,
    /// 网络类型
    pub network_type: NetworkType,
    /// 网络接口
    pub interface: Option<String>,
}

impl Default for GlobalNetworkConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
            network_type: NetworkType::Unicast,
            interface: None,
        }
    }
}

/// 渲染全局网络配置
pub fn render_global_network_config(
    ui: &mut egui::Ui,
    config: &mut GlobalNetworkConfig,
    enabled: bool,
) {
    egui::Grid::new("global_network_config_grid")
        .num_columns(2)
        .min_col_width(80.0)
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("目标地址");
            ui.add_enabled(
                enabled,
                egui::TextEdit::singleline(
                    &mut config.address,
                )
                .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("目标端口");
            ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut config.port)
                    .range(1..=65535),
            );
            ui.end_row();

            ui.label("网络类型");
            ui.add_enabled_ui(enabled, |ui| {
                egui::ComboBox::from_id_salt(
                    "global_network_type_combo",
                )
                .selected_text(format!(
                    "{:?}",
                    config.network_type
                ))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut config.network_type,
                        NetworkType::Unicast,
                        "Unicast",
                    );
                    ui.selectable_value(
                        &mut config.network_type,
                        NetworkType::Multicast,
                        "Multicast",
                    );
                    ui.selectable_value(
                        &mut config.network_type,
                        NetworkType::Broadcast,
                        "Broadcast",
                    );
                });
            });
            ui.end_row();
        });
}

/// 渲染单个报文的配置
pub fn render_single_message_config(
    ui: &mut egui::Ui,
    message: &mut MessageRuntimeState,
    enabled: bool,
) {
    // 报文基本信息
    ui.horizontal(|ui| {
        ui.label("启用");
        ui.add_enabled(
            enabled,
            egui::Checkbox::without_text(
                &mut message.definition.enabled,
            ),
        );

        ui.add_space(20.0);

        ui.label("间隔");
        ui.add_enabled(
            enabled,
            egui::DragValue::new(
                &mut message.definition.interval,
            )
            .range(1..=60000)
            .suffix(" ms"),
        );

        ui.add_space(20.0);

        ui.label("包数量");
        ui.add_enabled(
            enabled,
            egui::DragValue::new(
                &mut message.definition.packet_count,
            )
            .range(0..=999999)
            .suffix(""),
        );

        ui.add_space(20.0);
    });

    ui.add_space(10.0);

    // 可编辑字段配置
    let message_name_for_grid =
        message.definition.name.clone();
    let editable_fields = message.get_editable_fields_mut();

    if editable_fields.is_empty() {
        ui.label("此消息中没有可编辑的字段。");
    } else {
        ui.strong("字段参数");
        ui.separator();
        ui.add_space(5.0);

        egui::Grid::new(format!(
            "message_fields_{}",
            message_name_for_grid
        ))
        .num_columns(3)
        .min_col_width(80.0)
        .max_col_width(ui.available_width() - 100.0) // 确保Grid撑满可用宽度
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            // 表头
            ui.strong("名称");
            ui.strong("类型");
            ui.strong("值");
            ui.end_row();

            // 字段行
            for field in editable_fields {
                ui.label(&field.name);
                ui.label(&field.field_type);

                // 值列 - 使用 with_layout 和 allocate_response 撑满剩余空间
                ui.with_layout(
                    egui::Layout::left_to_right(
                        egui::Align::Center,
                    ),
                    |ui| {
                        ui.add_enabled(
                            enabled,
                            |ui: &mut egui::Ui| {
                                render_field_input(
                                    ui, field,
                                )
                            },
                        );
                        // 占用剩余空间
                        ui.allocate_response(
                            egui::Vec2::new(
                                ui.available_width(),
                                0.0,
                            ),
                            egui::Sense::hover(),
                        );
                    },
                );

                ui.end_row();
            }
        });
    }
}

/// 渲染所有消息的配置（滚动区域版本）
pub fn render_all_messages_config(
    ui: &mut egui::Ui,
    messages: &mut [MessageRuntimeState],
    enabled: bool,
    available_height: f32,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .min_scrolled_height(available_height)
        .max_height(available_height)
        .show(ui, |ui| {
            if messages.is_empty() {
                ui.label(
                    "未配置任何消息。请加载配置文件。",
                );
            } else {
                // 显示多个报文的配置
                for message in messages.iter_mut() {
                    let message_name =
                        message.definition.name.clone();
                    let message_enabled =
                        message.definition.enabled;

                    egui::CollapsingHeader::new(
                        egui::RichText::new(&message_name)
                            .strong()
                            .color(if message_enabled {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::GRAY
                            }),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        render_single_message_config(
                            ui, message, enabled,
                        );
                    });

                    ui.add_space(5.0);
                }
            }
        });
}

/// 根据字段类型渲染合适的输入控件
pub fn render_field_input(
    ui: &mut egui::Ui,
    field: &mut crate::app::config::message_types::FieldValue,
) -> egui::Response {
    // 如果字段不应该允许用户输入（包含函数表达式或不可编辑），不显示任何内容
    if !field.should_allow_input() {
        return ui.label("");
    }

    let base_type = extract_base_type(&field.field_type);

    match base_type.as_str() {
        // 整数类型
        "i8" => {
            let mut value: i8 =
                field.current_value.parse().unwrap_or(0);
            let response = ui.add(
                egui::DragValue::new(&mut value)
                    .range(-128..=127),
            );
            field.current_value = value.to_string();
            response
        }
        "i16" => {
            let mut value: i16 =
                field.current_value.parse().unwrap_or(0);
            let response = ui.add(
                egui::DragValue::new(&mut value)
                    .range(-32768..=32767),
            );
            field.current_value = value.to_string();
            response
        }
        "i32" => {
            let mut value: i32 =
                field.current_value.parse().unwrap_or(0);
            let response =
                ui.add(egui::DragValue::new(&mut value));
            field.current_value = value.to_string();
            response
        }
        "i64" => {
            let mut value: i64 =
                field.current_value.parse().unwrap_or(0);
            let response =
                ui.add(egui::DragValue::new(&mut value));
            field.current_value = value.to_string();
            response
        }
        "u8" => {
            let mut value: u8 =
                field.current_value.parse().unwrap_or(0);
            let response = ui.add(
                egui::DragValue::new(&mut value)
                    .range(0..=255),
            );
            field.current_value = value.to_string();
            response
        }
        "u16" => {
            let mut value: u16 =
                field.current_value.parse().unwrap_or(0);
            let response = ui.add(
                egui::DragValue::new(&mut value)
                    .range(0..=65535),
            );
            field.current_value = value.to_string();
            response
        }
        "u32" => {
            let mut value: u32 =
                field.current_value.parse().unwrap_or(0);
            let response =
                ui.add(egui::DragValue::new(&mut value));
            field.current_value = value.to_string();
            response
        }
        "u64" => {
            let mut value: u64 =
                field.current_value.parse().unwrap_or(0);
            let response =
                ui.add(egui::DragValue::new(&mut value));
            field.current_value = value.to_string();
            response
        }
        // 浮点类型
        "f32" => {
            let mut value: f32 =
                field.current_value.parse().unwrap_or(0.0);
            let response = ui.add(
                egui::DragValue::new(&mut value).speed(0.1),
            );
            field.current_value = value.to_string();
            response
        }
        "f64" => {
            let mut value: f64 =
                field.current_value.parse().unwrap_or(0.0);
            let response = ui.add(
                egui::DragValue::new(&mut value).speed(0.1),
            );
            field.current_value = value.to_string();
            response
        }
        // 布尔类型
        "bool" => {
            let mut value: bool = field
                .current_value
                .parse()
                .unwrap_or(false);
            let response = ui.checkbox(&mut value, "");
            field.current_value = value.to_string();
            response
        }
        // 十六进制类型
        "hex" => ui.add(
            egui::TextEdit::singleline(
                &mut field.current_value,
            )
            .hint_text("0x...")
            .desired_width(150.0),
        ),
        _ if base_type.starts_with("hex_") => ui.add(
            egui::TextEdit::singleline(
                &mut field.current_value,
            )
            .hint_text("0x...")
            .desired_width(150.0),
        ),
        // 默认文本输入
        _ => ui.add(
            egui::TextEdit::singleline(
                &mut field.current_value,
            )
            .hint_text("Enter value...")
            .desired_width(150.0),
        ),
    }
}

/// 从字段类型中提取基础类型（去除默认值部分）
fn extract_base_type(field_type: &str) -> String {
    if let Some(eq_pos) = field_type.find('=') {
        field_type[..eq_pos].to_string()
    } else {
        field_type.to_string()
    }
}
