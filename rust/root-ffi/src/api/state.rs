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

/// Удобный макрос для получения лока
#[macro_export]
macro_rules! with_state {
    ($var:ident, $body:block) => {{
        let mut $var = crate::api::state::APP_STATE.lock().unwrap();
        $body
    }};
}
