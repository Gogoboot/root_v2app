// root-crypto/src/error.rs
use thiserror::Error;

/// Ошибки криптографического модуля
#[derive(Error, Debug)]
pub enum CryptoError {
    // ─── Деривация ключа (Argon2) ─────────────
    #[error("Не удалось вывести ключ из пароля")]
    DerivationFailed,
    
    // ─── Симметричное шифрование ───────────────
    #[error("Ошибка шифрования данных")]
    EncryptionFailed,
    
    #[error("Ошибка расшифровки данных")]
    DecryptionFailed,
    
    #[error("Неверные параметры ключа или nonce")]
    InvalidKeyParams,
    
    // ─── Асимметричная криптография ────────────
    #[error("Ошибка создания подписи")]
    SignFailed,
    
    #[error("Ошибка проверки подписи")]
    VerifyFailed,
    
    #[error("Неверный формат публичного ключа")]
    InvalidPublicKey,
    
    // ─── Сериализация/упаковка ─────────────────
    #[error("Ошибка упаковки данных для хранения")]
    PackFailed,
    
    #[error("Ошибка распаковки данных")]
    UnpackFailed,
    
    // ─── Общие ─────────────────────────────────
    #[error("Крипто ошибка: {0}")]
    Other(String),

    #[error("Ошибка декодирования hex-строки")]
    HexDecodeFailed,

    #[error("Данные повреждены или обрезаны — их невозможно распаковать")]
    InvalidBlob,  // ← Добавьте эту строку, если её нет
}
