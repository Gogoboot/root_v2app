// src/crypto/mod.rs

pub mod types;
pub mod argon;
pub mod symmetric;
pub mod asymmetric;

pub use types::{CryptoError, EncryptedBlob, SecureKey, Salt, CryptoNonce};
pub use argon::{derive_key, wipe_password};
pub use symmetric::{encrypt, decrypt, pack_for_storage, unpack_from_storage};

// Экспорт для #23
pub use asymmetric::{
    Keypair, 
    SharedSecret, 
    encrypt_for_peer, 
    decrypt_from_peer,
    SignedMessage,
    sign_outgoing_message,
    verify_incoming_signature,
    send_encrypted_signed,
    receive_and_decrypt_with_verification,
    AsymmetricError,
};

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

    #[test]
    fn test_asymmetric_ecdh() {
        let alice = Keypair::generate().unwrap();
        let bob = Keypair::generate().unwrap();
        
        let alice_shared = alice.derive_shared_secret(&bob.public_key_bytes()).unwrap();
        let bob_shared = bob.derive_shared_secret(&alice.public_key_bytes()).unwrap();
        
        assert_eq!(alice_shared.0, bob_shared.0);
    }

    #[test]
    fn test_sign_verify_basic() {
        let kp = Keypair::generate().unwrap();
        let ed_signing = kp.get_ed25519_signing_key().unwrap();
        let ed_verifying = kp.get_ed25519_verifying_key().unwrap();
        
        let message = b"Test message";
        let sig = sign_outgoing_message(ed_signing, message);
        
        verify_incoming_signature(ed_verifying, message, &sig).unwrap();
    }
}
