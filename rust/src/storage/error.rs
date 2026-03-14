// ============================================================
// ROOT v2.0 — storage/error.rs
// ============================================================

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Ошибка базы данных: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Ошибка шифрования: {0}")]
    Crypto(String),

    #[error("Panic Button активирован — база данных уничтожена")]
    PanicButtonActivated,

    #[error("База данных не открыта")]
    NotOpen,

    #[error("Ошибка целостности: Merkle root не совпадает")]
    IntegrityError,

    #[error("Сообщение не найдено: id={0}")]
    MessageNotFound(u64),
}
