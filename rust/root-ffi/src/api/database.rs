// ============================================================
// ROOT v2.0 — api/database.rs
// FFI функции: открытие БД, Panic Button
// ============================================================

use super::state::APP_STATE;
use super::types::ApiError;
use root_identity::Identity;
use root_storage::Database;
use zeroize::Zeroizing;

pub fn unlock_database(password: String, db_path: String) -> Result<bool, ApiError> {
    // ✅ Обернуть пароль в Zeroizing
    let password = Zeroizing::new(password);

    let current_dir = std::env::current_dir().unwrap_or_default();
    println!("  📁 Рабочая папка: {:?}", current_dir);

    {
        let state = APP_STATE.lock().unwrap();
        if state.panic_activated {
            return Err(ApiError::PanicActivated);
        }
    }

    // ✅ Добавить mut
    let mut db = Database::open(&db_path, &password)
        .map_err(ApiError::from)?;

    db.initialize()
        .map_err(ApiError::from)?;

    let mut state = APP_STATE.lock().unwrap();

    // ✅ Мнемоника в Zeroizing
    if let Ok(Some((_, mnemonic_str))) = db.load_identity() {
        use bip39::Mnemonic;
        let mnemonic = Zeroizing::new(mnemonic_str);
        if let Ok(parsed) = mnemonic.parse::<Mnemonic>() {
            if let Ok(identity) = Identity::from_mnemonic(&parsed, "") {
                state.identity = Some(identity);
            }
        }
        println!("  ✅ Identity загружена из БД");
    }

    // Обновляем фазу
    let phase = if state.identity.is_some() {
        root_core::state::AppPhase::Ready
    } else {
        root_core::state::AppPhase::DbOpen
    };
    state.transition(phase);

    state.database = Some(db);
    println!("  ✅ База данных разблокирована: {}", db_path);
    Ok(true)
}

pub fn panic_button() -> Result<(), ApiError> {
    println!("  💣 PANIC BUTTON — уничтожение данных...");
    let mut state = APP_STATE.lock().unwrap();
    state.panic_activated = true;

    if let Some(shutdown) = state.p2p_shutdown.take() {
        let _ = shutdown.send(());
        println!("  🛑 P2P сигнал остановки отправлен");
    }

    state.p2p_sender = None;

    // ✅ Обработать результат panic_destroy()
    // В root-ffi/src/api/database.rs:
    if let Some(db) = state.database.as_mut() {
        db.panic_destroy()
            .map_err(ApiError::from)?;
    }

    state.identity = None;
    state.ledger = None;
    state.database = None;

    println!("  ✅ Все данные уничтожены. Перезапусти приложение.");
    Err(ApiError::PanicActivated)
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.verify_integrity()
    .map_err(ApiError::from)
}

pub fn is_panic_activated() -> bool {
    APP_STATE.lock().unwrap().panic_activated
}
