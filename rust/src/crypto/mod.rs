// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/mod.rs
// ═══════════════════════════════════════════════════════════

pub mod types;
pub mod argon;
pub mod symmetric;
pub mod asymmetric; 

pub use asymmetric::{Keypair, SharedSecret, encrypt_for_peer, decrypt_from_peer};
pub use types::{CryptoError, EncryptedBlob, SecureKey, Salt, CryptoNonce};
pub use argon::{derive_key, wipe_password};
//pub use symmetric::encrypt;
//pub use symmetric::decrypt;
pub use symmetric::{encrypt, decrypt, pack_for_storage, unpack_from_storage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let salt = [1u8; 16];
        let key = derive_key("test", &salt).unwrap();
        let plaintext = b"Hello";
        let enc = encrypt(&key, plaintext).unwrap();
        let dec = decrypt(&key, &enc).unwrap();
        assert_eq!(plaintext, dec.as_slice());
    }
}
