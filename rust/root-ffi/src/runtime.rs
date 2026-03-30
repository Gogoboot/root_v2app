// ============================================================
// ROOT v2.0 — root-ffi/src/runtime.rs
// Единый Tokio Runtime для всего приложения
// ============================================================

//! # Единый Tokio Runtime
//!
//! Этот модуль создаёт и хранит единственный экземпляр Tokio Runtime
//! для всего приложения. Все асинхронные задачи (P2P, сеть, БД)
//! должны запускаться через `runtime_handle()`, а не создавать
//! собственные Runtime.

use once_cell::sync::Lazy;
use tokio::runtime::{Runtime, Handle};

/// ✅ Единый runtime на всё приложение
pub static APP_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("root-p2p")
        .worker_threads(4)
        .build()
        .expect("Failed to create Tokio runtime for ROOT")
});

/// 🔑 Возвращает `Handle` для запуска асинхронных задач
#[inline]
pub fn runtime_handle() -> Handle {
    APP_RUNTIME.handle().clone()
}
