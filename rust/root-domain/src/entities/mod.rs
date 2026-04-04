// root-domain/src/entities/mod.rs

pub mod account;
pub mod contact;
pub mod message;

pub use account::{Account, AccountId};
pub use contact::{Contact, ContactId};
pub use message::{Message, MessageId};
