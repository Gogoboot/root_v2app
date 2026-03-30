// ============================================================
// ROOT v2.0 — api/identity.rs
// FFI функции: генерация и восстановление ключей
// ============================================================

use root_identity::Identity;
use zeroize::Zeroizing;
use root_economy::Ledger;
use super::state::APP_STATE;
use super::types::{ApiError, IdentityInfo};

pub fn generate_identity() -> Result<IdentityInfo, ApiError> {
    let (identity, mnemonic) = Identity::generate()
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;
    let pubkey_hex = hex::encode(identity.verifying_key.as_bytes());
    let mnemonic_str = mnemonic.to_string();

    let info = IdentityInfo {
        public_key: pubkey_hex.clone(),
        mnemonic: Some(Zeroizing::new(mnemonic_str.clone())),
        network: crate::NETWORK_ID.to_string(),
    };

    let mut ledger = Ledger::new();
    ledger.get_or_create(&pubkey_hex);

    let mut state = APP_STATE.lock().unwrap();
    state.identity = Some(identity);
    state.ledger = Some(ledger);

    if let Some(db) = state.database.as_ref() {
        let _ = db.save_identity(&pubkey_hex, &mnemonic_str);
        println!("  ✅ Identity сохранена в БД");
    }

    println!("  ✅ Identity создана: {}...", &info.public_key[..16]);
    Ok(info)
}

pub fn restore_identity(mnemonic: String) -> Result<IdentityInfo, ApiError> {
    use bip39::Mnemonic;
    let parsed = mnemonic
        .parse::<Mnemonic>()
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;

    let identity = Identity::from_mnemonic(&parsed)
        .map_err(|e| ApiError::IdentityError(e.to_string()))?;
    let pubkey_hex = hex::encode(identity.verifying_key.as_bytes());

    let info = IdentityInfo {
        public_key: pubkey_hex.clone(),
        mnemonic: None,
        network: crate::NETWORK_ID.to_string(),
    };

    let mut ledger = Ledger::new();
    ledger.get_or_create(&pubkey_hex);

    let mut state = APP_STATE.lock().unwrap();
    state.identity = Some(identity);
    state.ledger = Some(ledger);

    println!("  ✅ Identity восстановлена: {}...", &info.public_key[..16]);
    Ok(info)
}

pub fn get_public_key() -> Result<String, ApiError> {
    let state = APP_STATE.lock().unwrap();
    let identity = state.identity.as_ref().ok_or(ApiError::IdentityNotInitialized)?;
    Ok(hex::encode(identity.verifying_key.as_bytes()))
}

pub fn sign_message(message: Vec<u8>) -> Result<Vec<u8>, ApiError> {
    let state = APP_STATE.lock().unwrap();
    let identity = state.identity.as_ref().ok_or(ApiError::IdentityNotInitialized)?;
    let signature = identity.sign(&message);
    Ok(signature.to_bytes().to_vec())
}
