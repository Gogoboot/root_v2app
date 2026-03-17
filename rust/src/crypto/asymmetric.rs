// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — crypto/asymmetric.rs (Задача #22)
// ═══════════════════════════════════════════════════════════

use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305};
use rand::RngCore;
use sha2::{Sha256, Digest};
use zeroize::{Zeroizing, ZeroizeOnDrop};
use generic_array::GenericArray;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

// === Ошибки асимметричного шифрования ===
#[derive(Debug)]
pub enum AsymmetricError {
    KeyGenerationFailed(String),
    DerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    SignatureInvalid,
}

impl std::fmt::Display for AsymmetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// Пара ключей для P2P шифрования
#[derive(Clone, ZeroizeOnDrop)]
pub struct Keypair {
    secret_bytes: Zeroizing<[u8; 32]>,
    public_key: [u8; 32],
}

// Стандартная базовая точка для X25519 (генератор кривой)
const BASEPOINT: [u8; 32] = {
    let mut bp = [0u8; 32];
    bp[0] = 9; // 0x09 — базовая точка X25519
    bp
};

// === Генерация ключей ===

impl Keypair {
    /// Генерирует новую пару ключей X25519
    pub fn generate() -> Result<Self, AsymmetricError> {
        let mut secret = Zeroizing::new([0u8; 32]);
        
        // Генерация случайных байт для приватного ключа
        rand::thread_rng().fill_bytes(secret.as_mut_slice());
        
        // Клэмпинг приватного ключа (требуется для X25519 безопасности)
        secret[0] &= 248;
        secret[31] &= 127;
        secret[31] |= 64;
        
        // Вычисление публичного ключа через x25519 базовую точку
        let public_bytes = x25519_dalek::x25519(*secret, BASEPOINT);
        
        Ok(Self {
            secret_bytes: secret,
            public_key: public_bytes,
        })
    }

    /// Получает публичный ключ в формате [u8; 32]
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key
    }

    /// Вычисляет общий секрет с помощью пира
    pub fn derive_shared_secret(&self, peer_public_key: &[u8; 32]) -> Result<SharedSecret, AsymmetricError> {
        let shared_bytes = x25519_dalek::x25519(*self.secret_bytes, *peer_public_key);
        
        Ok(SharedSecret(shared_bytes))
    }

    /// Конвертирует приватный ключ в строку Base64 для хранения
    pub fn to_base64(&self) -> String {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        BASE64.encode(self.secret_bytes.as_slice())
    }

    /// Восстанавливает ключ из Base64
    pub fn from_base64(s: &str) -> Result<Self, AsymmetricError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        let decoded = BASE64.decode(s)
            .map_err(|e| AsymmetricError::KeyGenerationFailed(format!("Base64 decode: {}", e)))?;
        
        if decoded.len() != 32 {
            return Err(AsymmetricError::KeyGenerationFailed("Invalid key size".to_string()));
        }
        
        let mut secret = Zeroizing::new([0u8; 32]);
        secret.copy_from_slice(&decoded);
        
        // Проводим клэмпинг при восстановлении
        secret[0] &= 248;
        secret[31] &= 127;
        secret[31] |= 64;
        
        let public_bytes = x25519_dalek::x25519(*secret, BASEPOINT);
        
        Ok(Self {
            secret_bytes: secret,
            public_key: public_bytes,
        })
    }
}

impl PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        self.public_key == other.public_key
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
    
    // Шифруем (Nonce + Ciphertext)
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

pub fn sign_message(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

pub fn verify_signature(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    verifying_key.verify(message, signature).is_ok()
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
        
        let encrypted2 = encrypt_for_peer(&shared_secret, message).unwrap();
        assert_ne!(encrypted, encrypted2);
        
        let bob_shared = bob_kp.derive_shared_secret(&alice_kp.public_key_bytes()).unwrap();
        let decrypted = decrypt_from_peer(&bob_shared, &encrypted).unwrap();
        
        assert_eq!(&decrypted, message);
    }
}
