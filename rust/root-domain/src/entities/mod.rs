//! Доменные сущности RooT Messenger.
//!
//! Каждая сущность — это объект бизнес-логики.
//! Никакой логики хранения, шифрования или сети.
//!
//! | Сущность  | Идентификатор | Описание                        |
//! |-----------|---------------|---------------------------------|
//! | [`Message`]  | [`MessageId`]  | Зашифрованное сообщение      |
//! | [`Contact`]  | [`ContactId`]  | Контакт в адресной книге     |
//! | [`Account`]  | [`AccountId`]  | Аккаунт текущего пользователя|

pub mod account;
pub mod contact;
pub mod message;

pub use account::{Account, AccountId};
pub use contact::{Contact, ContactId};
pub use message::{Message, MessageId};
