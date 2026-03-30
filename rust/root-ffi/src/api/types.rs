// ============================================================
// ROOT v2.0 — api/types.rs
// DTO структуры для Flutter и типы ошибок
// ============================================================

use thiserror::Error;
use zeroize::Zeroizing;


#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Identity не инициализирована. Вызови generate_identity() или restore_identity()")]
    IdentityNotInitialized,

    #[error("База данных не открыта. Вызови unlock_database(password)")]
    DatabaseNotOpen,

    #[error("Ledger не инициализирован")]
    LedgerNotInitialized,

    #[error("Ошибка Identity: {0}")]
    IdentityError(String),

    #[error("Ошибка экономики: {0}")]
    EconomyError(String),

    #[error("Ошибка хранилища: {0}")]
    StorageError(String),

    #[error("Неверные параметры: {0}")]
    InvalidInput(String),

    #[error("Panic Button активирован — перезапусти приложение")]
    PanicActivated,
}

/// Информация об идентичности — передаётся в Flutter
/// Информация об идентичности — передаётся в Flutter
#[derive(Debug, Clone)]
pub struct IdentityInfo {
    /// Публичный ключ в hex (64 символа)
    pub public_key: String,
    /// 24 слова мнемоники — защищена Zeroizing (обнуляется при drop)
    pub mnemonic: Option<Zeroizing<String>>,
    /// Сеть: "root-mainnet-v2"
    pub network: String,
}

/// Баланс пользователя
#[derive(Debug, Clone)]
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

/// Сообщение для Flutter UI
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: u64,
    pub from_key: String,
    pub to_key: String,
    pub content: String,
    pub timestamp: u64,
    pub is_read: bool,
    /// Имя контакта если есть в адресной книге
    pub from_name: Option<String>,
}

/// Статус узла для экрана настроек
#[derive(Debug, Clone)]
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

/// Информация о vesting для экрана кошелька
#[derive(Debug, Clone)]
pub struct VestingInfo {
    pub total_sap: f64,
    pub available_sap: f64,
    pub locked_sap: f64,
    pub percent_unlocked: f64,
    pub fully_unlocked: bool,
    pub days_until_full: u64,
}

/// Результат транзакции
#[derive(Debug, Clone)]
pub struct TxResult {
    pub tx_id: String,
    pub amount_sap: f64,
    pub fee_sap: f64,
    pub burned_sap: f64,
    pub timestamp: u64,
    pub success: bool,
}

/// Предупреждение для P2P обмена
#[derive(Debug, Clone)]
pub struct P2pWarning {
    pub show_warning: bool,
    pub safe_methods: Vec<String>,
    pub unsafe_methods: Vec<String>,
    pub message: String,
}
