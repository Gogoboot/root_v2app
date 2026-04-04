// root-domain/src/entities/account.rs
// Доменная сущность аккаунта пользователя.
// ============================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Публичный ключ Ed25519 (hex)
    pub public_key: String,
    /// Зашифрованная мнемоника (хранится в инфраструктуре)
    pub encrypted_mnemonic: Option<Vec<u8>>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
