// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/symmetric.rs
// ═══════════════════════════════════════════════════════════

use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce, Key,
};
use rand::RngCore;
use crate::crypto::types::{CryptoError, EncryptedBlob, SecureKey};

pub fn encrypt(key: &SecureKey, plaintext: &[u8]) -> Result<EncryptedBlob, CryptoError> {
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)?;
    
    Ok(EncryptedBlob { nonce: nonce_bytes, data: ciphertext })
}

pub fn decrypt(key: &SecureKey, blob: &EncryptedBlob) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
    let nonce = Nonce::from(blob.nonce);
    cipher.decrypt(&nonce, blob.data.as_slice())
        .map_err(|_| CryptoError::DecryptionFailed)
}

pub fn pack_for_storage(blob: &EncryptedBlob) -> Vec<u8> {
    let mut result = Vec::with_capacity(12 + blob.data.len());
    result.extend_from_slice(&blob.nonce);
    result.extend_from_slice(&blob.data);
    result
}

pub fn unpack_from_storage(data: &[u8]) -> Result<EncryptedBlob, CryptoError> {
    if data.len() < 12 { return Err(CryptoError::InvalidBlob); }
    let (nonce_part, ciphertext) = data.split_at(12);
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(nonce_part);
    Ok(EncryptedBlob { nonce, data: ciphertext.to_vec() })
}
