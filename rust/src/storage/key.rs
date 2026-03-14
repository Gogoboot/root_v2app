// ============================================================
// ROOT v2.0 — storage/key.rs
// StorageKey — Argon2id деривация ключа из пароля
// ============================================================

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::constants::{
    ARGON2_ITERATIONS, ARGON2_MEMORY_KB, ARGON2_PARALLELISM, ARGON2_SALT, KEY_LEN,
};
use super::error::StorageError;

/// Ключ шифрования — никогда не записывается на диск
/// Автоматически обнуляется при выходе из области видимости
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct StorageKey {
    pub(crate) bytes: [u8; KEY_LEN],
}

impl StorageKey {
    /// Derive ключ из пароля через Argon2id
    pub fn from_password(password: &str) -> Result<Self, StorageError> {
        let params = Params::new(
            ARGON2_MEMORY_KB,
            ARGON2_ITERATIONS,
            ARGON2_PARALLELISM,
            Some(KEY_LEN),
        )
        .map_err(|e| StorageError::Crypto(e.to_string()))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key_bytes = [0u8; KEY_LEN];
        argon2
            .hash_password_into(password.as_bytes(), ARGON2_SALT, &mut key_bytes)
            .map_err(|e| StorageError::Crypto(e.to_string()))?;

        Ok(StorageKey { bytes: key_bytes })
    }

    /// Hex строка ключа для SQLCipher PRAGMA
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Panic Button — мгновенно обнулить ключ
    pub fn destroy(&mut self) {
        self.bytes.zeroize();
    }
}
