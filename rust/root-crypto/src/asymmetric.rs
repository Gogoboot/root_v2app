// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/asymmetric.rs (Задача #22 + #23)
// ═══════════════════════════════════════════════════════════

use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305};
use rand::RngCore;
use sha2::{Sha256, Digest};
use zeroize::{Zeroizing, ZeroizeOnDrop};
use generic_array::GenericArray;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use std::fmt;

// === Ошибки асимметричного шифрования ===
#[derive(Debug)]
pub enum AsymmetricError {
    KeyGenerationFailed(String),
    DerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    SignatureInvalid,
}

impl fmt::Display for AsymmetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsymmetricError::KeyGenerationFailed(e) => write!(f, "Key gen failed: {}", e),
            AsymmetricError::DerivationFailed => write!(f, "Key derivation failed"),
            AsymmetricError::EncryptionFailed => write!(f, "Encryption failed"),
            AsymmetricError::DecryptionFailed => write!(f, "Decryption failed"),
            AsymmetricError::SignatureInvalid => write!(f, "Invalid signature"),
        }
    }
}

impl std::error::Error for AsymmetricError {}

// === X25519 Ключи для ECDH обмена ===

/// Общий секрет после ECDH обмена
#[derive(ZeroizeOnDrop, Clone)]
pub struct SharedSecret(pub(crate) [u8; 32]);

/// Пара ключей для P2P шифрования (X25519 + Ed25519)
/// ⚠️ SigningKey не поддерживает Zeroize, поэтому храним без обёртки

pub struct Keypair {
    x25519_secret: Zeroizing<[u8; 32]>,
    x25519_public: [u8; 32],
    ed25519_signing: Option<SigningKey>,   // Без Zeroizing!
    ed25519_verifying: Option<VerifyingKey>,
}

// Стандартная базовая точка для X25519
const BASEPOINT: [u8; 32] = {
    let mut bp = [0u8; 32];
    bp[0] = 9;
    bp
};

// === Генерация ключей ===

impl Keypair {
    pub fn generate() -> Result<Self, AsymmetricError> {
        // X25519
        let mut x25519_secret = Zeroizing::new([0u8; 32]);
        rand::thread_rng().fill_bytes(x25519_secret.as_mut_slice());
        
        x25519_secret[0] &= 248;
        x25519_secret[31] &= 127;
        x25519_secret[31] |= 64;
        
        let x25519_public = x25519_dalek::x25519(*x25519_secret, BASEPOINT);
        
        // Ed25519
        let mut csprng = rand::rngs::OsRng {};
        let ed25519_signing = SigningKey::generate(&mut csprng);
        let ed25519_verifying = ed25519_signing.verifying_key();
        
        Ok(Self {
            x25519_secret,
            x25519_public,
            ed25519_signing: Some(ed25519_signing),  // Нет Zeroizing!
            ed25519_verifying: Some(ed25519_verifying),
        })
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.x25519_public
    }

    pub fn ed25519_public_key_bytes(&self) -> [u8; 32] {
        self.ed25519_verifying.as_ref()
            .map(|vk| vk.to_bytes())
            .unwrap_or([0u8; 32])
    }

    pub fn derive_shared_secret(&self, peer_x25519_public: &[u8; 32]) -> Result<SharedSecret, AsymmetricError> {
        let shared_bytes = x25519_dalek::x25519(*self.x25519_secret, *peer_x25519_public);
        Ok(SharedSecret(shared_bytes))
    }

    /// Конвертирует приватный ключ в строку Base64 для хранения
    pub fn to_base64(&self) -> String {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        BASE64.encode(self.x25519_secret.as_slice())
    }

    pub fn from_base64(s: &str) -> Result<Self, AsymmetricError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        let decoded = BASE64.decode(s)
            .map_err(|e| AsymmetricError::KeyGenerationFailed(format!("Base64 decode: {}", e)))?;
        
        if decoded.len() != 32 {
            return Err(AsymmetricError::KeyGenerationFailed("Invalid key size".to_string()));
        }
        
        let mut x25519_secret = Zeroizing::new([0u8; 32]);
        x25519_secret.copy_from_slice(&decoded);
        
