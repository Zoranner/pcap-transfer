use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket as TokioUdpSocket;
use tracing::{debug, warn};

use crate::cli::NetworkType;
use crate::config::NetworkConfig;
use crate::error::{DataTransferError, Result};

/// UDP套接字工厂
pub struct UdpSocketFactory;

impl UdpSocketFactory {
    /// 为发送器创建UDP套接字
    pub async fn create_sender(config: &NetworkConfig) -> Result<TokioUdpSocket> {
        create_udp_sender_socket(config).await
    }

    /// 为接收器创建UDP套接字
    pub async fn create_receiver(config: &NetworkConfig) -> Result<TokioUdpSocket> {
        create_udp_receiver_socket(config).await
    }
}

/// 创建UDP发送器（内部函数）
async fn create_udp_sender_socket(config: &NetworkConfig) -> Result<TokioUdpSocket> {
    let target_addr = SocketAddr::new(config.address, config.port);

    // 根据网络类型确定绑定地址
    let bind_addr = match config.network_type {
        NetworkType::Unicast => SocketAddr::new("0.0.0.0".parse().unwrap(), 0),
        NetworkType::Broadcast => SocketAddr::new("0.0.0.0".parse().unwrap(), 0),
        NetworkType::Multicast => SocketAddr::new("0.0.0.0".parse().unwrap(), 0),
    };

    debug!("创建UDP发送器: 绑定={}, 目标={}", bind_addr, target_addr);

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .map_err(|e| DataTransferError::network(format!("绑定UDP发送器失败 {bind_addr}: {e}")))?;

    // 配置套接字选项
    configure_sender_socket(socket, config).await
}

/// 创建UDP接收器（内部函数）
async fn create_udp_receiver_socket(config: &NetworkConfig) -> Result<TokioUdpSocket> {
    let bind_addr = SocketAddr::new(config.address, config.port);

    debug!("创建UDP接收器: 绑定={}", bind_addr);

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .map_err(|e| DataTransferError::network(format!("绑定UDP接收器失败 {bind_addr}: {e}")))?;

    // 配置套接字选项
    configure_receiver_socket(socket, config).await
}

/// 配置发送器套接字
async fn configure_sender_socket(
    socket: TokioUdpSocket,
    config: &NetworkConfig,
) -> Result<TokioUdpSocket> {
    match config.network_type {
        NetworkType::Broadcast => {
            let std_socket = socket.into_std().map_err(DataTransferError::Network)?;
            std_socket
                .set_broadcast(true)
                .map_err(|e| DataTransferError::config(format!("设置广播选项失败: {e}")))?;

            debug!("UDP广播模式已启用");
            Ok(TokioUdpSocket::from_std(std_socket).map_err(DataTransferError::Network)?)
        }
        NetworkType::Multicast => {
            let std_socket = socket.into_std().map_err(DataTransferError::Network)?;

            if let IpAddr::V4(multicast_addr) = config.address {
                if let Some(interface_name) = &config.interface {
                    warn!("组播接口配置需要手动实现: {}", interface_name);
                }

                // 设置组播TTL
                std_socket
                    .set_multicast_ttl_v4(32)
                    .map_err(|e| DataTransferError::config(format!("设置组播TTL失败: {e}")))?;

                debug!("IPv4组播发送器已配置: {}", multicast_addr);
            } else if let IpAddr::V6(_multicast_addr) = config.address {
                warn!("暂不支持IPv6组播");
            }

            Ok(TokioUdpSocket::from_std(std_socket).map_err(DataTransferError::Network)?)
        }
        NetworkType::Unicast => {
            debug!("UDP单播发送器已创建");
            Ok(socket)
        }
    }
}

/// 配置接收器套接字
async fn configure_receiver_socket(
    socket: TokioUdpSocket,
    config: &NetworkConfig,
) -> Result<TokioUdpSocket> {
    match config.network_type {
        NetworkType::Broadcast => {
            let std_socket = socket.into_std().map_err(DataTransferError::Network)?;
            std_socket
                .set_broadcast(true)
                .map_err(|e| DataTransferError::config(format!("设置广播选项失败: {e}")))?;

            debug!("UDP广播接收模式已启用");
            Ok(TokioUdpSocket::from_std(std_socket).map_err(DataTransferError::Network)?)
        }
        NetworkType::Multicast => {
            let std_socket = socket.into_std().map_err(DataTransferError::Network)?;

            if let IpAddr::V4(multicast_addr) = config.address {
                // 加入组播组
                let interface_addr = "0.0.0.0".parse().unwrap();
                std_socket
                    .join_multicast_v4(&multicast_addr, &interface_addr)
                    .map_err(|e| {
                        DataTransferError::network(format!("加入组播组失败 {multicast_addr}: {e}"))
                    })?;

                debug!("已加入IPv4组播组: {}", multicast_addr);
            } else if let IpAddr::V6(_multicast_addr) = config.address {
                warn!("暂不支持IPv6组播");
            }

            Ok(TokioUdpSocket::from_std(std_socket).map_err(DataTransferError::Network)?)
        }
        NetworkType::Unicast => {
            debug!("UDP单播接收器已创建");
            Ok(socket)
        }
    }
}

// 网络配置验证逻辑已移至 config.rs 模块
