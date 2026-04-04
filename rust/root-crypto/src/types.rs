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

pub type SecureKey = zeroize::Zeroizing<[u8; 32]>;
