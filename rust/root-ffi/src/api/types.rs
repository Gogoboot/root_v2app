//! DTO структуры и типы ошибок для FFI границы.
//!
//! Этот модуль — переводчик между Rust-миром и Flutter/Tauri.
//!
//! # Два типа ошибок в проекте
//!
//! - [`ApiError`] — ошибки бизнес-логики API слоя
//! - [`crate::error::FfiError`] — технические ошибки инфраструктуры
//!
//! TODO: объединить в один тип в отдельном спринте рефакторинга FFI.

use thiserror::Error;
use zeroize::Zeroizing;

// ─── ApiError ────────────────────────────────────────────────────────────────

/// Ошибки бизнес-логики API слоя.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Identity не инициализирована. Вызови generate_identity() или restore_identity()")]
    IdentityNotInitialized,

    #[error("База данных не открыта. Вызови unlock_database(password)")]
    DatabaseNotOpen,

    #[error("Ledger не инициализирован")]
    LedgerNotInitialized,

    #[error("Panic Button активирован — перезапусти приложение")]
    PanicActivated,

    #[error("Неверное состояние приложения: {0}")]
    InvalidState(String),

    #[error("Ошибка Identity: {0}")]
    IdentityError(String),

    #[error("Ошибка экономики: {0}")]
    EconomyError(String),

    #[error("Ошибка хранилища: {0}")]
    StorageError(String),

    #[error("Неверные параметры: {0}")]
    InvalidInput(String),

    #[error("Внутренняя ошибка: {0}")]
    InternalError(String),
}

// ─── Маппинг ошибок инфраструктуры → ApiError ────────────────────────────────

use root_storage::StorageError;
use root_identity::IdentityError;

impl From<StorageError> for ApiError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::NotOpen              => ApiError::DatabaseNotOpen,
            StorageError::PanicButtonActivated => ApiError::PanicActivated,
            other                              => ApiError::StorageError(other.to_string()),
        }
    }
}

impl From<IdentityError> for ApiError {
    fn from(e: IdentityError) -> Self {
        ApiError::IdentityError(e.to_string())
    }
}

// ─── DTO структуры ───────────────────────────────────────────────────────────

/// Информация об идентичности для передачи во Flutter/Tauri.
///
/// Мнемоника здесь обычная String — Zeroizing убирается намеренно
/// потому что Tauri не умеет сериализовать Zeroizing<String>.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdentityInfo {
    pub public_key: String,
    pub mnemonic: Option<String>,
    pub network: String,
}

/// Результат разблокировки базы данных.
///
/// Возвращается из `unlock_database()` вместо простого `bool`.
/// JS читает `status` и решает что показать пользователю.
///
/// # Статусы
///
/// - `"ok"` — всё хорошо, входим в приложение
/// - `"mnemonic_pending"` — мнемоника не была подтверждена,
///   показываем экран мнемоники снова
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[flutter_rust_bridge::frb]
pub struct UnlockResult {
    /// Статус: "ok" | "mnemonic_pending"
    pub status: String,

    /// Публичный ключ — всегда присутствует при успешном открытии
    pub public_key: Option<String>,

    /// Мнемоника — только когда status = "mnemonic_pending".
    /// Позволяет показать её снова без дополнительных запросов.
    pub mnemonic: Option<String>,
}

/// Баланс пользователя для экрана кошелька.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BalanceInfo {
    pub public_key: String,
    pub balance_sap: f64,
    pub balance_drops: u64,
    pub staked_sap: f64,
    pub reputation: u8,
    pub is_banned: bool,
    pub vesting_available_sap: f64,
    pub vesting_locked_sap: f64,
}

/// Сообщение для отображения в UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageInfo {
    pub id: u64,
    pub from_key: String,
    pub to_key: String,
    pub content: String,
    pub timestamp: u64,
    pub is_read: bool,
    pub from_name: Option<String>,
}

/// Статус узла для экрана настроек.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeStatus {
    pub public_key: String,
    pub is_active: bool,
    pub reputation: u8,
    pub staked_sap: f64,
    pub offense_count: u8,
    pub genesis_claimed: bool,
    pub tx_count: usize,
    pub peer_count: u32,
    pub network: String,
    pub version: String,
}

/// Информация о vesting для экрана кошелька.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VestingInfo {
    pub total_sap: f64,
    pub available_sap: f64,
    pub locked_sap: f64,
    pub percent_unlocked: f64,
    pub fully_unlocked: bool,
    pub days_until_full: u64,
}

/// Результат транзакции.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TxResult {
    pub tx_id: String,
    pub amount_sap: f64,
    pub fee_sap: f64,
    pub burned_sap: f64,
    pub timestamp: u64,
    pub success: bool,
}

/// Предупреждение о небезопасных P2P методах.
#[derive(Debug, Clone, serde::Serialize)]
pub struct P2pWarning {
    pub show_warning: bool,
    pub safe_methods: Vec<String>,
    pub unsafe_methods: Vec<String>,
    pub message: String,
}

/// Активный пир — передаётся в UI для отображения списка соединений.
/// Группировка и сортировка выполняются на стороне JS.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PeerInfoDto {
    /// Полный PeerID строкой
    pub peer_id: String,
    /// Протокол: "TCP" | "WS" | "QUIC" | "mDNS"
    pub protocol: String,
    /// UNIX timestamp момента подключения — для сортировки в UI
    pub connected_at: u64,
}
