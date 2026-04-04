// root-economy/src/error.rs
use root_crypto::CryptoError;
use thiserror::Error;

/// Ошибки экономического модуля (счета, транзакции, консенсус, казначейство, вестинг).
///
/// Все ошибки реализуют `std::error::Error` и могут быть
/// автоматически конвертированы в `FfiError` через `#[from]`.
#[derive(Error, Debug)]
pub enum EconomyError {
    // ─── Счета и баланс ──────────────────────────────────
    /// Указанный счёт не существует в реестре
    #[error("Счёт '{account_id}' не найден")]
    AccountNotFound { account_id: String },

    /// На счету недостаточно средств для операции
    #[error("Недостаточно средств: требуется {required}, доступно {available}")]
    InsufficientFunds { required: u64, available: u64 },

    /// Счёт заблокирован администратором или системой защиты
    // стало:
    // #[error("Счёт заморожен до {until_timestamp}")]
    // AccountFrozen { until_timestamp: u64 },

    /// Попытка провести транзакцию с нулевой или отрицательной суммой
    #[error("Неверная сумма: должна быть строго больше нуля")]
    InvalidAmount,

    // ─── Транзакции и консенсус ──────────────────────────
    /// Транзакция нарушает правила консенсуса сети
    #[error("Нарушение консенсуса: {reason}")]
    ConsensusViolation { reason: String },

    /// Обнаружена попытка двойного списания (double-spend)
    #[error("Обнаружено двойное списание (double-spend)")]
    DoubleSpend,

    /// Транзакция устарела (nonce превышает ожидаемый)
    #[error("Транзакция просрочена: nonce {current} > expected {expected}")]
    ExpiredTransaction { current: u64, expected: u64 },

    /// Подпись транзакции невалидна или не соответствует отправителю
    #[error("Невалидная подпись транзакции")]
    InvalidSignature,

    // ─── Казначейство и вестинг ──────────────────────────
    /// Казначейство заблокировано (режим обслуживания или аварийный стоп)
    #[error("Казначейство заблокировано")]
    TreasuryLocked,

    /// Попытка вывода средств до даты разблокировки по графику вестинга
    #[error("Нарушение графика вестинга: разблокировка только после {unlock_date}")]
    VestingViolation { unlock_date: u64 },

    #[error("Vesting: токены ещё не разблокированы. Доступно: {available} Drops")]
    VestingLocked { available: u64 },

    /// Превышен лимит выплат за период
    #[error("Лимит выплат исчерпан")]
    PayoutLimitExceeded,

    // ─── Крипто-зависимости (делегирование) ─────────────
    /// Ошибка из крипто-подсистемы (валидация подписи, хеши)
    #[error("Крипто: {0}")]
    Crypto(#[from] CryptoError),

    // ─── Общие ───────────────────────────────────────────
    /// Динамическая ошибка с описанием (для редких/нестандартных сценариев)
    #[error("Экономическая ошибка: {0}")]
    Other(String),

    //********************************** */
    #[error("Превышен лимит транзакций: максимум {max}/сек")]
    RateLimitExceeded { max: u32 },

    #[error("Превышен Hard Cap: эмиссия {current}, лимит {cap}")]
    HardCapExceeded { current: u64, cap: u64 },

    #[error("Невалидная транзакция: {0}")]
    InvalidTransaction(String),

    #[error("Узел заблокирован за систематические нарушения")]
    NodeBanned,

    #[error("Genesis период завершён: все {0} мест заняты")]
    GenesisEnded(u32),

    #[error("Узел не найден: {0}")]
    NodeNotFound(String),

    #[error("Недостаточный залог: нужно {required} Drops")]
    InsufficientStake { required: u64 },

    #[error("Недостаточная репутация: требуется {required}, есть {available}")]
    InsufficientReputation { required: u8, available: u8 },

    #[error("Velocity Limit: превышен дневной лимит продаж {limit} SAP")]
    VelocityLimitExceeded { limit: u64 },

    #[error("Proof of Personhood: превышен лимит Genesis бонусов для этого устройства")]
    PersonhoodViolation,

    #[error("Счёт заморожен до {until_timestamp}")]
    AccountFrozen { until_timestamp: u64 },
    //*************************************************** */
}

// 🔧 Методы-помощники для агрегатора / логирования
impl EconomyError {
    /// Код ошибки для метрик / API-ответов
    pub fn code(&self) -> &'static str {
        match self {
            EconomyError::AccountNotFound { .. } => "economy.account_not_found",
            EconomyError::InsufficientFunds { .. } => "economy.insufficient_funds", 
            EconomyError::ConsensusViolation { .. } => "economy.consensus",
            EconomyError::DoubleSpend => "economy.double_spend",
            EconomyError::ExpiredTransaction { .. } => "economy.expired",
            EconomyError::InvalidSignature => "economy.invalid_signature",
            EconomyError::TreasuryLocked => "economy.treasury_locked",
            EconomyError::VestingViolation { .. } => "economy.vesting",
            EconomyError::PayoutLimitExceeded => "economy.payout_limit",
            EconomyError::Crypto(e) => e.code(), // делегируем в CryptoError
            EconomyError::Other(_) => "economy.other",
            //***************************** */
            EconomyError::RateLimitExceeded { .. } => "economy.rate_limit",
            EconomyError::HardCapExceeded { .. } => "economy.hard_cap",
            EconomyError::InvalidTransaction(_) => "economy.invalid_tx",
            EconomyError::NodeBanned => "economy.node_banned",
            EconomyError::GenesisEnded(_) => "economy.genesis_ended",
            EconomyError::NodeNotFound(_) => "economy.node_not_found",
            EconomyError::InsufficientStake { .. } => "economy.insufficient_stake",
            EconomyError::InsufficientReputation { .. } => "economy.low_reputation",
            EconomyError::VelocityLimitExceeded { .. } => "economy.velocity_limit",
            EconomyError::PersonhoodViolation => "economy.personhood",
            EconomyError::VestingLocked { .. } => "economy.vesting_locked",
            EconomyError::AccountFrozen { .. } => "economy.account_frozen",
            EconomyError::InvalidAmount => "economy.invalid_amount",
        }
    }

    /// Можно ли повторить операцию при этой ошибке?
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            EconomyError::AccountNotFound { .. } |  // счёт может появиться позже
            EconomyError::InsufficientFunds { .. } | // можно пополнить и повторить
            EconomyError::ExpiredTransaction { .. } | // можно переподписать с новым nonce
            EconomyError::PayoutLimitExceeded |      // лимит сбросится в новом периоде
            EconomyError::Other(_)
        )
    }
}
