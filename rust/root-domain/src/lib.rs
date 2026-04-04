//! Доменный слой RooT Messenger — «конституция проекта».
//!
//! Этот крейт содержит:
//! - **Сущности** — [`Message`], [`Contact`], [`Account`] и их идентификаторы
//! - **Ошибки** — [`DomainError`], единый язык ошибок для всех слоёв
//! - **Порты** — [`StoragePort`], абстракция над хранилищем
//!
//! # Главное правило
//!
//! `root-domain` не зависит ни от одного другого крейта проекта.
//! Только `std` + `thiserror` + `serde`.
//!
//! ```text
//! root-ffi
//!   └── root-domain  ← все знают его
//!   └── root-storage ← реализует StoragePort
//!   └── root-crypto  ← реализует CryptoPort (будущее)
//! ```
//!
//! Если завтра заменить SQLite на RocksDB — `root-domain` не трогаем.
//! Если добавить web-интерфейс — `root-domain` не трогаем.

pub mod entities;
pub mod error;
pub mod ports;

// ─── Реэкспорт ───────────────────────────────────────────────────────────────
// Крейты-потребители пишут:
//   use root_domain::Message;
// вместо:
//   use root_domain::entities::message::Message;

pub use error::DomainError;

pub use entities::{
    Account, AccountId,
    Contact, ContactId,
    Message, MessageId,
};

pub use ports::StoragePort;
