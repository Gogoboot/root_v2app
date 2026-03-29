// ============================================================
// root-storage — домен хранилища
//
// SQLite, шифрование БД, модели данных, Merkle, PanicButton
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
pub use merkle::MerkleTree;
pub use models::{Contact, Message};
pub use panic::PanicButton;
