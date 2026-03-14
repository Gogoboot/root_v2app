// ============================================================
// ROOT v2.0 — api/utils.rs
// Утилиты: версия, валидация, целостность БД
// ============================================================

use super::state::CURRENT_DB;
use super::types::ApiError;

pub fn get_version() -> String {
    format!("{} ({})", crate::VERSION, crate::BUILD_DATE)
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    let db_guard = CURRENT_DB.lock().unwrap();
    let db       = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.verify_integrity()
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

/// Проверить что строка является валидным Ed25519 публичным ключом
/// Валидный ключ: 64 символа hex = 32 байта
pub fn validate_public_key(key: String) -> bool {
    if key.len() != 64 {
        return false;
    }
    // Проверяем что все символы hex
    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    // Проверяем что можно декодировать в валидный Ed25519 ключ
    if let Ok(bytes) = hex::decode(&key) {
        ed25519_dalek::VerifyingKey::from_bytes(
            bytes.as_slice().try_into().unwrap_or(&[0u8; 32])
        ).is_ok()
    } else {
        false
    }
}
