/* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
// ============================================================
// ROOT v2.0 — root-ffi/src/lib.rs
// Flutter FFI точка входа
//
// Архитектура — Cargo workspace:
//
//  root-identity   — Ed25519 ключи, BIP-39 мнемоника, Shamir, zeroize
//  root-crypto     — ChaCha20-Poly1305, Argon2id, X25519 ECDH
//  root-storage    — SQLite, шифрование БД, модели, Merkle, PanicButton
//  root-network    — libp2p, Gossipsub, mDNS
//  root-economy    — токен SAP, ledger, vesting, консенсус
//  root-core       — AppState (единое состояние приложения)
//  root-ffi        — этот крейт, Flutter bridge
//
//  api/            — FFI интерфейс для Flutter
//    types.rs      — DTO структуры (IdentityInfo, MessageInfo...)
//    state.rs      — APP_STATE: Arc<Mutex<AppState>>
//    identity.rs   — generate_identity, restore_identity
//    database.rs   — unlock_database, panic_button
//    messaging.rs  — send_message, get_messages, mark_read
//    contacts.rs   — add_contact, get_contacts
//    economy.rs    — get_balance, transfer, stake, vesting
//    p2p.rs        — start_p2p_node, send_p2p_message
//    utils.rs      — get_version, validate_public_key
//    mod.rs        — RootApi (единая точка входа для Flutter)
//
//  cli/            — терминальный интерфейс (отдельный bin)
// ============================================================

mod frb_generated;

pub mod api;
pub mod runtime;  // ← Добавить эту строку
pub use root_economy as economy;
pub use root_identity as identity;
pub use root_network as network;
pub use root_storage as storage;
pub use root_crypto as crypto;
pub use root_core::AppState;

// transport — алиас для обратной совместимости с flutter_rust_bridge
pub use network::channels as transport;

// ── Реэкспорт типов нужных Flutter ───────────────────────────

pub use identity::{Identity, ProtectedKey, SecretSeed, ShamirVault};
pub use storage::{Contact, Database, MerkleTree, Message, PanicButton, StorageError};
pub use economy::{DROPS_PER_SAP, EconomyError, Ledger, Transaction, TxType, VestingSchedule};
pub use api::{ApiError, BalanceInfo, IdentityInfo, MessageInfo, NodeStatus, RootApi, VestingInfo};

// ── Константы ────────────────────────────────────────────────

pub const VERSION: &str = "2.0.0-alpha";
pub const BUILD_DATE: &str = "2026-03";
pub const NETWORK_ID: &str = "root-mainnet-v2";

// ── Тесты ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "2.0.0-alpha");
        assert_eq!(NETWORK_ID, "root-mainnet-v2");
    }

    #[test]
    fn test_drops_per_sap() {
        assert_eq!(DROPS_PER_SAP, 100_000_000);
    }

    #[test]
    fn test_identity_generate() {
        let (identity, mnemonic) = Identity::generate().unwrap();
        let key = hex::encode(identity.verifying_key.as_bytes());
        assert_eq!(key.len(), 64);
        assert!(mnemonic.to_string().split_whitespace().count() == 24);
    }
}
