// root-domain/src/ports/storage.rs
// StoragePort — абстракция хранилища.
// root-ffi работает только с этим трейтом, не с Database напрямую.
// ============================================================

use crate::entities::{Account, Contact, Message};
use crate::error::DomainError;

pub trait StoragePort: Send {
    // ─── Сообщения ────────────────────────────────────────
    fn save_message(&self, msg: &Message) -> Result<(), DomainError>;
    fn get_messages(
        &self,
        from_key: &str,
        to_key: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>, DomainError>;
    fn mark_as_read(&self, msg_id: &str) -> Result<(), DomainError>;

    // ─── Контакты ─────────────────────────────────────────
    fn save_contact(&self, contact: &Contact) -> Result<(), DomainError>;
    fn get_contacts(&self) -> Result<Vec<Contact>, DomainError>;
    fn delete_contact(&self, public_key: &str) -> Result<(), DomainError>;

    // ─── Аккаунт ──────────────────────────────────────────
    fn save_account(&self, account: &Account) -> Result<(), DomainError>;
    fn load_account(&self) -> Result<Option<Account>, DomainError>;
}
