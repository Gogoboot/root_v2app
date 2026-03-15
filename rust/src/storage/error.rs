// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — storage/error.rs
// ═══════════════════════════════════════════════════════════

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("База данных не открыта")]
    NotOpen,
    
    #[error("Сообщение не найдено: {0}")]
    MessageNotFound(u64),
    
    #[error("Ошибка БД: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Крипто ошибка: {0}")]
    Crypto(String),
    
    #[error("Ошибка деривации ключа")]
    KeyDerivationFailed,
    
    #[error("Ошибка шифрования")]
    EncryptionFailed,
    
    #[error("Ошибка расшифровки")]
    DecryptionFailed,
    
    #[error("Ошибка сериализации")]
    SerializationFailed,
    
    #[error("Ошибка десериализации")]
    DeserializationFailed,

    #[error("Panic Button активирован — данные уничтожены")]
    PanicButtonActivated,
}

