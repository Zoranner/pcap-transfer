//! 标签页相关组件
//!
//! 包含各种标签页组件，如普通标签按钮、标签页容器等。

use eframe::egui;

/// 普通标签按钮组件
/// 用于简单的标签页切换，不包含状态信息
pub struct TabButton {
    label: String,
    is_selected: bool,
}

impl TabButton {
    /// 创建新的标签按钮
    pub fn new(
        label: impl Into<String>,
        is_selected: bool,
    ) -> Self {
        Self {
            label: label.into(),
            is_selected,
        }
    }

    /// 渲染标签按钮
    pub fn show(
        mut self,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        ui.selectable_value(
            &mut self.is_selected,
            true,
            &self.label,
        )
    }
}

/// 标签页容器组件
/// 用于管理多个标签页的显示和切换
pub struct TabContainer {
    selected_tab: usize,
    tabs: Vec<String>,
}

impl TabContainer {
    /// 创建新的标签页容器
    pub fn new(tabs: Vec<String>) -> Self {
        Self {
            selected_tab: 0,
            tabs,
        }
    }

    /// 获取当前选中的标签页索引
    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }

    /// 设置选中的标签页
    pub fn set_selected_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.selected_tab = index;
        }
    }

    /// 渲染标签页容器
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (index, tab) in self.tabs.iter().enumerate()
            {
                if TabButton::new(
                    tab,
                    self.selected_tab == index,
                )
                .show(ui)
                .clicked()
                {
                    self.selected_tab = index;
                }
                ui.add_space(10.0);
            }
        });
    }
}
