//! Доменная сущность контакта.
//!
//! Контакт идентифицируется по публичному ключу (`public_key`),
//! а не по нику — ник может меняться, ключ нет.

use serde::{Deserialize, Serialize};

/// Уникальный идентификатор контакта.
///
/// Паттерн **newtype** — обёртка над публичным ключом в hex-формате.
/// Компилятор не позволит перепутать `ContactId` с обычной строкой.
///
/// `Hash` + `Eq` позволяют использовать как ключ в `HashMap`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContactId(pub String);

impl ContactId {
    /// Создаёт `ContactId` из публичного ключа в hex-формате.
    pub fn new(public_key: String) -> Self {
        Self(public_key)
    }
}

impl std::fmt::Display for ContactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Contact ─────────────────────────────────────────────────────────────────

/// Доменная сущность контакта в RooT Messenger.
///
/// Создаётся только через [`Contact::new`].
/// Поля приватные — изменение только через методы.
///
/// # Идентификация
///
/// Основной идентификатор — `public_key`, не `nickname`.
/// Ник может меняться, публичный ключ — никогда.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    /// Публичный ключ в hex-формате — уникальный идентификатор контакта.
    ///
    /// Соответствует `peer_id` в libp2p сети.
    public_key: String,

    /// Человекочитаемое имя контакта.
    ///
    /// Задаётся пользователем, не уникально, может меняться.
    nickname: String,

    /// Время добавления контакта — Unix timestamp в миллисекундах.
    added_at: u64,

    /// Репутация контакта.
    ///
    /// Диапазон: 0..=255.
    ///
    /// TODO: семантика не определена. Планируется в рамках
    /// экономического слоя (root-economy, Sprint 3+).
    /// Пока всегда инициализируется как 0.
    reputation: u8,
}

impl Contact {
    /// Создаёт новый контакт.
    ///
    /// `reputation` инициализируется нулём — семантика будет
    /// определена в Sprint 3 (root-economy).
    ///
    /// # Пример
    ///
    /// ```rust
    /// let contact = Contact::new(
    ///     "a1b2c3...".to_string(), // публичный ключ
    ///     "Alice".to_string(),
    ///     1_700_000_000_000,       // Unix ms
    /// );
    /// ```
    pub fn new(public_key: String, nickname: String, added_at: u64) -> Self {
        Self {
            public_key,
            nickname,
            added_at,
            reputation: 0,
        }
    }

    /// Возвращает `ContactId` для этого контакта.
    ///
    /// `ContactId` строится из публичного ключа — они всегда совпадают.
    pub fn id(&self) -> ContactId {
        ContactId::new(self.public_key.clone())
    }

    // ─── Геттеры ─────────────────────────────────────────────────────────

    /// Возвращает публичный ключ в hex-формате.
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Возвращает никнейм контакта.
    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    /// Возвращает время добавления (Unix ms).
    pub fn added_at(&self) -> u64 {
        self.added_at
    }

    /// Возвращает репутацию контакта (0..=255).
    ///
    /// Семантика будет определена в Sprint 3.
    pub fn reputation(&self) -> u8 {
        self.reputation
    }

    // ─── Мутация состояния ───────────────────────────────────────────────

    /// Обновляет никнейм контакта.
    ///
    /// Единственный способ изменить ник — прямой доступ к полю закрыт.
    pub fn rename(&mut self, new_nickname: String) {
        self.nickname = new_nickname;
    }

    /// Обновляет репутацию контакта.
    ///
    /// TODO: заменить на бизнес-метод когда будет определена
    /// семантика репутации в Sprint 3.
    pub fn set_reputation(&mut self, value: u8) {
        self.reputation = value;
    }
}
