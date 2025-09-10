//! 应用程序图标模块
//! 负责加载和管理应用程序图标

use eframe::egui;
use std::sync::Arc;

/// 应用程序图标数据
/// ICO格式包含多个分辨率，系统会自动选择合适的尺寸
const ICON_ICO: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit.ico"
);

/// PNG格式图标（作为备用方案）
const ICON_16: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_16.png"
);
const ICON_32: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_32.png"
);
const ICON_48: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_48.png"
);
const ICON_64: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_64.png"
);
const ICON_128: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_128.png"
);
const ICON_256: &[u8] = include_bytes!(
    "../../assets/icons/icons8_in_transit_256.png"
);

/// 创建应用程序图标数据
///
/// 优先使用ICO格式（包含多分辨率），如果失败则使用PNG备用方案
pub fn create_icon_data(
) -> Result<Arc<egui::IconData>, Box<dyn std::error::Error>>
{
    // 首先尝试使用ICO格式（推荐）
    match create_icon_from_ico() {
        Ok(icon) => Ok(icon),
        Err(e) => {
            eprintln!("Warning: Failed to load ICO icon: {}, falling back to PNG", e);
            // 如果ICO加载失败，使用PNG备用方案
            let icon_size = get_optimal_icon_size();
            create_icon_data_with_size(icon_size)
        }
    }
}

/// 根据系统特性选择最优图标尺寸
fn get_optimal_icon_size() -> u32 {
    // Windows 通常使用32x32或48x48
    #[cfg(target_os = "windows")]
    return 48;

    // macOS 通常使用64x64或128x128
    #[cfg(target_os = "macos")]
    return 64;

    // Linux 系统通常使用48x48或64x64
    #[cfg(target_os = "linux")]
    return 48;

    // 其他系统默认使用64x64
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    return 64;
}

/// 从ICO文件创建图标数据
///
/// ICO文件包含多个分辨率，image库会自动选择合适的尺寸
fn create_icon_from_ico(
) -> Result<Arc<egui::IconData>, Box<dyn std::error::Error>>
{
    // 加载ICO文件，image库会自动处理多分辨率
    let image = image::load_from_memory_with_format(
        ICON_ICO,
        image::ImageFormat::Ico,
    )?;
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    let icon_data = egui::IconData {
        rgba: pixels,
        width,
        height,
    };

    Ok(Arc::new(icon_data))
}

/// 创建指定尺寸的图标数据
pub fn create_icon_data_with_size(
    size: u32,
) -> Result<Arc<egui::IconData>, Box<dyn std::error::Error>>
{
    let icon_bytes = match size {
        16 => ICON_16,
        32 => ICON_32,
        48 => ICON_48,
        64 => ICON_64,
        128 => ICON_128,
        256 => ICON_256,
        _ => {
            // 如果请求的尺寸不存在，选择最接近的尺寸
            if size <= 24 {
                ICON_16
            } else if size <= 40 {
                ICON_32
            } else if size <= 56 {
                ICON_48
            } else if size <= 96 {
                ICON_64
            } else if size <= 192 {
                ICON_128
            } else {
                ICON_256
            }
        }
    };

    let image = image::load_from_memory(icon_bytes)?;
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    let icon_data = egui::IconData {
        rgba: pixels,
        width,
        height,
    };

    Ok(Arc::new(icon_data))
}
