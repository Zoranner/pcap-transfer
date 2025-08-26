use anyhow::{Context, Result};
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

/// 验证IP地址格式
pub fn validate_ip_address(
    address: &str,
) -> Result<IpAddr> {
    address.parse().with_context(|| {
        format!("无效的IP地址格式: {address}")
    })
}

/// 验证端口范围
pub fn validate_port(port: u16) -> Result<u16> {
    if port == 0 {
        anyhow::bail!("端口号不能为0");
    }
    Ok(port)
}

/// 检查数据集路径是否存在
pub fn validate_dataset_path(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!(
            "数据集路径不存在: {}",
            path.display()
        );
    }

    if !path.is_dir() {
        anyhow::bail!(
            "数据集路径必须是目录: {}",
            path.display()
        );
    }

    Ok(())
}

/// 创建输出目录（如果不存在）
pub fn ensure_output_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).with_context(
            || {
                format!(
                    "创建输出目录失败: {}",
                    path.display()
                )
            },
        )?;
    }

    if !path.is_dir() {
        anyhow::bail!(
            "输出路径必须是目录: {}",
            path.display()
        );
    }

    Ok(())
}

/// 判断IP地址是否为广播地址
pub fn is_broadcast_address(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => {
            // 255.255.255.255 是广播地址
            *ipv4 == Ipv4Addr::BROADCAST ||
            // 以255结尾的地址可能是网络广播地址
            ipv4.octets()[3] == 255
        }
        IpAddr::V6(_) => false, // IPv6没有广播
    }
}

/// 判断IP地址是否为组播地址
pub fn is_multicast_address(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => ipv4.is_multicast(),
        IpAddr::V6(ipv6) => ipv6.is_multicast(),
    }
}

/// 格式化字节大小
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 格式化速率
pub fn format_rate(bps: f64) -> String {
    if bps >= 1_000_000_000.0 {
        format!("{:.2} Gbps", bps / 1_000_000_000.0)
    } else if bps >= 1_000_000.0 {
        format!("{:.2} Mbps", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.2} Kbps", bps / 1_000.0)
    } else {
        format!("{bps:.2} bps")
    }
}
