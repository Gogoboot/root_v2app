// ============================================================
// ROOT v2.0 — storage/models.rs
// Инфраструктурные модели данных.
//
// Важно: storage::Message и domain::Message — разные типы.
//   storage::Message.id = Option<u64> (rowid SQLite)
//   domain::Message.id  = MessageId   (SHA-256 хеш)
// ============================================================

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

// ─── Message (инфраструктурная) ──────────────────────────────────────────────

/// Сообщение как оно хранится в SQLite.
///
/// Отличается от [`root_domain::Message`]:
/// - `id` здесь — это rowid SQLite (`Option<u64>`), не SHA-256
/// - поля публичные — это внутренняя структура хранилища
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    /// Внутренний идентификатор SQLite (rowid).
    /// `None` до сохранения в БД — присваивается автоматически.
    pub id: Option<u64>,
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
    /// Создаёт новое сообщение без id — он присвоится SQLite при сохранении.
    pub fn new(from_key: String, to_key: String, content: String) -> Self {
        Message {
            id: None,
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

    /// SHA-256 хеш сообщения для построения Merkle Tree.
    ///
    /// Хешируются все значимые поля — изменение любого из них
    /// изменит хеш и Merkle Tree это обнаружит.
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

    /// Вычисляет SHA-256 идентификатор для доменного слоя.
    ///
    /// Это отдельный хеш от [`Message::hash`] — он не включает rowid,
    /// только бизнес-данные. Используется как `MessageId` в domain.
    fn compute_message_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.from_key.as_bytes());
        hasher.update(self.to_key.as_bytes());
        hasher.update(self.content.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ─── Contact (инфраструктурная) ──────────────────────────────────────────────

/// Контакт как он хранится в SQLite.
///
/// Поля публичные — внутренняя структура хранилища.
/// Для передачи в бизнес-логику конвертируется в [`root_domain::Contact`].
#[derive(Debug, Clone)]
pub struct Contact {
    pub public_key: String,
    pub nickname: String,
    pub added_at: u64,
    pub reputation: u8,
}

// ─── Конвертация storage → domain ────────────────────────────────────────────

use root_domain::{
    Contact as DomainContact,
    Message as DomainMessage,
    MessageId,
};

impl From<Message> for DomainMessage {
    /// Конвертирует инфраструктурное сообщение в доменное.
    ///
    /// SHA-256 id вычисляется здесь из полей сообщения —
    /// rowid SQLite в доменный слой не передаётся.
    fn from(m: Message) -> Self {
        // Вычисляем SHA-256 id для доменного слоя
        let message_id = MessageId::new(m.compute_message_id());

        DomainMessage::new(
            message_id,
            m.from_key,
            m.to_key,
            m.content,
            m.timestamp,
        )
    }
}

impl From<Contact> for DomainContact {
    /// Конвертирует инфраструктурный контакт в доменный.
    ///
    /// Использует конструктор [`DomainContact::new`] —
    /// прямой доступ к полям закрыт (они приватные).
    fn from(c: Contact) -> Self {
        // Сначала создаём контакт через конструктор
        let mut contact = DomainContact::new(
            c.public_key,
            c.nickname,
            c.added_at,
        );
        // Затем восстанавливаем reputation из БД
        contact.set_reputation(c.reputation);
        contact
    }
}
