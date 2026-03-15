mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
// ============================================================
// ROOT v2.0 — lib.rs
// ============================================================
//
//  Дерево модулей:
//
//  identity/           — ключи, мнемоника, Shamir, защита в памяти
//    seed.rs           — SecretSeed (zeroize обёртка)
//    keys.rs           — Identity (Ed25519 генерация, подпись)
//    shamir.rs         — ShamirVault (3/5 разделение ключа)
//    protected.rs      — ProtectedKey (XOR маскировка в памяти)
//
//  storage/            — база данных, шифрование, верификация
//    constants.rs      — константы (KEY_LEN, Argon2 параметры)
//    error.rs          — StorageError
//    key.rs            — StorageKey (Argon2id деривация)
//    models.rs         — Message, Contact
//    merkle.rs         — MerkleTree (верификация целостности)
//    panic.rs          — PanicButton
//    database.rs       — Database (главный движок SQLite)
//
//  network/            — P2P сеть (libp2p)
//    behaviour.rs      — RootBehaviour (Gossipsub + mDNS)
//    channels.rs       — start_node_channels (Flutter режим)
//    node.rs           — start_node (интерактивный CLI режим)
//
//  economy/            — токен SAP, экономика
//    constants.rs      — все константы
//    types.rs          — EconomyError, Transaction, TxType
//    vesting.rs        — VestingSchedule
//    protection.rs     — VelocityTracker, AnomalyDetector, PersonhoodRegistry
//    account.rs        — Account
//    treasury.rs       — Treasury + стабфонд
//    consensus.rs      — WitnessConfig, Proof-of-Relay
//    ledger.rs         — Ledger (главный движок экономики)
//
//  api/                — FFI интерфейс для Flutter
//    types.rs          — DTO структуры (IdentityInfo, MessageInfo...)
//    state.rs          — глобальное состояние (lazy_static)
//    identity.rs       — generate_identity, restore_identity
//    database.rs       — unlock_database, panic_button
//    messaging.rs      — send_message, get_messages, mark_read
//    contacts.rs       — add_contact, get_contacts
//    economy.rs        — get_balance, transfer, stake, vesting
//    p2p.rs            — start_p2p_node, send_p2p_message
//    utils.rs          — get_version, validate_public_key
//    mod.rs            — RootApi (единая точка входа для Flutter)
//
//  cli/                — терминальный интерфейс
//    main.rs           — точка входа
//    commands/
//      identity.rs     — generate, restore, show
//      node.rs         — listen, connect, status
//      server.rs       — bootstrap сервер (VPS)
//      contacts.rs     — add, list
//      messages.rs     — list, send
//      db.rs           — unlock, verify
// ============================================================

pub mod api;
pub mod economy;
pub mod identity;
pub mod network;
pub mod storage;
pub mod crypto;


// transport — алиас для обратной совместимости с flutter_rust_bridge
pub use network::channels as transport;

// ── Реэкспорт ключевых типов ─────────────────────────────────

pub use identity::{Identity, ProtectedKey, SecretSeed, ShamirVault};

pub use storage::{Contact, Database, MerkleTree, Message, PanicButton, StorageError};

pub use economy::{
    Account, AnomalyDetector, DROPS_PER_SAP, EconomyError, Ledger, Transaction, Treasury, TxType,
    VelocityTracker, VestingSchedule,
};

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
        let (identity, mnemonic) = Identity::generate();
        let key = hex::encode(identity.verifying_key.as_bytes());
        assert_eq!(key.len(), 64);
        assert!(mnemonic.to_string().split_whitespace().count() == 24);
    }
}
