// ============================================================
// ROOT v2.0 — api/contacts.rs
// FFI функции: управление контактами
// ============================================================

use root_storage::{Contact, StorageError};
use super::messaging::now_secs;
use super::state::APP_STATE;
use super::types::ApiError;

pub fn add_contact(public_key: String, nickname: String) -> Result<(), ApiError> {
    if !super::utils::validate_public_key(public_key.clone()) {
        return Err(ApiError::InvalidInput(
            "Неверный публичный ключ — должен быть 64 символа hex".to_string(),
        ));
    }
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.add_contact(&Contact {
        public_key,
        nickname,
        added_at: now_secs(),
        reputation: 50,
    })
    .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))
}

pub fn get_contacts() -> Result<Vec<Contact>, ApiError> {
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.get_contacts()
        .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))
}
