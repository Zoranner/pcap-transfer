//! 应用状态管理模块
//!
//! 负责管理应用程序的状态同步和更新

use crate::core::network::sender::TransferState;
use crate::core::stats::collector::TransferStats;
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

    /// 同步接收器状态
    pub fn sync_receiver_state(
        local_state: &mut TransferState,
        shared_state: &Option<Arc<Mutex<TransferState>>>,
    ) {
        if let Some(shared_state) = shared_state {
            if let Ok(state) = shared_state.lock() {
                *local_state = state.clone();
            }
        }
    }

    /// 安全地渲染发送器统计信息
    pub fn render_sender_stats_safely(
        stats: &Arc<Mutex<TransferStats>>,
        ui: &mut eframe::egui::Ui,
    ) {
        if let Ok(stats_guard) = stats.lock() {
            crate::ui::components::render_stats(
                ui,
                &stats_guard,
            );
        } else {
            // 如果无法获取锁，显示默认统计信息
            let default_stats = TransferStats::default();
            crate::ui::components::render_stats(
                ui,
                &default_stats,
            );
        }
    }

    /// 安全地渲染接收器统计信息
    pub fn render_receiver_stats_safely(
        stats: &Arc<Mutex<TransferStats>>,
        ui: &mut eframe::egui::Ui,
    ) {
        if let Ok(stats_guard) = stats.lock() {
            crate::ui::components::render_stats(
                ui,
                &stats_guard,
            );
        } else {
            // 如果无法获取锁，显示默认统计信息
            let default_stats = TransferStats::default();
            crate::ui::components::render_stats(
                ui,
                &default_stats,
            );
        }
    }
}
