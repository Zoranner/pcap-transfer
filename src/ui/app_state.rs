//! 应用状态管理模块
//!
//! 负责管理应用程序的状态同步和更新

use crate::core::network::TransferState;
use std::sync::{Arc, Mutex};

/// 应用状态管理器
pub struct AppStateManager;

impl AppStateManager {
    /// 同步发送器状态
    pub fn sync_sender_state(
        local_state: &mut TransferState,
        shared_state: &Option<Arc<Mutex<TransferState>>>,
    ) {
        if let Some(shared_state) = shared_state {
            if let Ok(state) = shared_state.lock() {
                *local_state = state.clone();
            }
        }
    }
}
