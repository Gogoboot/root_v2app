// ============================================================
// root-crypto — домен шифрования
//
// Argon2id деривация ключей, ChaCha20-Poly1305, ECDH X25519
// ============================================================

pub mod argon;
pub mod asymmetric;
pub mod symmetric;
pub mod types;

pub use argon::{derive_key, wipe_password};
pub use asymmetric::{
    AsymmetricError, Keypair, SharedSecret, SignedMessage,
    decrypt_from_peer, encrypt_for_peer,
    receive_and_decrypt_with_verification, send_encrypted_signed,
    sign_outgoing_message, verify_incoming_signature,
};
pub use symmetric::{decrypt, encrypt, pack_for_storage, unpack_from_storage};
pub use types::{CryptoError, CryptoNonce, EncryptedBlob, Salt, SecureKey};


#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroizing;  // ✅ Добавить импорт

    #[test]
    fn test_derive_key() {
        let salt: Salt = [1u8; 32];
        let password = Zeroizing::new(String::from("test_password"));  // ✅
        let key = derive_key(&password, &salt).unwrap();
        // ключ — 32 байта
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_same_password_same_key() {
        let salt: Salt = [7u8; 32];
        let password = Zeroizing::new(String::from("password"));  // ✅
        let key1 = derive_key(&password, &salt).unwrap();
        let key2 = derive_key(&password, &salt).unwrap();
        // детерминированная деривация — одинаковый пароль даёт одинаковый ключ
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let salt: Salt = [3u8; 32];
        let password1 = Zeroizing::new(String::from("password1"));  // ✅
        let password2 = Zeroizing::new(String::from("password2"));  // ✅
        let key1 = derive_key(&password1, &salt).unwrap();
        let key2 = derive_key(&password2, &salt).unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let salt: Salt = [1u8; 32];
        let password = Zeroizing::new(String::from("test"));  // ✅
        let key = derive_key(&password, &salt).unwrap();
        let plaintext = b"secret message";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let salt: Salt = [1u8; 32];
        let password1 = Zeroizing::new(String::from("correct"));  // ✅
        let password2 = Zeroizing::new(String::from("wrong"));    // ✅
        let key1 = derive_key(&password1, &salt).unwrap();
        let key2 = derive_key(&password2, &salt).unwrap();
        let encrypted = encrypt(&key1, b"data").unwrap();
        // неверный ключ — расшифровка должна упасть
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_ecdh_shared_secret() {
        let alice = Keypair::generate().unwrap();
        let bob = Keypair::generate().unwrap();
        let alice_shared = alice.derive_shared_secret(&bob.public_key_bytes()).unwrap();
        let bob_shared = bob.derive_shared_secret(&alice.public_key_bytes()).unwrap();
        // общий секрет должен совпасть с обеих сторон
        assert_eq!(alice_shared.0, bob_shared.0);
    }
}
