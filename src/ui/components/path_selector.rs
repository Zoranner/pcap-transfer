//! 路径选择组件 - 包含输入框和浏览按钮的组合组件

use eframe::egui;
use egui_extras::{Size, StripBuilder};

/// 路径选择组件
/// 包含文本输入框和浏览按钮的组合
pub struct PathSelector<'a> {
    path: &'a mut String,
    hint_text: Option<String>,
}

impl<'a> PathSelector<'a> {
    /// 创建新的路径选择组件
    pub fn new(path: &'a mut String) -> Self {
        Self {
            path,
            hint_text: None,
        }
    }

    /// 设置提示文本
    #[allow(dead_code)]
    pub fn hint_text(
        mut self,
        hint: impl Into<String>,
    ) -> Self {
        self.hint_text = Some(hint.into());
        self
    }
}

impl<'a> egui::Widget for PathSelector<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut text_response = None;

        StripBuilder::new(ui)
            .size(Size::remainder()) // 输入框占用剩余空间
            .size(Size::exact(50.0)) // 按钮固定50px宽度
            .horizontal(|mut strip| {
                // 输入框
                strip.cell(|ui| {
                    let text_edit = if let Some(hint) =
                        &self.hint_text
                    {
                        egui::TextEdit::singleline(
                            self.path,
                        )
                        .hint_text(hint)
                    } else {
                        egui::TextEdit::singleline(
                            self.path,
                        )
                    };
                    text_response = Some(ui.add(text_edit));
                });

                // 浏览按钮
                strip.cell(|ui| {
                    if ui.button("Browse").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new()
                                .pick_folder()
                        {
                            *self.path = path
                                .to_string_lossy()
                                .to_string();
                        }
                    }
                });
            });

        text_response.unwrap_or_else(|| {
            ui.allocate_response(
                egui::Vec2::ZERO,
                egui::Sense::hover(),
            )
        })
    }
}
