// ============================================================
// ROOT v2.0 — storage/models.rs
// Message, Contact — структуры данных
// ============================================================

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone, Debug)]  // ← добавить
pub struct Message {
    pub id: Option<u64>,  // ← Option, не u64
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
            id: None, // присваивается базой данных
            from_key,
            to_key,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_read: false,
        }
    }

    /// SHA256 хеш сообщения для Merkle Tree
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        if let Some(id) = self.id {
            hasher.update(id.to_le_bytes());
        }
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

// ─── Конвертация между storage и domain моделями ─────────────

use root_domain::entities::{
    Contact as DomainContact,
    Message as DomainMessage,
};

impl From<Message> for DomainMessage {
    fn from(m: Message) -> Self {
        DomainMessage {
            id: m.id.map(|id| id.to_string()),
            from_key: m.from_key,
            to_key: m.to_key,
            content: m.content,
            timestamp: m.timestamp,
            is_read: m.is_read,
        }
    }
}

impl From<Contact> for DomainContact {
    fn from(c: Contact) -> Self {
        DomainContact {
            public_key: c.public_key,
            nickname: c.nickname,
            added_at: c.added_at,
            reputation: c.reputation,
        }
    }
}

// fn now_secs() -> u64 {
//     SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs()
// }