        x25519_secret[0] &= 248;
        x25519_secret[31] &= 127;
        x25519_secret[31] |= 64;
        
        let x25519_public = x25519_dalek::x25519(*x25519_secret, BASEPOINT);
        
        // При загрузке только X25519, Ed25519 генерируем заново
        let mut csprng = rand::rngs::OsRng {};
        let ed25519_signing = SigningKey::generate(&mut csprng);
        let ed25519_verifying = ed25519_signing.verifying_key();
        
        Ok(Self {
            x25519_secret,
            x25519_public,
            ed25519_signing: Some(ed25519_signing),
            ed25519_verifying: Some(ed25519_verifying),
        })
    }

    pub fn can_sign(&self) -> bool {
        self.ed25519_signing.is_some()
    }
    
    pub fn can_verify(&self) -> bool {
        self.ed25519_verifying.is_some()
    }

    // === Геттеры для тестов (#23) ===
    pub fn get_ed25519_signing_key(&self) -> Option<&SigningKey> {
        self.ed25519_signing.as_ref()
    }
    
    pub fn get_ed25519_verifying_key(&self) -> Option<&VerifyingKey> {
        self.ed25519_verifying.as_ref()
    }
}

impl PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        self.x25519_public == other.x25519_public
    }
}

// === Шифрование и дешифровка сообщений ===

/// Зашифровывает сообщение для конкретного пира
pub fn encrypt_for_peer(
    shared_secret: &SharedSecret,
    plaintext: &[u8],
) -> Result<Vec<u8>, AsymmetricError> {
    // Создаём ключ из общего секрета через SHA256
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.0);
    let hash = hasher.finalize();
    
    let key = GenericArray::from_slice(&hash[..32]);
    let cipher = ChaCha20Poly1305::new(key);
    
    // Уникальный nonce для каждого сообщения
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    
    let nonce_array = GenericArray::from_slice(&nonce);
    let ciphertext = cipher.encrypt(nonce_array, plaintext)
        .map_err(|_| AsymmetricError::EncryptionFailed)?;
    
    // Возвращаем: [12 байт nonce][ciphertext с tag]
    let mut result = vec![0u8; 12 + ciphertext.len()];
    result[..12].copy_from_slice(&nonce);
    result[12..].copy_from_slice(&ciphertext);
    
    Ok(result)
}

/// Расшифровывает сообщение от пира
pub fn decrypt_from_peer(
    shared_secret: &SharedSecret,
    encrypted_blob: &[u8],
) -> Result<Vec<u8>, AsymmetricError> {
    if encrypted_blob.len() < 12 {
        return Err(AsymmetricError::DecryptionFailed);
    }
    
    // Ключ из общего секрета
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.0);
    let hash = hasher.finalize();
    
    let key = GenericArray::from_slice(&hash[..32]);
    let cipher = ChaCha20Poly1305::new(key);
    
    // Разделяем nonce и ciphertext
    let nonce = &encrypted_blob[..12];
    let ciphertext = &encrypted_blob[12..];
    
    // Расшифровываем
    let nonce_array = GenericArray::from_slice(nonce);
    let plaintext = cipher.decrypt(nonce_array, ciphertext)
        .map_err(|_| AsymmetricError::DecryptionFailed)?;
    
    Ok(plaintext)
}

// === Подписи (Ed25519) для задачи #23 ===

#[derive(Clone)]
pub struct SignedMessage {
    pub encrypted_payload: Vec<u8>,
    pub signature: [u8; 64],
}

pub fn sign_outgoing_message(
    signing_key: &SigningKey,
    encrypted_payload: &[u8],
) -> [u8; 64] {
    let sig = signing_key.sign(encrypted_payload);
    sig.to_bytes()
}

pub fn verify_incoming_signature(
    sender_verifying_key: &VerifyingKey,
    encrypted_payload: &[u8],
    signature: &[u8; 64],
) -> Result<(), AsymmetricError> {
    let sig = Signature::from_bytes(signature);
    sender_verifying_key.verify(encrypted_payload, &sig)
        .map_err(|_| AsymmetricError::SignatureInvalid)?;
    Ok(())
}

