// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/argon.rs
// ═══════════════════════════════════════════════════════════

use argon2::{Argon2, Algorithm, Version, Params};
use zeroize::{Zeroize, Zeroizing};
use crate::types::{CryptoError, SecureKey};

// ✅ Импорт констант (после настройки constants.rs)
use crate::constants::ARGON2_PEPPER;

pub fn derive_key(password: &Zeroizing<String>, salt: &[u8]) -> Result<SecureKey, CryptoError> {
    
    // ✅ Шаг 1: Комбинируем соль + pepper
    let mut combined = Vec::with_capacity(salt.len() + ARGON2_PEPPER.len());
    combined.extend_from_slice(salt);
    combined.extend_from_slice(ARGON2_PEPPER);
 
    // ✅ Шаг 2: Параметры Argon2id
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, Some(32))
            .map_err(|_| CryptoError::DerivationFailed)?,
    );
    
    // ✅ Шаг 3: Используем combined (соль + pepper), а не только salt!
    let mut key_bytes = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), &combined, &mut key_bytes)
        .map_err(|_| CryptoError::DerivationFailed)?;
    
    // ✅ Шаг 4: Копируем в SecureKey
    let mut key = SecureKey::default();
    key[..32].copy_from_slice(&key_bytes);
    
    // ✅ Шаг 5: Затирание временных буферов
    key_bytes.zeroize();
    combined.zeroize();  // ← ✅ Важно! Pepper тоже затереть
    
    Ok(key)
}

pub fn wipe_password<T: Zeroize>(secret: &mut T) {
    secret.zeroize();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = Zeroizing::new(String::from("test_password"));
        let salt = [1u8; 32];  // ✅ 32 байта
        
        let key1 = derive_key(&password, &salt).unwrap();
        let key2 = derive_key(&password, &salt).unwrap();
        
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_salt() {
        let password = Zeroizing::new(String::from("test_password"));
        let salt1 = [1u8; 32];
        let salt2 = [2u8; 32];
        
        let key1 = derive_key(&password, &salt1).unwrap();
        let key2 = derive_key(&password, &salt2).unwrap();
        
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

}

