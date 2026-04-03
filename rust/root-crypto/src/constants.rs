// ============================================================
// root-crypto — constants.rs
// Криптографические константы
// ============================================================

/// Pepper для Argon2id — фиксированный секрет уровня приложения
pub const ARGON2_PEPPER: &[u8] = b"ROOT_v2_PEPPER_2026";

/// Argon2id параметры — баланс безопасности и скорости
pub const ARGON2_MEMORY_KB: u32 = 65536;   // 64 MB
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;
