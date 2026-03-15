// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/argon.rs
// ═══════════════════════════════════════════════════════════

use argon2::{Argon2, Algorithm, Version, Params};
use password_hash::{SaltString, PasswordHasher};
//use rand::RngCore;
use crate::crypto::types::{CryptoError, SecureKey, Salt};

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
        let hash_bytes = hash.as_bytes();
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
