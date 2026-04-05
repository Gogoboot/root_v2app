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
        mnemonic: Some(mnemonic_str.clone()), // обычная String
        //mnemonic: Some(Zeroizing::new(mnemonic_str.clone())),
        network: crate::NETWORK_ID.to_string(),
    };

    let mut ledger = Ledger::new();
    ledger.get_or_create(&pubkey_hex);

    // Один lock — всё делаем внутри одного блока.
    // Два lock() на одном мьютексе в одном потоке = дедлок.
    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::InternalError("mutex poisoned".into()))?;

    // ─── Проверка: аккаунт уже существует ────────────────────────────────
    // Если identity уже есть в БД — запрещаем создавать новую.
    // Иначе старая мнемоника будет перезаписана и старые сообщения
    // станут нечитаемы (они зашифрованы старым ключом).
    if let Some(db) = state.database.as_ref() {
        if let Ok(Some(_)) = db.load_identity() {
            return Err(ApiError::InvalidInput(
                "Аккаунт уже существует. Используй восстановление из мнемоники.".into()
            ));
        }
    }
    // ─────────────────────────────────────────────────────────────────────

    state.identity = Some(identity);
    state.ledger = Some(ledger);

    if let Some(db) = state.database.as_ref() {
        db.save_identity(&pubkey_hex, &mnemonic_str)
            .map_err(|e| ApiError::StorageError(e.to_string()))?;
        println!("  ✅ Identity сохранена в БД");
    }

    // Обновляем фазу внутри того же lock
    state.transition(AppPhase::Identified);

    println!("  ✅ Identity создана: {}...", &info.public_key[..16]);
    Ok(info)
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
