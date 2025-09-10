//! 字体设置模块 - 处理跨平台中文字体支持

use eframe::egui;

/// 设置跨平台的中文字体支持
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 根据不同操作系统加载合适的中文字体
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用微软雅黑
        if let Ok(font_data) =
            std::fs::read("C:\\Windows\\Fonts\\msyh.ttc")
        {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data)
                    .into(),
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
            if let Ok(font_data) = std::fs::read(font_path)
            {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
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
            if let Ok(font_data) = std::fs::read(font_path)
            {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                break;
            }
        }
    }

    // 如果成功加载了中文字体，将其设置为默认字体
    if fonts.font_data.contains_key("chinese_font") {
        if let Some(family) = fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
        {
            family.insert(0, "chinese_font".to_owned())
        }

        if let Some(family) = fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
        {
            family.push("chinese_font".to_owned())
        }
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
