//! Доменная сущность сообщения.
//!
//! Этот модуль знает **только** о структуре сообщения с точки зрения
//! бизнес-логики. Никакого SQLite, никакого шифрования, никакого libp2p.
//!
//! # Жизненный цикл сообщения
//!
//! ```text
//! root-crypto: вычисляет SHA-256 → создаёт MessageId
//!      ↓
//! root-domain: Message::new(id, ...) → доменная сущность
//!      ↓
//! root-storage: сохраняет в SQLite
//! ```

use serde::{Deserialize, Serialize};

/// Уникальный идентификатор сообщения.
///
/// Внутри — SHA-256 хеш, вычисленный в `root-crypto`
/// **до** создания доменной сущности.
///
/// Паттерн **newtype**: обёртка над `String` чтобы компилятор
/// не позволил перепутать идентификатор сообщения с обычной строкой.
///
/// `Hash` + `Eq` позволяют использовать как ключ в `HashMap`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    /// Создаёт `MessageId` из готового SHA-256 хеша.
    ///
    /// Вызывается из `root-crypto` или `root-messaging` —
    /// не из `root-domain`.
    pub fn new(hash: String) -> Self {
        Self(hash)
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Message ─────────────────────────────────────────────────────────────────

/// Доменная сущность сообщения в RooT Messenger.
///
/// Создаётся только через [`Message::new`] — прямая инициализация
/// полей запрещена (поля приватные).
///
/// # Что domain НЕ делает
///
/// - Не вычисляет SHA-256 — это делает `root-crypto`
/// - Не шифрует содержимое — приходит уже зашифрованным
/// - Не знает про SQLite — сохранение в `root-storage`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Уникальный идентификатор — SHA-256 хеш.
    ///
    /// Вычисляется снаружи (в `root-crypto`) и передаётся
    /// готовым в [`Message::new`]. Domain доверяет что хеш корректный.
    id: MessageId,

    /// Публичный ключ отправителя в hex-формате.
    ///
    /// Используется для верификации подписи и маршрутизации через DHT.
    from_key: String,

    /// Публичный ключ получателя в hex-формате.
    ///
    /// По этому ключу получатель находится в DHT Kademlia.
    to_key: String,

    /// Зашифрованное содержимое сообщения.
    ///
    /// Domain не знает что внутри — это намеренно.
    /// Расшифровка в `root-crypto` на стороне получателя.
    content: String,

    /// Время отправки — Unix timestamp в миллисекундах.
    ///
    /// Миллисекунды (не секунды) — секундной точности недостаточно
    /// при быстром обмене сообщениями.
    timestamp: u64,

    /// Прочитано ли сообщение получателем.
    ///
    /// `false` при создании. Становится `true` когда получатель
    /// открыл чат и сообщение отобразилось на экране.
    is_read: bool,
}

impl Message {
    /// Создаёт новое сообщение из готового идентификатора.
    ///
    /// `id` должен быть вычислен в `root-crypto` до вызова этого метода.
    /// `is_read` всегда `false` — новое сообщение непрочитано.
    ///
    /// # Пример
    ///
    /// ```rust
    /// // В root-crypto:
    /// let id = MessageId::new(sha256(from_key + to_key + content + timestamp));
    ///
    /// // В root-domain:
    /// let msg = Message::new(id, from_key, to_key, content, timestamp);
    /// ```
    pub fn new(
        id: MessageId,
        from_key: String,
        to_key: String,
        content: String,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            from_key,
            to_key,
            content,
            timestamp,
            is_read: false,
        }
    }

    // ─── Геттеры ─────────────────────────────────────────────────────────
    // Поля приватные — доступ только через методы.
    // Это защищает инварианты: никто не изменит id или content напрямую.

    /// Возвращает идентификатор сообщения.
    pub fn id(&self) -> &MessageId {
        &self.id
    }

    /// Возвращает публичный ключ отправителя.
    pub fn from_key(&self) -> &str {
        &self.from_key
    }

    /// Возвращает публичный ключ получателя.
    pub fn to_key(&self) -> &str {
        &self.to_key
    }

    /// Возвращает зашифрованное содержимое.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Возвращает время отправки (Unix ms).
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Возвращает статус прочтения.
    pub fn is_read(&self) -> bool {
        self.is_read
    }

    // ─── Мутация состояния ───────────────────────────────────────────────

    /// Помечает сообщение как прочитанное.
    ///
    /// Единственный способ изменить `is_read` — через этот метод.
    /// Прямой доступ к полю закрыт намеренно.
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}
