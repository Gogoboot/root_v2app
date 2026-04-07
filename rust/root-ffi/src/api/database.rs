// ============================================================
// ROOT v2.0 — api/src/database.rs
// FFI функции: открытие БД, Panic Button
// ============================================================

use super::state::APP_STATE;
use super::types::{ApiError, UnlockResult};
use root_identity::Identity;
use root_storage::Database;
use zeroize::Zeroizing;
use crate::require_state;
use root_core::state::AppPhase;

/// Открывает базу данных и проверяет статус аккаунта.
///
/// # Сценарии
///
/// - `status: "ok"` — всё хорошо, входим в приложение
/// - `status: "mnemonic_pending"` — мнемоника не была подтверждена,
///   показываем экран мнемоники снова вместе с самой мнемоникой
///
/// # Ошибки
///
/// - Неверный пароль → `Err(ApiError::StorageError)`
/// - Panic активирован → `Err(ApiError::PanicActivated)`
pub fn unlock_database(password: String, db_path: String) -> Result<UnlockResult, ApiError> {
    let password = Zeroizing::new(password);

    println!("  📂 Путь к БД: {}", db_path);

    // Проверяем panic до открытия БД
    {
        let state = APP_STATE.lock()
            .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
        if state.panic_activated {
            return Err(ApiError::PanicActivated);
        }
    }

    // Открываем БД и инициализируем таблицы
    let mut db = Database::open(&db_path, &password)
        .map_err(ApiError::from)?;

    db.initialize().map_err(ApiError::from)?;
    println!("  ✅ Таблицы инициализированы");

    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    // Загружаем identity из БД если есть
    let mut public_key: Option<String> = None;

    if let Ok(Some((pubkey, mnemonic_str))) = db.load_identity() {
        use bip39::Mnemonic;
        let mnemonic = Zeroizing::new(mnemonic_str.clone());
        if let Ok(parsed) = mnemonic.parse::<Mnemonic>() {
            if let Ok(identity) = Identity::from_mnemonic(&parsed, "") {
                state.identity = Some(identity);
                public_key = Some(pubkey.clone());
                println!("  ✅ Identity загружена из БД");
            }
        }

        // ── Проверяем флаг подтверждения мнемоники ──────────────────────
        // Читаем настройку ДО того как сохраняем db в state,
        // потому что после move db будет недоступна.
        let confirmed = db.get_setting("mnemonic_confirmed")
            .unwrap_or(None)
            .map(|v| v == "true")
            .unwrap_or(false); // если настройки нет — считаем не подтверждено

        // Обновляем фазы
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
        }

        state.database = Some(db);

        // ── Возвращаем результат в зависимости от флага ─────────────────
        if !confirmed {
            // Сценарий 2 — мнемоника не подтверждена, показываем снова
            println!("  🔍 DEBUG: Returning mnemonic_pending | confirmed={}, mnemonic_len={}", //** */
            confirmed, mnemonic_str.len());  // ← ЛОГ 1 ******
            println!("  ⚠️  Мнемоника не подтверждена — возвращаем на экран мнемоники");
            return Ok(UnlockResult {
                status: "mnemonic_pending".to_string(),
                public_key,
                mnemonic: Some(mnemonic_str),
            });
        }

        // Сценарий 3 — обычный вход, всё подтверждено
        println!("  ✅ База данных разблокирована: {}", db_path);
        println!("  🔍 DEBUG: Returning ok (confirmed) | confirmed={}", confirmed);  // ← ЛОГ 2*****
        return Ok(UnlockResult {
            status: "ok".to_string(),
            public_key,
            mnemonic: None,
        });
    }

    // Identity в БД нет — первый запуск или чистая установка
    // Переходим в DbOpen, identity будет создана через generate_identity()
    if !state.transition(AppPhase::DbOpen) {
        return Err(ApiError::InternalError(
            format!("Transition failed: {:?} → DbOpen", state.phase)
        ));
    }
    println!("  🔄 Phase: ... → DbOpen (identity pending)");

    state.database = Some(db);
    println!("  ✅ База данных разблокирована: {}", db_path);
    println!("  DEBUG: Returning ok (no identity) | db_has_identity=false");  // ← ЛОГ 3******

    Ok(UnlockResult {
        status: "ok".to_string(),
        public_key: None,
        mnemonic: None,
    })
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
    state.ledger   = None;
    state.database = None;

    println!("  ✅ Все данные уничтожены. Перезапусти приложение.");
    Ok(())
}

pub fn verify_db_integrity() -> Result<bool, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    let db = state.database.as_ref()
        .ok_or(ApiError::DatabaseNotOpen)?;

    db.verify_integrity().map_err(ApiError::from)
}

pub fn is_panic_activated() -> Result<bool, ApiError> {
    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;
    Ok(state.panic_activated)
}

pub fn lock_database() -> Result<(), ApiError> {
    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    if let Some(shutdown) = state.p2p_shutdown.take() {
        let _ = shutdown.send(());
    }
    state.p2p_sender = None;

    state.reset();

    println!("  🔒 База данных закрыта, состояние сброшено в Fresh");
    Ok(())
}
