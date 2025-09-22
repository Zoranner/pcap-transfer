//! 网络通信模块

/// 传输状态枚举
#[derive(Debug, Clone)]
pub enum TransferState {
    /// 空闲状态
    Idle,
    /// 运行中
    Running,
    /// 错误状态
    Error(String),
}
