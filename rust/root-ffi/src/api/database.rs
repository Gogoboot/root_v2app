// ============================================================
// ROOT v2.0 — api/database.rs
// FFI функции: открытие БД, Panic Button
// ============================================================

use super::state::APP_STATE;
use super::types::ApiError;
use root_identity::Identity;
use root_storage::Database;
use zeroize::Zeroizing;
use crate::require_state;              // ← Добавлено
use root_core::state::AppPhase;        // ← Добавлено

pub fn unlock_database(password: String, db_path: String) -> Result<bool, ApiError> {
    let password = Zeroizing::new(password);

    let current_dir = std::env::current_dir().unwrap_or_default();
    println!("  📁 Рабочая папка: {:?}", current_dir);
    println!("  📂 Путь к БД: {}", db_path);

    // ✅ Безопасная проверка panic
    {
        let state = APP_STATE.lock()
            .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
        if state.panic_activated {
            return Err(ApiError::PanicActivated);
        }
    }

    let mut db = Database::open(&db_path, &password)
        .map_err(ApiError::from)?;

    db.initialize().map_err(ApiError::from)?;
    println!("  ✅ Таблицы созданы");

    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

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

    // ✅ Поэтапный переход (уже правильно!)
    if !state.transition(AppPhase::DbOpen) {
        return Err(ApiError::InternalError(
            format!("Transition failed: {:?} → DbOpen", state.phase)
        ));
    }
    
    if state.identity.is_some() {
        if !state.transition(AppPhase::Ready) {
            return Err(ApiError::InternalError(
                "Transition failed: DbOpen → Ready".into()
            ));
        }
        println!("  🔄 Phase: ... → DbOpen → Ready");
    } else {
        println!("  🔄 Phase: ... → DbOpen (identity pending)");
    }

    state.database = Some(db);
    println!("  ✅ База данных разблокирована: {}", db_path);
    Ok(true)
}

pub fn panic_button() -> Result<(), ApiError> {
    println!("  💣 PANIC BUTTON — уничтожение данных...");
    
    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
    
    state.panic_activated = true;

    if let Some(shutdown) = state.p2p_shutdown.take() {
        let _ = shutdown.send(());
        println!("  🛑 P2P сигнал остановки отправлен");
    }
    state.p2p_sender = None;

    if let Some(db) = state.database.as_mut() {
        db.panic_destroy().map_err(ApiError::from)?;
    }

    state.identity = None;
    state.ledger = None;
    state.database = None;

    println!("  ✅ Все данные уничтожены. Перезапусти приложение.");
    Ok(())  // ← Успешное выполнение
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);  // ← Добавлено
    
    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
    
    let db = state.database.as_ref()
        .ok_or(ApiError::DatabaseNotOpen)?;
    
    db.verify_integrity().map_err(ApiError::from)
}

/// Проверяет, активирован ли PanicButton
/// 
/// # Returns
/// * `Ok(true)` — panic активирован
/// * `Ok(false)` — panic не активирован  
/// * `Err(ApiError::InternalError)` — критическая ошибка (mutex poisoned)
pub fn is_panic_activated() -> Result<bool, ApiError> {  // ← Изменён возврат на Result
    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
    Ok(state.panic_activated)
}

/// Выход из аккаунта — сбрасывает всё состояние обратно в Fresh.
/// Вызывается из UI при нажатии "Выйти из аккаунта".
pub fn lock_database() -> Result<(), ApiError> {
    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    // Останавливаем P2P если запущен — дропаем каналы
    // Это сигнализирует фоновому потоку что нужно завершиться
    if let Some(shutdown) = state.p2p_shutdown.take() {
        let _ = shutdown.send(());
    }
    state.p2p_sender = None;

    // Полный сброс — как будто приложение только запустилось
    state.reset();

    println!("  🔒 База данных закрыта, состояние сброшено в Fresh");
    Ok(())
}
