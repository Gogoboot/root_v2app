// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — storage/error.rs
// ═══════════════════════════════════════════════════════════

use thiserror::Error;
use root_crypto::CryptoError;  // ← Импортируем!



#[derive(Error, Debug)]
pub enum StorageError {
    #[error("База данных не открыта")]
    NotOpen,
    
    #[error("Сообщение не найдено: {0}")]
    MessageNotFound(u64),
    
    #[error("Ошибка БД: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Контакт с именем {0} уже существует")]
    DuplicateNickname(String), 
    
    // ✅ ЗАМЕНЯЕМ строку Crypto(String) на типизированную ошибку:
    #[error("Крипто: {0}")]
    Crypto(#[from] CryptoError),  // ← Автоматическая конвертация!
    
    #[error("Ошибка управления ключами: {0}")]
    KeyError(String),
    
    /// Дерево Merkle не содержит данных (ошибка инициализации/логики)
    #[error("Merkle tree is empty — no data to verify")]
    MerkleTreeEmpty,
   
    #[error("Ошибка сериализации")]
    SerializationFailed,
    
    #[error("Ошибка десериализации")]
    DeserializationFailed,

    #[error("Нарушении целостности данных")]
    MerkleVerificationFailed,

    #[error("Panic Button активирован — данные уничтожены")]
    PanicButtonActivated,
}

