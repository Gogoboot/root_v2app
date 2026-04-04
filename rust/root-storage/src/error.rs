// root-storage/src/error.rs
use thiserror::Error;
use root_crypto::CryptoError;

/// Ошибки модуля хранилища (БД, шифрование, целостность).
/// 
/// Все ошибки реализуют `std::error::Error` и могут быть 
/// автоматически конвертированы в `FfiError` через `?`.
#[derive(Error, Debug)]
pub enum StorageError {
    // ─── Состояние хранилища ─────────────────────────────
    
    /// Попытка использовать БД до вызова `Database::open()`
    #[error("База данных не открыта")]
    NotOpen,
    
    /// Хранилище заблокировано после активации кнопки паники
    #[error("Panic Button активирован — данные уничтожены")]
    PanicButtonActivated,

    // ─── Операции с данными ──────────────────────────────
    
    /// Запрошенное сообщение отсутствует в БД
    #[error("Сообщение не найдено: {0}")]
    MessageNotFound(u64),
    
    /// Попытка добавить контакт с уже существующим ником
    #[error("Контакт с именем '{0}' уже существует")]
    DuplicateNickname(String),

    // ─── База данных (rusqlite) ─────────────────────────
    
    /// Ошибка на уровне SQLite (конвертируется автоматически)
    #[error("Ошибка БД: {0}")]
    Database(#[from] rusqlite::Error),

    // ─── Криптография (делегировано в root-crypto) ─────
    
    /// Ошибка из крипто-подсистемы (автоматическая конвертация)
    #[error("Крипто: {0}")]
    Crypto(#[from] CryptoError),
    
    /// Ошибка управления ключами (загрузка, сохранение, соль)
    #[error("Ошибка управления ключами: {0}")]
    KeyError(String),

    // ─── Целостность (Merkle) ──────────────────────────
    
    /// Дерево Merkle не содержит данных (ошибка инициализации)
    #[error("Merkle tree is empty — no data to verify")]
    MerkleTreeEmpty,
    
    /// Не совпадает хеш в дереве Merkle — данные повреждены
    #[error("Нарушение целостности данных")]  // ← Исправлена опечатка!
    MerkleVerificationFailed,

    // ─── Сериализация ──────────────────────────────────
    
    /// Ошибка при преобразовании структуры в байты
    #[error("Ошибка сериализации данных")]
    SerializationFailed,
    
    /// Ошибка при восстановлении структуры из байтов
    #[error("Ошибка десериализации данных")]
    DeserializationFailed,
}

// 🔧 Опционально: методы-помощники
impl StorageError {
    pub fn code(&self) -> &'static str {
        match self {
            StorageError::NotOpen => "storage.not_open",
            StorageError::PanicButtonActivated => "storage.panic",
            StorageError::MessageNotFound(_) => "storage.not_found",
            StorageError::DuplicateNickname(_) => "storage.duplicate",
            StorageError::Database(_) => "storage.db_error",
            StorageError::Crypto(e) => e.code(),  // делегируем в CryptoError
            StorageError::KeyError(_) => "storage.key_error",
            StorageError::MerkleTreeEmpty => "storage.merkle_empty",
            StorageError::MerkleVerificationFailed => "storage.integrity",
            StorageError::SerializationFailed => "storage.serialize",
            StorageError::DeserializationFailed => "storage.deserialize",
        }
    }

    /// Можно ли повторить операцию при этой ошибке?
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, 
            StorageError::PanicButtonActivated | 
            StorageError::MerkleVerificationFailed
        )
    }
}
