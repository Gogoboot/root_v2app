// ============================================================
// ROOT v2.0 — api/utils.rs
// Утилиты: версия, валидация, целостность БД
// ============================================================

use super::state::APP_STATE;
use super::types::ApiError;
use crate::require_state;
use root_core::state::AppPhase;


pub fn get_version() -> String {
    format!("{} ({})", crate::VERSION, crate::BUILD_DATE)
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.verify_integrity()
    .map_err(ApiError::from)
}

pub fn validate_public_key(key: String) -> bool {
    if key.len() != 64 {
        return false;
    }
    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    if let Ok(bytes) = hex::decode(&key) {
        ed25519_dalek::VerifyingKey::from_bytes(
            bytes.as_slice().try_into().unwrap_or(&[0u8; 32])
        ).is_ok()
    } else {
        false
    }
}
