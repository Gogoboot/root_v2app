// ============================================================
// ROOT v2.0 — api/messaging.rs
// FFI функции: отправка и получение сообщений
// ============================================================

use crate::storage::{Message, StorageError};
use std::time::{SystemTime, UNIX_EPOCH};

use super::identity::get_public_key;
use super::state::CURRENT_DB;
use super::types::{ApiError, MessageInfo};

pub fn send_message(to_key: String, content: String) -> Result<u64, ApiError> {
    let from_key = get_public_key()?;
    let msg = Message::new(from_key, to_key, content);

    let mut db_guard = CURRENT_DB.lock().unwrap();
    let db = db_guard.as_mut().ok_or(ApiError::DatabaseNotOpen)?;

    let id = db
        .save_message(msg)
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    println!("  ✉️  Сообщение #{} сохранено", id);
    Ok(id)
}

pub fn get_messages() -> Result<Vec<MessageInfo>, ApiError> {
    let public_key = get_public_key()?;

    let db_guard = CURRENT_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;

    let messages = db
        .get_messages(&public_key)
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    let contacts = db.get_contacts().unwrap_or_default();

    let infos = messages
        .into_iter()
        .map(|m| {
            let from_name = contacts
                .iter()
                .find(|c| c.public_key == m.from_key)
                .map(|c| c.nickname.clone());
            MessageInfo {
                id: m.id,
                from_key: m.from_key,
                to_key: m.to_key,
                content: m.content,
                timestamp: m.timestamp,
                is_read: m.is_read,
                from_name,
            }
        })
        .collect();

    Ok(infos)
}

pub fn get_unread_count() -> Result<u64, ApiError> {
    let public_key = get_public_key()?;
    let db_guard = CURRENT_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.unread_count(&public_key)
        .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))
}

pub fn mark_message_read(msg_id: u64) -> Result<(), ApiError> {
    let db_guard = CURRENT_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.mark_read(msg_id)
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
