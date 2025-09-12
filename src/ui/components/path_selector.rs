//! 路径选择组件 - 包含输入框和浏览按钮的组合组件

use egui;

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
        let button_width = 60.0;
        let spacing = ui.spacing().item_spacing.x;

        // 根据可用空间确定实际宽度
        let actual_width = ui.available_width();
        let input_width =
            actual_width - button_width - spacing;

        ui.horizontal(|ui| {
            // 输入框
            let text_edit =
                if let Some(hint) = &self.hint_text {
                    egui::TextEdit::singleline(self.path)
                        .desired_width(input_width)
                        .hint_text(hint)
                } else {
                    egui::TextEdit::singleline(self.path)
                        .desired_width(input_width)
                };

            let text_response = ui.add(text_edit);

            // 浏览按钮
            if ui.button("Browse").clicked() {
                if let Some(path) =
                    rfd::FileDialog::new().pick_folder()
                {
                    *self.path =
                        path.to_string_lossy().to_string();
                }
            }

            text_response
        })
        .inner
    }
}
