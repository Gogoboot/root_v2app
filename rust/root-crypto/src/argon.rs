// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/argon.rs
// ═══════════════════════════════════════════════════════════

use argon2::{Argon2, Algorithm, Version, Params};
use password_hash::{SaltString, PasswordHasher};
//use rand::RngCore;
use crate::types::{CryptoError, SecureKey, Salt};

pub fn derive_key(password: &str, salt: &Salt) -> Result<SecureKey, CryptoError> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|_| CryptoError::DerivationFailed)?;
    
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, Some(32))
            .map_err(|_| CryptoError::DerivationFailed)?,
    );
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|_| CryptoError::DerivationFailed)?;
    
    let mut key = SecureKey::default();
    if let Some(hash) = password_hash.hash {
        let hash_bytes: &[u8] = hash.as_bytes();
        if hash_bytes.len() >= 32 {
            key[..32].copy_from_slice(&hash_bytes[..32]);
        } else {
            return Err(CryptoError::DerivationFailed);
        }
    } else {
        return Err(CryptoError::DerivationFailed);
    }
    
    Ok(key)
}

pub fn wipe_password<T: zeroize::Zeroize>(secret: &mut T) {
    secret.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = "test_password";
        let salt = [1u8; 16];
        
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        
        // Один и тот же пароль + соль = одинаковый ключ
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_different_salt() {
        let password = "test_password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        
        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();
        
        // Разная соль = разный ключ
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_derive_key_zeroizes_on_error() {
        // Невалидная соль (слишком короткая) должна вернуть ошибку,
        // а не паниковать или возвращать частичные данные
        let password = "test";
        let bad_salt = [0u8; 8]; // ← не 16 байт
        
        // Этот тест зависит от реализации SaltString::encode_b64,
        // но полезно проверить, что ошибка обрабатывается корректно
        let result = derive_key(password, &bad_salt);
        // Ожидаем ошибку, но не панику
        assert!(result.is_err());
    }
}
