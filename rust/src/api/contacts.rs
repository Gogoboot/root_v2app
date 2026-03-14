// ============================================================
// ROOT v2.0 — api/contacts.rs
//
// FFI функции: управление контактами
// ============================================================

use crate::storage::Contact;

use super::state::CURRENT_DB;
use super::types::ApiError;
use super::messaging::now_secs;

pub fn add_contact(public_key: String, nickname: String) -> Result<(), ApiError> {
    // Валидация публичного ключа
    if !super::utils::validate_public_key(public_key.clone()) {
        return Err(ApiError::InvalidInput(
            "Неверный публичный ключ — должен быть 64 символа hex".to_string(),
        ));
    }

    let db_guard = CURRENT_DB.lock().unwrap();
    let db       = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;

    db.add_contact(&Contact {
        public_key,
        nickname,
        added_at:   now_secs(),
        reputation: 50,
    })
    .map_err(|e| ApiError::StorageError(e.to_string()))
}

pub fn get_contacts() -> Result<Vec<Contact>, ApiError> {
    let db_guard = CURRENT_DB.lock().unwrap();
    let db       = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.get_contacts()
        .map_err(|e| ApiError::StorageError(e.to_string()))
}