pub fn receive_and_decrypt_with_verification(
    my_keypair: &Keypair,
    sender_x25519_public: &[u8; 32],
    sender_ed25519_public: &[u8; 32],
    signed_message: &SignedMessage,
) -> Result<Vec<u8>, AsymmetricError> {
    // 1. Сначала проверяем подпись
    let verifying_key = VerifyingKey::from_bytes(sender_ed25519_public)
        .map_err(|_| AsymmetricError::SignatureInvalid)?;
    verify_incoming_signature(&verifying_key, &signed_message.encrypted_payload, &signed_message.signature)?;
    
    // 2. Только после успеха — расшифровываем
    let shared_secret = my_keypair.derive_shared_secret(sender_x25519_public)?;
    decrypt_from_peer(&shared_secret, &signed_message.encrypted_payload)
}

pub fn send_encrypted_signed(
    my_signing_key: &SigningKey,
    shared_secret: &SharedSecret,
    plaintext: &[u8],
) -> Result<SignedMessage, AsymmetricError> {
    let encrypted_payload = encrypt_for_peer(shared_secret, plaintext)?;
    let signature = sign_outgoing_message(my_signing_key, &encrypted_payload);
    
    Ok(SignedMessage {
        encrypted_payload,
        signature,
    })
}

// === Юнит-тесты ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation_and_serialization() {
        let kp1 = Keypair::generate().unwrap();
        let serialized = kp1.to_base64();
        
        let kp2 = Keypair::from_base64(&serialized).unwrap();
        
        assert_eq!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }

    #[test]
    fn test_ecdh_exchange() {
        let alice_kp = Keypair::generate().unwrap();
        let bob_kp = Keypair::generate().unwrap();
        
        let alice_shared = alice_kp.derive_shared_secret(&bob_kp.public_key_bytes()).unwrap();
        let bob_shared = bob_kp.derive_shared_secret(&alice_kp.public_key_bytes()).unwrap();
        
        assert_eq!(alice_shared.0, bob_shared.0);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let alice_kp = Keypair::generate().unwrap();
        let bob_kp = Keypair::generate().unwrap();
        
        let shared_secret = alice_kp.derive_shared_secret(&bob_kp.public_key_bytes()).unwrap();
        
        let message = b"Hello, P2P!";
        let encrypted = encrypt_for_peer(&shared_secret, message).unwrap();
        
        let bob_shared = bob_kp.derive_shared_secret(&alice_kp.public_key_bytes()).unwrap();
        let decrypted = decrypt_from_peer(&bob_shared, &encrypted).unwrap();
        
        assert_eq!(&decrypted, message);
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

    #[test]
    fn test_verify_rejects_invalid_signature() {
        let kp_sender = Keypair::generate().unwrap();
        let kp_receiver = Keypair::generate().unwrap();
        
        let ed_signing = kp_sender.get_ed25519_signing_key().unwrap();
        let ed_verifying_receiver = kp_receiver.get_ed25519_verifying_key().unwrap();
        
        let message = b"Test message";
        let sig = sign_outgoing_message(ed_signing, message);
        
        let result = verify_incoming_signature(ed_verifying_receiver, message, &sig);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AsymmetricError::SignatureInvalid));
    }

    #[test]
    fn test_full_send_receive_with_signature() {
        let alice_kp = Keypair::generate().unwrap();
        let bob_kp = Keypair::generate().unwrap();
        
        let alice_shared = alice_kp.derive_shared_secret(&bob_kp.public_key_bytes()).unwrap();
        let ed_alice_signing = alice_kp.get_ed25519_signing_key().unwrap();
        
        let plaintext = b"Secret message from Alice";
        let signed_msg = send_encrypted_signed(ed_alice_signing, &alice_shared, plaintext).unwrap();
        
        let result = receive_and_decrypt_with_verification(
            &bob_kp,
            &alice_kp.public_key_bytes(),
            &alice_kp.ed25519_public_key_bytes(),
            &signed_msg,
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), plaintext);
    }
}
