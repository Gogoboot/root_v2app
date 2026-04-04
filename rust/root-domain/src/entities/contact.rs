// root-domain/src/entities/contact.rs
// Доменная сущность контакта.
// ============================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    /// Публичный ключ (hex) — основной идентификатор
    pub public_key: String,
    pub nickname: String,
    pub added_at: u64,
    pub reputation: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContactId(pub String);

impl std::fmt::Display for ContactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
