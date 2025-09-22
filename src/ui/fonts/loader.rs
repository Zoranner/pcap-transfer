//! 字体设置模块 - 处理跨平台中文字体支持

use egui::{FontDefinitions, FontFamily};

/// 设置跨平台的中文字体支持
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 使用系统字体支持中文
    // Windows 系统字体
    #[cfg(target_os = "windows")]
    {
        // 尝试加载 Windows 系统中文字体
        if let Ok(font_data) =
            std::fs::read("C:/Windows/Fonts/msyh.ttc")
        {
            fonts.font_data.insert(
                "microsoft_yahei".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            // 将微软雅黑设为默认字体
            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "microsoft_yahei".to_owned());
        } else if let Ok(font_data) =
            std::fs::read("C:/Windows/Fonts/simsun.ttc")
        {
            fonts.font_data.insert(
                "simsun".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "simsun".to_owned());
        }
    }

    // Linux 系统字体
    #[cfg(target_os = "linux")]
    {
        // 尝试加载 Linux 系统中文字体
        let font_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];

        for path in &font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                let font_name = format!(
                    "system_font_{}",
                    fonts.font_data.len()
                );
                fonts.font_data.insert(
                    font_name.clone(),
                    egui::FontData::from_owned(font_data),
                );

                fonts
                    .families
                    .get_mut(&FontFamily::Proportional)
                    .unwrap()
                    .insert(0, font_name);
                break;
            }
        }
    }

    // macOS 系统字体
    #[cfg(target_os = "macos")]
    {
        // 尝试加载 macOS 系统中文字体
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Arial Unicode MS.ttf",
        ];

        for path in &font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                let font_name = format!(
                    "system_font_{}",
                    fonts.font_data.len()
                );
                fonts.font_data.insert(
                    font_name.clone(),
                    egui::FontData::from_owned(font_data),
                );

                fonts
                    .families
                    .get_mut(&FontFamily::Proportional)
                    .unwrap()
                    .insert(0, font_name);
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
    tracing::info!(
        "System fonts configured for Chinese display"
    );
}
