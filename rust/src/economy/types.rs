// ============================================================
// ROOT v2.0 — economy/types.rs
// Базовые типы: ошибки, транзакции
// ============================================================

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use super::constants::DROPS_PER_SAP;

// ── Ошибки экономики ─────────────────────────────────────────

#[derive(Error, Debug)]
pub enum EconomyError {
    #[error("Недостаточно средств: нужно {need} Drops, есть {have} Drops")]
    InsufficientFunds { need: u64, have: u64 },

    #[error("Превышен Hard Cap: эмиссия {current}, лимит {cap}")]
    HardCapExceeded { current: u64, cap: u64 },

    #[error("Недостаточный залог: нужно {need} Drops")]
    InsufficientStake { need: u64 },

    #[error("Узел не найден: {0}")]
    NodeNotFound(String),

    #[error("Превышен лимит транзакций: максимум {max}/сек")]
    RateLimitExceeded { max: u32 },

    #[error("Неверная транзакция: {0}")]
    InvalidTransaction(String),

    #[error("Treasury заблокирован: баланс ниже минимального резерва")]
    TreasuryReserveLocked,

    #[error("Узел забанен за систематические нарушения")]
    NodeBanned,

    #[error("Genesis период завершён: все {0} мест заняты")]
    GenesisEnded(u32),

    #[error("Velocity Limit: превышен дневной лимит продаж {limit} SAP")]
    VelocityLimitExceeded { limit: u64 },

    #[error("Vesting: токены ещё не разблокированы. Доступно: {available} Drops")]
    VestingLocked { available: u64 },

    #[error("P2P торговля требует репутацию >= {required}, у вас {have}")]
    InsufficientReputation { required: u8, have: u8 },

    #[error("Аккаунт заморожен до {until_timestamp} (аномальная активность)")]
    AccountFrozen { until_timestamp: u64 },

    #[error("Proof of Personhood: превышен лимит Genesis бонусов для этого устройства")]
    PersonhoodViolation,
}

// ── Транзакция ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Уникальный ID транзакции (SHA256 hash)
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount_drops: u64,
    pub fee_drops: u64,
    /// Сожжено при P2P обмене
    pub burned_drops: u64,
    pub tx_type: TxType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxType {
    /// Обычный перевод
    Transfer,
    /// P2P обмен (серый рынок — с предупреждением)
    P2PExchange,
    /// Вознаграждение за relay
    RelayReward {
        relayed_bytes: u64,
        witnesses: Vec<String>,
    },
    /// Вознаграждение свидетелю
    WitnessReward { relay_tx_id: String },
    /// Заморозка stake
    Stake,
    /// Разморозка stake
    Unstake,
    /// Genesis бонус первым 1000 узлам
    GenesisBonus,
    /// Штраф за нарушение
    SlashPenalty { offense_number: u8 },
    /// Автовыкуп стабфондом при падении курса
    StabFundBuyback { price_drop_pct: f64 },
}

impl Transaction {
    /// Создать новую транзакцию с автоматическим ID и timestamp
    pub fn new(
        from: String,
        to: String,
        amount_drops: u64,
        fee_drops: u64,
        burned_drops: u64,
        tx_type: TxType,
    ) -> Self {
        let timestamp = now_secs();
        let mut hasher = Sha256::new();
        hasher.update(from.as_bytes());
        hasher.update(to.as_bytes());
        hasher.update(amount_drops.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        let id = hex::encode(hasher.finalize());
        Transaction {
            id,
            from,
            to,
            amount_drops,
            fee_drops,
            burned_drops,
            tx_type,
            timestamp,
        }
    }

    /// Сумма в SAP (для отображения)
    pub fn amount_sap(&self) -> f64 {
        self.amount_drops as f64 / DROPS_PER_SAP as f64
    }
}

// ── Вспомогательная функция ───────────────────────────────────

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
