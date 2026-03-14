// ============================================================
// ROOT v2.0 — api/mod.rs
// Публичный FFI интерфейс для Flutter
//
// Подмодули:
//   types      — DTO структуры (IdentityInfo, MessageInfo...)
//   state      — глобальное состояние (lazy_static)
//   identity   — generate, restore, get_public_key
//   database   — unlock, panic_button
//   messaging  — send, get, mark_read
//   contacts   — add, get
//   economy    — balance, transfer, p2p, vesting, stake
//   p2p        — start_node, send_p2p, get_incoming
//   utils      — version, verify, validate
// ============================================================

pub mod types;
pub mod state;
pub mod identity;
pub mod database;
pub mod messaging;
pub mod contacts;
pub mod economy;
pub mod p2p;
pub mod utils;

// ── Реэкспорт ────────────────────────────────────────────────
pub use types::{
    ApiError, IdentityInfo, BalanceInfo, MessageInfo,
    NodeStatus, VestingInfo, TxResult, P2pWarning,
};

// ── Единая точка входа для flutter_rust_bridge ───────────────
// Все pub fn собраны здесь через re-export из подмодулей

#[flutter_rust_bridge::frb(opaque)]
pub struct RootApi {}

#[flutter_rust_bridge::frb(sync)]
impl RootApi {
    // ── Identity ─────────────────────────────────────────────
    pub fn generate_identity() -> Result<IdentityInfo, ApiError> {
        identity::generate_identity()
    }
    pub fn restore_identity(mnemonic: String) -> Result<IdentityInfo, ApiError> {
        identity::restore_identity(mnemonic)
    }
    pub fn get_public_key() -> Result<String, ApiError> {
        identity::get_public_key()
    }
    pub fn sign_message(message: Vec<u8>) -> Result<Vec<u8>, ApiError> {
        identity::sign_message(message)
    }

    // ── Database ─────────────────────────────────────────────
    pub fn unlock_database(password: String, db_path: String) -> Result<bool, ApiError> {
        database::unlock_database(password, db_path)
    }
    pub fn panic_button() -> Result<(), ApiError> {
        database::panic_button()
    }
    pub fn verify_db_integrity() -> Result<bool, ApiError> {
        database::verify_db_integrity()
    }
    pub fn is_panic_activated() -> bool {
        database::is_panic_activated()
    }

    // ── Messaging ────────────────────────────────────────────
    pub fn send_message(to_key: String, content: String) -> Result<u64, ApiError> {
        messaging::send_message(to_key, content)
    }
    pub fn get_messages() -> Result<Vec<MessageInfo>, ApiError> {
        messaging::get_messages()
    }
    pub fn get_unread_count() -> Result<u64, ApiError> {
        messaging::get_unread_count()
    }
    pub fn mark_message_read(msg_id: u64) -> Result<(), ApiError> {
        messaging::mark_message_read(msg_id)
    }

    // ── Contacts ─────────────────────────────────────────────
    pub fn add_contact(public_key: String, nickname: String) -> Result<(), ApiError> {
        contacts::add_contact(public_key, nickname)
    }
    pub fn get_contacts() -> Result<Vec<crate::storage::Contact>, ApiError> {
        contacts::get_contacts()
    }

    // ── Economy ──────────────────────────────────────────────
    pub fn get_balance() -> Result<BalanceInfo, ApiError> {
        economy::get_balance()
    }
    pub fn transfer(to_key: String, amount_sap: f64) -> Result<TxResult, ApiError> {
        economy::transfer(to_key, amount_sap)
    }
    pub fn p2p_exchange(to_key: String, amount_sap: f64) -> Result<TxResult, ApiError> {
        economy::p2p_exchange(to_key, amount_sap)
    }
    pub fn get_p2p_warning() -> P2pWarning {
        economy::get_p2p_warning()
    }
    pub fn get_vesting_info() -> Result<Option<VestingInfo>, ApiError> {
        economy::get_vesting_info()
    }
    pub fn stake_node() -> Result<bool, ApiError> {
        economy::stake_node()
    }
    pub fn unstake_node() -> Result<bool, ApiError> {
        economy::unstake_node()
    }
    pub fn get_node_status() -> Result<NodeStatus, ApiError> {
        economy::get_node_status()
    }
    pub fn claim_genesis(ip: String, device_id: String) -> Result<f64, ApiError> {
        economy::claim_genesis(ip, device_id)
    }

    // ── P2P ──────────────────────────────────────────────────
    pub fn start_p2p_node() -> Result<String, ApiError> {
        p2p::start_p2p_node()
    }
    pub fn send_p2p_message(content: String) -> Result<(), ApiError> {
        p2p::send_p2p_message(content)
    }
    pub fn is_p2p_running() -> bool {
        p2p::is_p2p_running()
    }
    pub fn get_incoming_messages() -> Vec<MessageInfo> {
        p2p::get_incoming_messages()
    }
    pub fn get_peer_count() -> u32 {
        p2p::get_peer_count()
    }

    // ── Utils ────────────────────────────────────────────────
    pub fn get_version() -> String {
        utils::get_version()
    }
    pub fn validate_public_key(key: String) -> bool {
        utils::validate_public_key(key)
    }
}
