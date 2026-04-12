// ============================================================
// ROOT v2.0 — api/identity.rs
// FFI функции: генерация и восстановление ключей
// ============================================================

use super::state::APP_STATE;
use super::types::{ApiError, IdentityInfo};
use root_economy::Ledger;
use root_identity::Identity;
use zeroize::Zeroizing;
use crate::require_state;
use root_core::state::AppPhase;

pub fn generate_identity() -> Result<IdentityInfo, ApiError> {
    let (identity, mnemonic) = Identity::generate("")
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;

    let pubkey_hex = hex::encode(identity.verifying_key.as_bytes());
    let mnemonic_str = mnemonic.to_string();

    let info = IdentityInfo {
        public_key: pubkey_hex.clone(),
        mnemonic: Some(mnemonic_str.clone()),
        network: crate::NETWORK_ID.to_string(),
    };

    let mut ledger = Ledger::new();
    ledger.get_or_create(&pubkey_hex);

    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    // ─── Проверка: аккаунт уже существует ────────────────────────────────
    if let Some(db) = state.database.as_ref() {
        if let Ok(Some(_)) = db.load_identity() {
            return Err(ApiError::InvalidInput(
                "Аккаунт уже существует. Используй восстановление из мнемоники.".into()
            ));
        }
    }

    state.identity = Some(identity);
    state.ledger = Some(ledger);

    if let Some(db) = state.database.as_ref() {
        // Сохраняем ключи и мнемонику
        db.save_identity(&pubkey_hex, &mnemonic_str)
            .map_err(|e| ApiError::StorageError(e.to_string()))?;
        println!("  ✅ Identity сохранена в БД");

        // Помечаем мнемонику как НЕ подтверждённую.
        // Станет "true" только когда пользователь нажмёт "Я записал".
        db.set_setting("mnemonic_confirmed", "false")
            .map_err(|e| ApiError::StorageError(e.to_string()))?;
        println!("  ⚠️  mnemonic_confirmed = false");
    }

    if !state.transition(AppPhase::Identified) {
        return Err(ApiError::InternalError(
            format!("Transition failed: {:?} → Identified", state.phase)
        ));
    }

    if state.database.is_some() {
        if !state.transition(AppPhase::Ready) {
            return Err(ApiError::InternalError(
                "Transition Identified → Ready failed".into()
            ));
        }
        println!("  🔄 Phase: ... → Identified → Ready");
    }

    println!("  ✅ Identity создана: {}...", &info.public_key[..16]);
    Ok(info)
}

/// Подтверждает что пользователь записал мнемонику.
///
/// Вызывается из JS когда пользователь нажимает "Я записал все слова".
/// После этого при следующем входе мнемоника показана не будет.
pub fn confirm_mnemonic() -> Result<(), ApiError> {
    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    let db = state.database.as_ref()
        .ok_or(ApiError::DatabaseNotOpen)?;

    db.set_setting("mnemonic_confirmed", "true")
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    println!("  ✅ mnemonic_confirmed = true");
    Ok(())
}

pub fn restore_identity(mnemonic: String) -> Result<IdentityInfo, ApiError> {
    use bip39::Mnemonic;

    let parsed = mnemonic
        .parse::<Mnemonic>()
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;

    let identity = Identity::from_mnemonic(&parsed, "")
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;

    let pubkey_hex = hex::encode(identity.verifying_key.as_bytes());

    let info = IdentityInfo {
        public_key: pubkey_hex.clone(),
        mnemonic: None,
        network: crate::NETWORK_ID.to_string(),
    };

    let mut ledger = Ledger::new();
    ledger.get_or_create(&pubkey_hex);

    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    state.identity = Some(identity);
    state.ledger = Some(ledger);

    // При восстановлении мнемоника уже известна пользователю —
    // сразу помечаем как подтверждённую
    if let Some(db) = state.database.as_ref() {
        db.set_setting("mnemonic_confirmed", "true")
            .map_err(|e| ApiError::StorageError(e.to_string()))?;
        println!("  ✅ mnemonic_confirmed = true (восстановление)");
    }

    let old_phase = state.phase.clone();

    // Если фаза уже Ready или P2PActive — transition не нужен
    // Это нормально когда restore вызывается после unlock_database
    if !matches!(state.phase, AppPhase::Ready | AppPhase::P2PActive) {
        let target_phase = if state.database.is_some() {
            AppPhase::Ready
        } else {
            AppPhase::Identified
        };

        if !state.transition(target_phase.clone()) {
            return Err(ApiError::InternalError(
                format!("Transition failed: {:?} → {:?}", old_phase, target_phase)
            ));
        }

        // println внутри блока — target_phase доступна здесь
        println!("  🔄 Phase: {:?} → {:?}", old_phase, target_phase);
    } else {
        println!("  🔄 Phase уже Ready — transition пропущен");
    }

    println!("  ✅ Identity восстановлена: {}...", &info.public_key[..16]);
    Ok(info)
}


pub fn get_public_key() -> Result<String, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    let identity = state
        .identity
        .as_ref()
        .ok_or(ApiError::IdentityNotInitialized)?;

    Ok(hex::encode(identity.verifying_key.as_bytes()))
}

pub fn sign_message(message: Vec<u8>) -> Result<Vec<u8>, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    let identity = state
        .identity
        .as_ref()
        .ok_or(ApiError::IdentityNotInitialized)?;

    let signature = identity.sign(&message);
    Ok(signature.to_bytes().to_vec())
}
