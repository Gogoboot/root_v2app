// root-domain/src/entities/message.rs
// Доменная сущность сообщения.
// Не содержит логики хранения или шифрования.
// ============================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// SHA-256 хеш — уникальный идентификатор сообщения
    pub id: Option<String>,
    /// Публичный ключ отправителя (hex)
    pub from_key: String,
    /// Публичный ключ получателя (hex)
    pub to_key: String,
    /// Содержимое (зашифровано на уровне инфраструктуры)
    pub content: String,
    pub timestamp: u64,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
