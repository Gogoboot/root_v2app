// ============================================================
// ROOT v2.0 — api/database.rs
// FFI функции: открытие БД, Panic Button
// ============================================================

use crate::identity::Identity;
use crate::storage::Database;

use super::state::{CURRENT_DB, CURRENT_IDENTITY, CURRENT_LEDGER, PANIC_ACTIVATED};
use super::types::ApiError;

pub fn unlock_database(password: String, db_path: String) -> Result<bool, ApiError> {
    let current_dir = std::env::current_dir().unwrap_or_default();
    println!("  📁 Рабочая папка: {:?}", current_dir);

    if *PANIC_ACTIVATED.lock().unwrap() {
        return Err(ApiError::PanicActivated);
    }

    let db =
        Database::open(&db_path, &password).map_err(|e| ApiError::StorageError(e.to_string()))?;

    db.initialize()
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    // Загружаем identity из БД если она сохранена
    if let Ok(Some((_, mnemonic))) = db.load_identity() {
        use bip39::Mnemonic;
        if let Ok(parsed) = mnemonic.parse::<Mnemonic>() {
            let identity = Identity::from_mnemonic(&parsed);
            *CURRENT_IDENTITY.lock().unwrap() = Some(identity);
            println!("  ✅ Identity загружена из БД");
        }
    }

    *CURRENT_DB.lock().unwrap() = Some(db);
    println!("  ✅ База данных разблокирована: {}", db_path);
    Ok(true)
}

pub fn panic_button() -> Result<(), ApiError> {
    println!("  🆘 PANIC BUTTON — уничтожение данных...");
    *PANIC_ACTIVATED.lock().unwrap() = true;

    if let Some(db) = CURRENT_DB.lock().unwrap().as_mut() {
        db.panic_destroy();
    }

    *CURRENT_IDENTITY.lock().unwrap() = None;
    *CURRENT_LEDGER.lock().unwrap() = None;
    *CURRENT_DB.lock().unwrap() = None;

    println!("  ✅ Все данные уничтожены. Перезапусти приложение.");
    Err(ApiError::PanicActivated)
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    let db_guard = CURRENT_DB.lock().unwrap();
    let db = db_guard.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.verify_integrity()
        .map_err(|e| ApiError::StorageError(e.to_string()))
}

pub fn is_panic_activated() -> bool {
    *PANIC_ACTIVATED.lock().unwrap()
}
