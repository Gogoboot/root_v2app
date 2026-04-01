// ============================================================
// ROOT v2.0 — storage/constants.rs
// ============================================================

/// Длина ключа шифрования (AES-256 = 32 байта)
pub const KEY_LEN: usize = 32;

/// Argon2id параметры — баланс безопасности и скорости
pub const ARGON2_MEMORY_KB: u32 = 65536; // 64 MB
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;

/// Salt для Argon2 (в продакшн — уникальный для каждого устройства)
pub const ARGON2_SALT: &[u8] = b"ROOT_v2_storage_salt_2026";

/// Pepper — дополнительный секрет в коде (не соль!)
/// Используется ДОПОЛНИТЕЛЬНО к уникальной соли из SaltManager
pub const ARGON2_PEPPER: &[u8] = b"ROOT_v2_pepper_x7k9m2p4q8r1t5w3"; // ← Не соль!

/// Имя файла базы данных по умолчанию
pub const DB_FILENAME: &str = "root_messages.db";

/// Максимум сообщений в памяти для Merkle Tree
pub const MAX_MESSAGES_IN_MEMORY: usize = 10_000;
