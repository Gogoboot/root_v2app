// root-domain/src/lib.rs
// Конституция проекта ROOT.
// Зависимости: только std + thiserror + serde.
// Никаких tokio, libp2p, rusqlite, root-crypto, root-storage.
// ============================================================

pub mod entities;
pub mod error;
pub mod ports;

// Удобный реэкспорт для крейтов-потребителей
pub use error::DomainError;
pub use entities::{Account, AccountId, Contact, ContactId, Message, MessageId};
pub use ports::StoragePort;


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
