// ============================================================
// ROOT v2.0 — storage/constants.rs
// ============================================================

/// Длина ключа шифрования (AES-256 = 32 байта)
pub const KEY_LEN: usize = 32;

/// Имя файла базы данных по умолчанию
pub const DB_FILENAME: &str = "root_messages.db";

/// Максимум сообщений в памяти для Merkle Tree
pub const MAX_MESSAGES_IN_MEMORY: usize = 10_000;
