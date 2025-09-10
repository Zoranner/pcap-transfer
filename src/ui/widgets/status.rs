//! 状态相关组件
//!
//! 包含状态指示器、状态标签按钮等与状态显示相关的组件。

use crate::core::network::sender::TransferState;
use eframe::egui;

/// 状态标签按钮组件
/// 结合了标签页切换和状态显示功能
pub struct StatusTabButton {
    label: String,
    state: TransferState,
    is_selected: bool,
}

impl StatusTabButton {
    /// 创建新的状态标签按钮
    pub fn new(
        label: impl Into<String>,
        state: TransferState,
        is_selected: bool,
    ) -> Self {
        Self {
            label: label.into(),
            state,
            is_selected,
        }
    }

    /// 渲染状态标签按钮
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let button_size =
            egui::Vec2::new(ui.available_width(), 30.0);
        let button_rect = egui::Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            button_size,
        );

        // 创建按钮响应
        let response = ui.allocate_rect(
            button_rect,
            egui::Sense::click(),
        );

        // 绘制按钮背景
        let bg_color = if self.is_selected {
            ui.style().visuals.selection.bg_fill
        } else {
            ui.style().visuals.window_fill()
        };

        let stroke = if self.is_selected {
            ui.style().visuals.selection.stroke
        } else {
            egui::Stroke::new(
                1.0,
                ui.style().visuals.window_stroke.color,
            )
        };

        ui.painter().rect_filled(
            button_rect,
            4.0,
            bg_color,
        );
        ui.painter().rect_stroke(button_rect, 4.0, stroke);

        // 绘制按钮文本
        let text_color = if self.is_selected {
            ui.style().visuals.selection.stroke.color
        } else {
            ui.style().visuals.text_color()
        };

        let text_rect = egui::Rect::from_center_size(
            button_rect.center(),
            egui::Vec2::new(
                button_rect.width() - 20.0,
                button_rect.height(),
            ),
        );

        ui.painter().text(
            text_rect.min
                + egui::Vec2::new(
                    8.0,
                    text_rect.height() * 0.5,
                ),
            egui::Align2::LEFT_CENTER,
            &self.label,
            ui.style().text_styles
                [&egui::TextStyle::Button]
                .clone(),
            text_color,
        );

        // 绘制状态指示圆点
        let dot_radius = 4.0;
        let dot_pos = egui::pos2(
            button_rect.right() - dot_radius - 6.0,
            button_rect.center().y,
        );

        let dot_color = match self.state {
            TransferState::Idle => egui::Color32::GRAY,
            TransferState::Running => egui::Color32::GREEN,
            TransferState::Completed => egui::Color32::GRAY,
            TransferState::Error(_) => egui::Color32::RED,
        };

        ui.painter()
            .circle_filled(dot_pos, dot_radius, dot_color);

        response
    }
}
