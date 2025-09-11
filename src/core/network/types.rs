use socket2::Socket;
use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket as TokioUdpSocket;
use tracing::{debug, warn};

use crate::app::config::types::{
    NetworkConfig, NetworkType,
};
use crate::app::error::types::{DataTransferError, Result};

/// UDP套接字工厂
pub struct UdpSocketFactory;

impl UdpSocketFactory {
    /// 为发送器创建UDP套接字
    pub async fn create_sender(
        config: &NetworkConfig,
    ) -> Result<TokioUdpSocket> {
        create_udp_sender_socket(config).await
    }

    /// 为接收器创建UDP套接字
    pub async fn create_receiver(
        config: &NetworkConfig,
    ) -> Result<TokioUdpSocket> {
        create_udp_receiver_socket(config).await
    }
}

/// 创建UDP发送器（内部函数）
async fn create_udp_sender_socket(
    config: &NetworkConfig,
) -> Result<TokioUdpSocket> {
    let target_addr =
        SocketAddr::new(config.address, config.port);

    // 根据网络类型确定绑定地址
    let bind_addr = match config.network_type {
        NetworkType::Unicast => {
            SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
        }
        NetworkType::Broadcast => {
            SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
        }
        NetworkType::Multicast => {
            SocketAddr::new("0.0.0.0".parse().unwrap(), 0)
        }
    };

    debug!(
        "Creating UDP sender: bind={}, target={}",
        bind_addr, target_addr
    );

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .map_err(|e| {
            DataTransferError::network(format!(
                "Failed to bind UDP sender {bind_addr}: {e}"
            ))
        })?;

    // 配置套接字选项
    configure_sender_socket(socket, config).await
}

/// 创建UDP接收器（内部函数）
async fn create_udp_receiver_socket(
    config: &NetworkConfig,
) -> Result<TokioUdpSocket> {
    let bind_addr =
        SocketAddr::new(config.address, config.port);

    debug!("Creating UDP receiver: bind={}", bind_addr);

    let socket = TokioUdpSocket::bind(bind_addr)
        .await
        .map_err(|e| {
            DataTransferError::network(format!(
                "Failed to bind UDP receiver {bind_addr}: {e}"
            ))
        })?;

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
            let std_socket = socket
                .into_std()
                .map_err(DataTransferError::Network)?;
            std_socket.set_broadcast(true).map_err(
                |e| {
                    DataTransferError::config(format!(
                        "Failed to set broadcast option: {e}"
                    ))
                },
            )?;

            debug!("UDP broadcast mode enabled");
            Ok(TokioUdpSocket::from_std(std_socket)
                .map_err(DataTransferError::Network)?)
        }
        NetworkType::Multicast => {
            let std_socket = socket
                .into_std()
                .map_err(DataTransferError::Network)?;

            if let IpAddr::V4(multicast_addr) =
                config.address
            {
                if let Some(interface_name) =
                    &config.interface
                {
                    warn!(
                        "Multicast interface configuration requires manual implementation: {}",
                        interface_name
                    );
                }

                // 设置组播TTL
                std_socket
                    .set_multicast_ttl_v4(32)
                    .map_err(|e| {
                        DataTransferError::config(format!(
                            "Failed to set multicast TTL: {e}"
                        ))
                    })?;

                debug!(
                    "IPv4 multicast sender configured: {}",
                    multicast_addr
                );
            } else if let IpAddr::V6(_multicast_addr) =
                config.address
            {
                warn!("IPv6 multicast not supported yet");
            }

            Ok(TokioUdpSocket::from_std(std_socket)
                .map_err(DataTransferError::Network)?)
        }
        NetworkType::Unicast => {
            debug!("UDP unicast sender created");
            Ok(socket)
        }
    }
}

/// 配置接收器套接字
async fn configure_receiver_socket(
    socket: TokioUdpSocket,
    config: &NetworkConfig,
) -> Result<TokioUdpSocket> {
    // 转换为标准socket进行配置
    let std_socket = socket
        .into_std()
        .map_err(DataTransferError::Network)?;

    // 设置接收缓冲区大小以减少丢包
    let socket2 = Socket::from(std_socket);
    if let Err(e) =
        socket2.set_recv_buffer_size(2 * 1024 * 1024)
    {
        warn!("Failed to set receive buffer size: {}", e);
    }
    let std_socket: std::net::UdpSocket = socket2.into();

    match config.network_type {
        NetworkType::Broadcast => {
            std_socket.set_broadcast(true).map_err(
                |e| {
                    DataTransferError::config(format!(
                        "Failed to set broadcast option: {e}"
                    ))
                },
            )?;
            debug!("UDP broadcast receive mode enabled");
        }
        NetworkType::Multicast => {
            if let IpAddr::V4(multicast_addr) =
                config.address
            {
                // 加入组播组
                let interface_addr =
                    "0.0.0.0".parse().unwrap();
                std_socket
                    .join_multicast_v4(&multicast_addr, &interface_addr)
                    .map_err(|e| {
                        DataTransferError::network(format!("Failed to join multicast group {multicast_addr}: {e}"))
                    })?;

                debug!(
                    "Joined IPv4 multicast group: {}",
                    multicast_addr
                );
            } else if let IpAddr::V6(_multicast_addr) =
                config.address
            {
                warn!("IPv6 multicast not supported yet");
            }
        }
        NetworkType::Unicast => {
            debug!("UDP unicast receiver created");
        }
    }

    // 统一转换回tokio socket
    TokioUdpSocket::from_std(std_socket)
        .map_err(DataTransferError::Network)
}

// 网络配置验证逻辑已移至 config.rs 模块
