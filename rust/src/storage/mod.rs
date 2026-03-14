// ============================================================
// ROOT v2.0 — storage/mod.rs
//
// Подмодули:
//   constants — константы (KEY_LEN, Argon2 параметры...)
//   error     — StorageError
//   key       — StorageKey (Argon2id деривация)
//   models    — Message, Contact (структуры данных)
//   merkle    — MerkleTree (верификация целостности)
//   panic     — PanicButton
//   database  — Database (главный движок SQLite)
// ============================================================

pub mod constants;
pub mod database;
pub mod error;
pub mod key;
pub mod merkle;
pub mod models;
pub mod panic;

pub use constants::*;
pub use database::Database;
pub use error::StorageError;
pub use key::StorageKey;
pub use merkle::MerkleTree;
pub use models::{Contact, Message};
pub use panic::PanicButton;
