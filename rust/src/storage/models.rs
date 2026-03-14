// ============================================================
// ROOT v2.0 — storage/models.rs
// Message, Contact — структуры данных
// ============================================================

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    /// Публичный ключ отправителя (hex)
    pub from_key: String,
    /// Публичный ключ получателя (hex)
    pub to_key: String,
    /// Содержимое (в будущем — E2E зашифровано)
    pub content: String,
    pub timestamp: u64,
    pub is_read: bool,
}

impl Message {
    pub fn new(from_key: String, to_key: String, content: String) -> Self {
        Message {
            id: 0, // присваивается базой данных
            from_key,
            to_key,
            content,
            timestamp: now_secs(),
            is_read: false,
        }
    }

    /// SHA256 хеш сообщения для Merkle Tree
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.id.to_le_bytes());
        hasher.update(self.from_key.as_bytes());
        hasher.update(self.to_key.as_bytes());
        hasher.update(self.content.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub public_key: String,
    pub nickname: String,
    pub added_at: u64,
    pub reputation: u8,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
