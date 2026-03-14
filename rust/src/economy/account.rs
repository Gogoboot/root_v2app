// ============================================================
// ROOT v2.0 — economy/account.rs
// Счёт пользователя
// ============================================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::{DROPS_PER_SAP, MAX_TXS_PER_SECOND, MIN_STAKE_DROPS};
use super::protection::{AnomalyDetector, VelocityTracker};
use super::types::EconomyError;
use super::vesting::VestingSchedule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub public_key: String,
    pub balance_drops: u64,
    pub staked_drops: u64,
    pub reputation: u8,
    pub offense_count: u8,
    pub is_banned: bool,
    pub tx_history: Vec<String>,
    pub last_tx_timestamp: u64,
    pub tx_count_this_second: u32,
    pub genesis_claimed: bool,
    /// Vesting расписание (если получал Genesis)
    pub vesting: Option<VestingSchedule>,
    /// Трекер скорости продаж
    pub velocity: VelocityTracker,
    /// Детектор аномалий
    pub anomaly: AnomalyDetector,
    /// Timestamp получения Genesis
    pub genesis_timestamp: Option<u64>,
}

impl Account {
    pub fn new(public_key: String) -> Self {
        Account {
            public_key,
            balance_drops: 0,
            staked_drops: 0,
            reputation: 50,
            offense_count: 0,
            is_banned: false,
            tx_history: Vec::new(),
            last_tx_timestamp: 0,
            tx_count_this_second: 0,
            genesis_claimed: false,
            vesting: None,
            velocity: VelocityTracker::new(),
            anomaly: AnomalyDetector::new(),
            genesis_timestamp: None,
        }
    }

    /// Баланс в SAP (для отображения)
    pub fn balance_sap(&self) -> f64 {
        self.balance_drops as f64 / DROPS_PER_SAP as f64
    }

    /// Активный relay-узел: stake >= 10 SAP и не забанен
    pub fn is_active_node(&self) -> bool {
        self.staked_drops >= MIN_STAKE_DROPS && !self.is_banned
    }

    /// Проверить rate limit (макс 10 tx/сек)
    pub fn check_rate_limit(&mut self) -> Result<(), EconomyError> {
        let now = now_secs();
        if now > self.last_tx_timestamp {
            self.tx_count_this_second = 0;
            self.last_tx_timestamp = now;
        }
        self.tx_count_this_second += 1;
        if self.tx_count_this_second > MAX_TXS_PER_SECOND {
            return Err(EconomyError::RateLimitExceeded {
                max: MAX_TXS_PER_SECOND,
            });
        }
        Ok(())
    }

    /// Добавить ID транзакции в историю (хранит последние 100)
    pub fn add_tx(&mut self, tx_id: String) {
        if self.tx_history.len() >= 100 {
            self.tx_history.remove(0);
        }
        self.tx_history.push(tx_id);
    }

    /// Возраст Genesis бонуса в секундах
    pub fn genesis_age_secs(&self) -> Option<u64> {
        self.genesis_timestamp.map(|ts| now_secs() - ts)
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
