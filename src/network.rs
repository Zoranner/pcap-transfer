use anyhow::{Context, Result};
use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket as TokioUdpSocket;
use tracing::{debug, info, warn};

use crate::cli::NetworkType;
use crate::utils::{is_broadcast_address, is_multicast_address, validate_ip_address};

/// UDP发送器配置
#[derive(Debug, Clone)]
pub struct SenderConfig {
    pub target_address: IpAddr,
    pub target_port: u16,
    pub network_type: NetworkType,
    #[allow(dead_code)]
    pub interface: Option<String>,
}

/// UDP接收器配置
#[derive(Debug, Clone)]
pub struct ReceiverConfig {
    pub bind_address: IpAddr,
    pub bind_port: u16,
    pub network_type: NetworkType,
    #[allow(dead_code)]
    pub interface: Option<String>,
}

/// 创建UDP发送器
pub async fn create_udp_sender(config: SenderConfig) -> Result<TokioUdpSocket> {
    let target_addr = SocketAddr::new(config.target_address, config.target_port);

    // 根据网络类型确定绑定地址
    let bind_addr = match config.network_type {
        NetworkType::Unicast => SocketAddr::new("0.0.0.0".parse()?, 0),
        NetworkType::Broadcast => SocketAddr::new("0.0.0.0".parse()?, 0),
        NetworkType::Multicast => SocketAddr::new("0.0.0.0".parse()?, 0),
    };

    debug!("创建UDP发送器: 绑定={}, 目标={}", bind_addr, target_addr);

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .with_context(|| format!("绑定UDP发送器失败: {bind_addr}"))?;

    // 配置套接字选项
    match config.network_type {
        NetworkType::Broadcast => {
            let std_socket = socket.into_std()?;
            std_socket.set_broadcast(true).context("设置广播选项失败")?;

            info!("已启用UDP广播模式");
            return Ok(TokioUdpSocket::from_std(std_socket)?);
        }
        NetworkType::Multicast => {
            let std_socket = socket.into_std()?;

            if let IpAddr::V4(multicast_addr) = config.target_address {
                if let Some(interface_name) = &config.interface {
                    // 如果指定了接口，尝试加入组播组
                    warn!("多播接口配置需要手动实现: {}", interface_name);
                }

                // 设置组播TTL
                std_socket
                    .set_multicast_ttl_v4(32)
                    .context("设置组播TTL失败")?;

                info!("已配置IPv4组播发送器: {}", multicast_addr);
            } else if let IpAddr::V6(_multicast_addr) = config.target_address {
                warn!("IPv6组播暂不支持");
            }

            return Ok(TokioUdpSocket::from_std(std_socket)?);
        }
        NetworkType::Unicast => {
            info!("已创建单播UDP发送器");
        }
    }

    Ok(socket)
}

/// 创建UDP接收器
pub async fn create_udp_receiver(config: ReceiverConfig) -> Result<TokioUdpSocket> {
    let bind_addr = SocketAddr::new(config.bind_address, config.bind_port);

    debug!("创建UDP接收器: 绑定={}", bind_addr);

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .with_context(|| format!("绑定UDP接收器失败: {bind_addr}"))?;

    // 配置套接字选项
    match config.network_type {
        NetworkType::Broadcast => {
            let std_socket = socket.into_std()?;
            std_socket.set_broadcast(true).context("设置广播选项失败")?;

            info!("已启用UDP广播接收模式");
            return Ok(TokioUdpSocket::from_std(std_socket)?);
        }
        NetworkType::Multicast => {
            let std_socket = socket.into_std()?;

            if let IpAddr::V4(multicast_addr) = config.bind_address {
                // 加入组播组
                let interface_addr = "0.0.0.0".parse()?;
                std_socket
                    .join_multicast_v4(&multicast_addr, &interface_addr)
                    .with_context(|| format!("加入组播组失败: {multicast_addr}"))?;

                info!("已加入IPv4组播组: {}", multicast_addr);
            } else if let IpAddr::V6(_multicast_addr) = config.bind_address {
                warn!("IPv6组播暂不支持");
            }

            return Ok(TokioUdpSocket::from_std(std_socket)?);
        }
        NetworkType::Unicast => {
            info!("已创建单播UDP接收器");
        }
    }

    Ok(socket)
}

/// 验证网络配置
pub fn validate_network_config(address: &str, network_type: &NetworkType) -> Result<IpAddr> {
    let ip_addr = validate_ip_address(address)?;

    match network_type {
        NetworkType::Broadcast => {
            if !is_broadcast_address(&ip_addr) {
                warn!("地址{}可能不是有效的广播地址", address);
            }
        }
        NetworkType::Multicast => {
            if !is_multicast_address(&ip_addr) {
                anyhow::bail!("地址{}不是有效的组播地址", address);
            }
        }
        NetworkType::Unicast => {
            if is_broadcast_address(&ip_addr) || is_multicast_address(&ip_addr) {
                warn!("单播模式使用了特殊地址: {}", address);
            }
        }
    }

    Ok(ip_addr)
}
