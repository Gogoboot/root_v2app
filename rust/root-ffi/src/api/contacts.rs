// ============================================================
// ROOT v2.0 — api/contacts.rs
// FFI функции: управление контактами
// ============================================================

use super::messaging::now_secs;
use super::state::APP_STATE;
use super::types::{ApiError, ContactInfo};
use crate::require_state;
use root_core::state::AppPhase;
use root_storage::Contact;

pub fn add_contact(public_key: String, nickname: String) -> Result<(), ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
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
    .map_err(ApiError::from)
}

pub fn get_contacts() -> Result<Vec<ContactInfo>, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    let contacts = db.get_contacts().map_err(ApiError::from)?;
    Ok(contacts
        .into_iter()
        .map(|c| ContactInfo {
            public_key: c.public_key,
            nickname: c.nickname,
            added_at: c.added_at,
            reputation: c.reputation,
        })
        .collect())
}

