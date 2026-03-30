// ============================================================
// ROOT v2.0 — api/database.rs
// FFI функции: открытие БД, Panic Button
// ============================================================

use root_identity::Identity;
use root_storage::Database;
use super::state::APP_STATE;
use super::types::ApiError;

pub fn unlock_database(password: String, db_path: String) -> Result<bool, ApiError> {
    let current_dir = std::env::current_dir().unwrap_or_default();
    println!("  📁 Рабочая папка: {:?}", current_dir);

    {
        let state = APP_STATE.lock().unwrap();
        if state.panic_activated {
            return Err(ApiError::PanicActivated);
        }
    }

    let db = Database::open(&db_path, &password)
        .map_err(|e: root_storage::StorageError| ApiError::StorageError(e.to_string()))?;

    db.initialize()
        .map_err(|e: root_storage::StorageError| ApiError::StorageError(e.to_string()))?;

    let mut state = APP_STATE.lock().unwrap();

    if let Ok(Some((_, mnemonic))) = db.load_identity() {
        use bip39::Mnemonic;
        if let Ok(parsed) = mnemonic.parse::<Mnemonic>() {
            let identity = Identity::from_mnemonic(&parsed);
            state.identity = Some(identity);
            println!("  ✅ Identity загружена из БД");
        }
    }

    state.database = Some(db);
    println!("  ✅ База данных разблокирована: {}", db_path);
    Ok(true)
}

pub fn panic_button() -> Result<(), ApiError> {
    println!("  💣 PANIC BUTTON — уничтожение данных...");
    let mut state = APP_STATE.lock().unwrap();
    state.panic_activated = true;

    // Останавливаем P2P узел через oneshot канал
    if let Some(shutdown) = state.p2p_shutdown.take() {
        let _ = shutdown.send(());
        println!("  🛑 P2P сигнал остановки отправлен");
    }

    state.p2p_sender = None;

    if let Some(db) = state.database.as_mut() {
        db.panic_destroy();
    }

    state.identity = None;
    state.ledger   = None;
    state.database = None;

    println!("  ✅ Все данные уничтожены. Перезапусти приложение.");
    Err(ApiError::PanicActivated)
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.verify_integrity()
        .map_err(|e: root_storage::StorageError| ApiError::StorageError(e.to_string()))
}

pub fn is_panic_activated() -> bool {
    APP_STATE.lock().unwrap().panic_activated
}
