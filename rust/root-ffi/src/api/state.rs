// ============================================================
// ROOT v2.0 — api/state.rs
// Глобальная точка доступа к AppState
//
// AppState живёт в root-core.
// Здесь только Arc<Mutex<>> обёртка для Flutter FFI.
// ============================================================

use root_core::AppState;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref APP_STATE: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState::new()));
}

/// Макрос проверки состояния приложения
/// Возвращает InvalidState если текущая фаза не совпадает с ожидаемой
// root-ffi/src/api/state.rs
#[macro_export]
macro_rules! require_state {
    ($($pat:pat_param)|+) => {{
        // ✅ Безопасная блокировка вместо unwrap()
        let state = $crate::api::state::APP_STATE.lock()
            .map_err(|_| $crate::api::types::ApiError::InternalError("mutex poisoned".into()))?;
        
        if !matches!(state.phase, $($pat)|+) {
            return Err($crate::api::types::ApiError::InvalidState(
                format!("Неверное состояние: {:?}", state.phase)
            ));
        }
        // ✅ MutexGuard автоматически освобождается при выходе из блока макроса
    }};
}
