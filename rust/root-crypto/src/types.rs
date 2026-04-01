// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/types.rs
// ═══════════════════════════════════════════════════════════

//use zeroize::Zeroize;
use serde::{Serialize, Deserialize};

pub type Salt = [u8; 32];
pub type CryptoNonce = [u8; 12];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncryptedBlob {
    pub nonce: CryptoNonce,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum CryptoError {
    DerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    InvalidNonce,
    InvalidBlob,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CryptoError::DerivationFailed => write!(f, "Ошибка деривации ключа"),
            CryptoError::EncryptionFailed => write!(f, "Ошибка шифрования"),
            CryptoError::DecryptionFailed => write!(f, "Ошибка расшифровки или подмена данных"),
            CryptoError::InvalidNonce => write!(f, "Неверный размер nonce"),
            CryptoError::InvalidBlob => write!(f, "Неверный формат зашифрованного блока"),
        }
    }
}

impl std::error::Error for CryptoError {}

pub type SecureKey = zeroize::Zeroizing<[u8; 32]>;
