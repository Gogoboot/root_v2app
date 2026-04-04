// root-identity/src/error.rs
use thiserror::Error;

/// Ошибки модуля управления идентичностью (ключи, мнемоника, Shamir).
/// 
/// Все ошибки реализуют `std::error::Error` и могут быть 
/// автоматически конвертированы в `StorageError` или `FfiError` через `#[from]`.
#[derive(Error, Debug)]
pub enum IdentityError {
    // ─── Ключи и мнемоника ───────────────────────────────
    
    /// Мнемоническая фраза невалидна (не проходит проверку BIP-39)
    #[error("Неверная мнемоническая фраза")]
    InvalidMnemonic,
    
    /// Не удалось сгенерировать ключевую пару
    #[error("Ошибка генерации ключевой пары")]
    KeyGenerationFailed,
    
    /// Публичный ключ в неверном формате (не hex / не валидная кривая)
    #[error("Неверный формат публичного ключа: {0}")]
    InvalidPublicKey(String),
    
    /// Приватный ключ не найден в защищённом хранилище
    #[error("Приватный ключ не найден")]
    PrivateKeyNotFound,

    // ─── Shamir's Secret Sharing ─────────────────────────
    
    /// Недостаточно шардов для восстановления секрета
    #[error("Недостаточно шардов: нужно {needed}, есть {have}")]
    InsufficientShares { needed: u8, have: u8 },
    
    /// Один или несколько шардов повреждены / невалидны
    #[error("Невалидный шард: {0}")]
    InvalidShare(String),
    
    /// Ошибка при сборке секрета из шардов
    #[error("Не удалось восстановить секрет из шардов")]
    SecretReconstructionFailed,

    // ─── Защищённое хранение ─────────────────────────────
    
    /// Ошибка при сохранении данных в защищённое хранилище
    #[error("Ошибка сохранения в protected storage: {0}")]
    ProtectedStorageSave(String),
    
    /// Ошибка при чтении из защищённого хранилища
    #[error("Ошибка чтения из protected storage: {0}")]
    ProtectedStorageLoad(String),

    // ─── Крипто-зависимости (делегирование) ─────────────
    
    /// Ошибка из крипто-подсистемы (автоматическая конвертация)
    #[error("Крипто: {0}")]
    Crypto(#[from] root_crypto::CryptoError),

    // ─── Общие ───────────────────────────────────────────
    
    /// Динамическая ошибка с описанием (для редких случаев)
    #[error("Ошибка идентичности: {0}")]
    Other(String),
}

// 🔧 Методы-помощники для агрегатора / логирования
impl IdentityError {
    /// Код ошибки для метрик / API-ответов
    pub fn code(&self) -> &'static str {
        match self {
            IdentityError::InvalidMnemonic => "identity.invalid_mnemonic",
            IdentityError::KeyGenerationFailed => "identity.key_gen",
            IdentityError::InvalidPublicKey(_) => "identity.invalid_pubkey",
            IdentityError::PrivateKeyNotFound => "identity.key_not_found",
            IdentityError::InsufficientShares { .. } => "identity.insufficient_shares",
            IdentityError::InvalidShare(_) => "identity.invalid_share",
            IdentityError::SecretReconstructionFailed => "identity.reconstruct",
            IdentityError::ProtectedStorageSave(_) => "identity.storage_save",
            IdentityError::ProtectedStorageLoad(_) => "identity.storage_load",
            IdentityError::Crypto(e) => e.code(),  // делегируем в CryptoError
            IdentityError::Other(_) => "identity.other",
        }
    }
}
