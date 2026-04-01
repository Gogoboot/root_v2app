// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/argon.rs
// ═══════════════════════════════════════════════════════════

use argon2::{Argon2, Algorithm, Version, Params};
use zeroize::{Zeroize, Zeroizing};  // ✅ Добавлен Zeroize
use crate::types::{CryptoError, SecureKey};

pub fn derive_key(password: &Zeroizing<String>, salt: &[u8]) -> Result<SecureKey, CryptoError> {
    // ✅ Параметры Argon2id
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, Some(32))  // 19MB, 2 итерации, 1 поток, 32 байта вывод
            .map_err(|_| CryptoError::DerivationFailed)?,
    );
    
    // ✅ Используем hash_password_into с raw солью (не SaltString!)
    let mut key_bytes = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|_| CryptoError::DerivationFailed)?;
    
    // ✅ Копируем в SecureKey
    let mut key = SecureKey::default();
    key[..32].copy_from_slice(&key_bytes);
    
    // ✅ Затирание временного буфера
    key_bytes.zeroize();
    
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

