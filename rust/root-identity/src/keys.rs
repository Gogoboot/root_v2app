// ============================================================
// ROOT v2.0 — identity/keys.rs
// Identity — Ed25519 ключевая пара
// ============================================================

use bip39::{Language, Mnemonic};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroize;
use thiserror::Error;

use super::seed::SecretSeed;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Ошибка генерации мнемоники: {0}")]
    MnemonicGenerationFailed(String),
    #[error("Неверный seed — недостаточно байт")]
    InvalidSeed,
}

pub struct Identity {
    /// Публичный ключ — можно передавать другим
    pub verifying_key: VerifyingKey,
    /// Приватный ключ — только в памяти, никогда не покидает устройство
    signing_key: SigningKey,
}

impl Identity {
    /// Генерация новой идентичности
    /// Возвращает Identity + мнемонику для резервной копии
    pub fn generate() -> Result<(Self, Mnemonic), IdentityError> {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| IdentityError::MnemonicGenerationFailed(e.to_string()))?;

        entropy.zeroize();

        let identity = Self::from_mnemonic(&mnemonic)?;
        Ok((identity, mnemonic))
    }

    /// Восстановление из мнемоники (24 слова)
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Result<Self, IdentityError> {
        let seed_bytes = mnemonic.to_seed("ROOT_v2");
        let mut seed = SecretSeed(seed_bytes);

        let key_bytes: [u8; 32] = seed.0[..32]
            .try_into()
            .map_err(|_| IdentityError::InvalidSeed)?;

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        seed.zeroize();

        Ok(Identity {
            signing_key,
            verifying_key,
        })
    }

    /// Подписать сообщение приватным ключом
    pub fn sign(&self, message: &[u8]) -> ed25519_dalek::Signature {
        use ed25519_dalek::Signer;
        self.signing_key.sign(message)
    }

    /// Получить байты приватного ключа для libp2p PeerID
    /// Возвращает SecretSeed — автоматически обнуляется при выходе из области видимости
    pub fn signing_key_bytes(&self) -> SecretSeed {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&self.signing_key.to_bytes());
        SecretSeed(bytes)
    }
}
