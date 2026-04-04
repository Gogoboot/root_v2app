// root-crypto/src/error.rs
use thiserror::Error;

/// Ошибки криптографического модуля.
/// 
/// Все варианты реализуют `std::error::Error` и могут быть 
/// автоматически конвертированы в `StorageError` через `#[from]`.
#[derive(Error, Debug)]
pub enum CryptoError {
    // ─── Деривация ключа (Argon2) ─────────────
    
    /// Не удалось вывести ключ из пароля (неверный пароль/соль)
    #[error("Не удалось вывести ключ из пароля")]
    DerivationFailed,
    
    // ─── Симметричное шифрование ───────────────
    
    /// Ошибка шифрования данных (ChaCha20-Poly1305)
    #[error("Ошибка шифрования данных")]
    EncryptionFailed,
    
    /// Ошибка расшифровки (неверный ключ или повреждённые данные)
    #[error("Ошибка расшифровки данных")]
    DecryptionFailed,
    
    /// Неверные параметры ключа или nonce
    #[error("Неверные параметры ключа или nonce")]
    InvalidKeyParams,
    
    // ─── Асимметричная криптография ────────────
    
    /// Ошибка создания цифровой подписи
    #[error("Ошибка создания подписи")]
    SignFailed,
    
    /// Ошибка проверки цифровой подписи
    #[error("Ошибка проверки подписи")]
    VerifyFailed,
    
    /// Неверный формат публичного ключа
    #[error("Неверный формат публичного ключа")]
    InvalidPublicKey,
    
    // ─── Сериализация/упаковка ─────────────────
    
    /// Ошибка упаковки данных для хранения (nonce + ciphertext)
    #[error("Ошибка упаковки данных для хранения")]
    PackFailed,
    
    /// Ошибка распаковки данных из хранилища
    #[error("Ошибка распаковки данных")]
    UnpackFailed,
    
    // ─── Общие ─────────────────────────────────
    
    /// Динамическая ошибка с описанием (для редких случаев)
    #[error("Крипто ошибка: {0}")]
    Other(String),

    /// Ошибка декодирования hex-строки
    #[error("Ошибка декодирования hex-строки")]
    HexDecodeFailed,

    /// Данные повреждены или обрезаны — невозможно распаковать
    #[error("Данные повреждены или обрезаны — их невозможно распаковать")]
    InvalidBlob,
}

// 🔧 Опционально: методы-помощники для агрегатора
impl CryptoError {
    /// Код ошибки для логирования / API-ответов
    pub fn code(&self) -> &'static str {
        match self {
            CryptoError::DerivationFailed => "crypto.derivation",
            CryptoError::EncryptionFailed => "crypto.encrypt",
            CryptoError::DecryptionFailed => "crypto.decrypt",
            CryptoError::InvalidKeyParams => "crypto.invalid_params",
            CryptoError::SignFailed => "crypto.sign",
            CryptoError::VerifyFailed => "crypto.verify",
            CryptoError::InvalidPublicKey => "crypto.invalid_pubkey",
            CryptoError::PackFailed => "crypto.pack",
            CryptoError::UnpackFailed => "crypto.unpack",
            CryptoError::HexDecodeFailed => "crypto.hex_decode",
            CryptoError::InvalidBlob => "crypto.invalid_blob",
            CryptoError::Other(_) => "crypto.other",
        }
    }
}
