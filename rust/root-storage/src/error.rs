//! Типы ошибок модуля `root-storage`.
//!
//! Центральный тип — [`StorageError`]. Он охватывает все возможные
//! сбои: состояние БД, операции с данными, криптографию, целостность
//! и сериализацию.
//!
//! # Конвертация ошибок
//!
//! Благодаря атрибуту `#[from]`, оператор `?` автоматически
//! преобразует сторонние ошибки в [`StorageError`]:
//!
//! ```rust
//! fn example(conn: &Connection) -> Result<(), StorageError> {
//!     conn.execute("...", [])?; // rusqlite::Error → StorageError::Database
//!     Ok(())
//! }
//! ```

use thiserror::Error;
use root_crypto::CryptoError;

/// Все возможные ошибки подсистемы хранилища.
///
/// Реализует [`std::error::Error`] через макрос `thiserror`,
/// поэтому может использоваться с оператором `?` в любой функции
/// возвращающей `Result<_, StorageError>`.
///
/// Для передачи через FFI-границу используй метод [`StorageError::code`],
/// который возвращает стабильную строку-идентификатор ошибки.
#[derive(Error, Debug)]
pub enum StorageError {

    // ─── Состояние хранилища ─────────────────────────────────────────────

    /// БД не была открыта перед использованием.
    ///
    /// Возникает если вызвать методы [`Database`] до [`Database::open()`].
    #[error("База данных не открыта")]
    NotOpen,

    /// Хранилище уничтожено после активации кнопки паники.
    ///
    /// После этой ошибки восстановление невозможно —
    /// см. [`StorageError::is_recoverable`].
    #[error("Panic Button активирован — данные уничтожены")]
    PanicButtonActivated,

    // ─── Операции с данными ──────────────────────────────────────────────

    /// Сообщение с указанным числовым ID не найдено в БД.
    ///
    /// Число внутри — это внутренний rowid SQLite, не SHA-256 хеш.
    #[error("Сообщение не найдено: {0}")]
    MessageNotFound(u64),

    /// Попытка добавить контакт с именем которое уже занято.
    ///
    /// Строка внутри — дублирующийся никнейм.
    #[error("Контакт с именем '{0}' уже существует")]
    DuplicateNickname(String),

    // ─── База данных ─────────────────────────────────────────────────────

    /// Низкоуровневая ошибка SQLite.
    ///
    /// Создаётся автоматически через `?` благодаря `#[from]`.
    /// Это означает: `impl From<rusqlite::Error> for StorageError`
    /// генерируется макросом — писать руками не нужно.
    #[error("Ошибка БД: {0}")]
    Database(#[from] rusqlite::Error),

    // ─── Криптография ────────────────────────────────────────────────────

    /// Ошибка из крейта `root-crypto`.
    ///
    /// Тоже создаётся автоматически через `#[from]`.
    /// Делегирует строковое представление в [`CryptoError`].
    #[error("Крипто: {0}")]
    Crypto(#[from] CryptoError),

    /// Ошибка при работе с ключами шифрования.
    ///
    /// Строка внутри содержит детали: что именно не удалось
    /// (загрузка, сохранение, вычисление соли и т.д.).
    #[error("Ошибка управления ключами: {0}")]
    KeyError(String),

    // ─── Целостность (Merkle) ────────────────────────────────────────────

    /// Дерево Merkle пустое — верификация невозможна.
    ///
    /// Обычно означает что БД была создана но ни одна запись
    /// ещё не добавлена в дерево.
    #[error("Merkle tree is empty — no data to verify")]
    MerkleTreeEmpty,

    /// Хеш в дереве Merkle не совпал — данные повреждены или подделаны.
    ///
    /// Это критическая ошибка: после неё продолжать работу небезопасно.
    /// [`StorageError::is_recoverable`] вернёт `false`.
    #[error("Нарушение целостности данных")]
    MerkleVerificationFailed,

    // ─── Сериализация ────────────────────────────────────────────────────

    /// Не удалось преобразовать структуру в байты для записи в БД.
    #[error("Ошибка сериализации данных")]
    SerializationFailed,

    /// Не удалось восстановить структуру из байт прочитанных из БД.
    #[error("Ошибка десериализации данных")]
    DeserializationFailed,

    // ─── Внутренние ошибки ───────────────────────────────────────────────

    /// Внутренняя ошибка инфраструктуры (например: отравленный мьютекс).
    ///
    /// Используется в [`crate::test_utils::InMemoryStorage`] и других
    /// местах где нет подходящего варианта выше.
    /// Строка внутри — произвольное описание для отладки.
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

// ─── Вспомогательные методы ──────────────────────────────────────────────────

impl StorageError {
    /// Возвращает стабильный строковый код ошибки.
    ///
    /// Коды используются на FFI-границе и во Flutter-клиенте
    /// для обработки конкретных ситуаций без разбора текстовых сообщений.
    ///
    /// # Стабильность
    ///
    /// Коды не должны меняться между версиями — Flutter-код может
    /// зависеть от них. Добавлять новые можно, переименовывать нельзя.
    ///
    /// # Пример
    ///
    /// ```rust
    /// let err = StorageError::NotOpen;
    /// assert_eq!(err.code(), "storage.not_open");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            StorageError::NotOpen                  => "storage.not_open",
            StorageError::PanicButtonActivated     => "storage.panic",
            StorageError::MessageNotFound(_)       => "storage.not_found",
            StorageError::DuplicateNickname(_)     => "storage.duplicate",
            StorageError::Database(_)              => "storage.db_error",
            // Делегируем код в CryptoError — он сам знает свой код
            StorageError::Crypto(e)                => e.code(),
            StorageError::KeyError(_)              => "storage.key_error",
            StorageError::MerkleTreeEmpty          => "storage.merkle_empty",
            StorageError::MerkleVerificationFailed => "storage.integrity",
            StorageError::SerializationFailed      => "storage.serialize",
            StorageError::DeserializationFailed    => "storage.deserialize",
            StorageError::Internal(_)              => "storage.internal",
        }
    }

    /// Показывает можно ли повторить операцию после этой ошибки.
    ///
    /// Возвращает `false` для двух необратимых ситуаций:
    /// - [`StorageError::PanicButtonActivated`] — данные уничтожены навсегда
    /// - [`StorageError::MerkleVerificationFailed`] — данные повреждены,
    ///   продолжать небезопасно
    ///
    /// Для всех остальных ошибок возвращает `true` — операцию
    /// теоретически можно попробовать снова.
    ///
    /// # Пример
    ///
    /// ```rust
    /// if !err.is_recoverable() {
    ///     // показать пользователю критическое сообщение
    ///     // и завершить работу
    /// }
    /// ```
    pub fn is_recoverable(&self) -> bool {
        // matches! проверяет паттерн и возвращает bool.
        // ! инвертирует: recoverable = НЕ является одним из критических вариантов.
        !matches!(
            self,
            StorageError::PanicButtonActivated | StorageError::MerkleVerificationFailed
        )
    }
}
