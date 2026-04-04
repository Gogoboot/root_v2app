// ============================================================
// ROOT v2.0 — identity/keys.rs
// Identity — Ed25519 ключевая пара
// ============================================================

use bip39::{Language, Mnemonic};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::{Zeroize, Zeroizing};
use thiserror::Error;

use super::seed::SecretSeed;
use crate::constants::{BIP39_PREFIX, DERIVATION_INDEX};   // ← константы из root-crypto

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
    /// Генерация новой идентичности.
    ///
    /// `passphrase` — дополнительная защита поверх мнемоники.
    /// Пустая строка `""` означает без passphrase.
    ///
    /// # Важно
    ///
    /// Passphrase не хранится нигде — пользователь должен
    /// запомнить её отдельно от мнемоники. Без неё восстановление
    /// аккаунта невозможно.
    ///
    /// # Пример
    ///
    /// ```rust
    /// // Без passphrase:
    /// let (identity, mnemonic) = Identity::generate("")?;
    ///
    /// // С passphrase:
    /// let (identity, mnemonic) = Identity::generate("my_secret")?;
    /// ```
    pub fn generate(passphrase: &str) -> Result<(Self, Mnemonic), IdentityError> {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| IdentityError::MnemonicGenerationFailed(e.to_string()))?;

        // Обнуляем энтропию сразу — она больше не нужна
        entropy.zeroize();

        let identity = Self::from_mnemonic(&mnemonic, passphrase)?;
        Ok((identity, mnemonic))
    }

    /// Восстановление идентичности из мнемоники (24 слова).
    ///
    /// `passphrase` должна совпадать с той что использовалась
    /// при генерации — иначе получится другой аккаунт без ошибки.
    ///
    /// # Как работает деривация
    ///
    /// ```text
    /// мнемоника + "ROOT_v2_" + passphrase
    ///     ↓ BIP39 to_seed()
    /// seed [u8; 64]
    ///     ↓ байты 0..32  (DERIVATION_INDEX = 0)
    /// Ed25519 signing key
    /// ```
    ///
    /// Префикс `"ROOT_v2_"` защищает от коллизий с другими
    /// BIP39 приложениями использующими ту же мнемонику.
    pub fn from_mnemonic(mnemonic: &Mnemonic, passphrase: &str) -> Result<Self, IdentityError> {
        // Комбинируем системный префикс + пользовательская passphrase.
        // Zeroizing обнулит строку в памяти когда она выйдет из области видимости.
        let combined = Zeroizing::new(format!("{}{}", BIP39_PREFIX, passphrase));

        let seed_bytes = mnemonic.to_seed(&*combined);
        let mut seed = SecretSeed(seed_bytes);

        // DERIVATION_INDEX = 0 → берём первые 32 байта.
        // В будущем при BIP32: index * 32 .. (index + 1) * 32
        let offset = DERIVATION_INDEX as usize * 32;
        let key_bytes: [u8; 32] = seed.0[offset..offset + 32]
            .try_into()
            .map_err(|_| IdentityError::InvalidSeed)?;

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        // Обнуляем seed — приватные байты больше не нужны
        seed.zeroize();

        Ok(Identity {
            signing_key,
            verifying_key,
        })
    }

    /// Подписать сообщение приватным ключом.
    pub fn sign(&self, message: &[u8]) -> ed25519_dalek::Signature {
        use ed25519_dalek::Signer;
        self.signing_key.sign(message)
    }

    /// Байты приватного ключа для libp2p PeerID.
    ///
    /// Возвращает [`SecretSeed`] — автоматически обнуляется
    /// при выходе из области видимости.
    pub fn signing_key_bytes(&self) -> SecretSeed {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&self.signing_key.to_bytes());
        SecretSeed(bytes)
    }

    /// Публичный ключ в hex-формате (64 символа).
    ///
    /// Используется для вычисления приватных топиков (S2-T6)
    /// и как идентификатор в DHT.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }
}
