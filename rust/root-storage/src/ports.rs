// root-storage/src/ports.rs
// Реализация StoragePort для Database.
// Адаптер между доменным контрактом и SQLite инфраструктурой.
// ============================================================

use root_domain::{
    DomainError,
    StoragePort,
    entities::{Account, Contact as DomainContact, Message as DomainMessage},
};

use crate::database::Database;
use crate::models::Contact;

impl StoragePort for Database {
    // ─── Сообщения ────────────────────────────────────────

    fn save_message(&self, _msg: &DomainMessage) -> Result<(), DomainError> {
        // StoragePort принимает &self, но save_message требует &mut self.
        // Это известное ограничение — решим в следующем спринте через Mutex.
        // Пока оставляем TODO.
        todo!("save_message: требует &mut self — обернуть Database в Mutex")
    }

    fn get_messages(
        &self,
        from_key: &str,
        _to_key: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DomainMessage>, DomainError> {
        let page_size = limit as usize;
        let page = (offset as usize) / page_size.max(1);
        let public_key = from_key; // фильтр по одному ключу

        self.get_messages(public_key, page, page_size)
            .map(|msgs| msgs.into_iter().map(DomainMessage::from).collect())
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn mark_as_read(&self, msg_id: &str) -> Result<(), DomainError> {
        let id: u64 = msg_id
            .parse()
            .map_err(|_| DomainError::Validation {
                field: "msg_id".to_string(),
                message: "не является числом".to_string(),
            })?;
        self.mark_read(id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    // ─── Контакты ─────────────────────────────────────────

    fn save_contact(&self, contact: &DomainContact) -> Result<(), DomainError> {
        let storage_contact = Contact {
            public_key: contact.public_key().to_string(),
            nickname:   contact.nickname().to_string(),
            added_at:   contact.added_at(),
            reputation: contact.reputation(),
        };
        self.add_contact(&storage_contact)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_contacts(&self) -> Result<Vec<DomainContact>, DomainError> {
        self.get_contacts()
            .map(|contacts| contacts.into_iter().map(DomainContact::from).collect())
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn delete_contact(&self, _public_key: &str) -> Result<(), DomainError> {
        todo!("delete_contact: метод ещё не реализован в Database")
    }

    // ─── Аккаунт ──────────────────────────────────────────

    fn save_account(&self, _account: &Account) -> Result<(), DomainError> {
        todo!("save_account: использовать save_identity")
    }

    fn load_account(&self) -> Result<Option<Account>, DomainError> {
        todo!("load_account: использовать load_identity")
    }
}
