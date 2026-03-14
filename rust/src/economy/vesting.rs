// ============================================================
// ROOT v2.0 — economy/vesting.rs
// Расписание разблокировки Genesis бонуса
// ============================================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::{DROPS_PER_SAP, VESTING_IMMEDIATE_PCT, VESTING_SCHEDULE};
use super::types::EconomyError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    /// Timestamp когда был выдан Genesis бонус
    pub grant_timestamp: u64,
    /// Общая сумма бонуса в Drops
    pub total_drops: u64,
    /// Уже разблокировано
    pub unlocked_drops: u64,
}

impl VestingSchedule {
    /// Создать новое расписание — 10% доступно сразу
    pub fn new(total_drops: u64) -> Self {
        let immediate = (total_drops as f64 * VESTING_IMMEDIATE_PCT) as u64;
        VestingSchedule {
            grant_timestamp: now_secs(),
            total_drops,
            unlocked_drops: immediate,
        }
    }

    /// Рассчитать сколько Drops доступно сейчас
    pub fn available_drops(&self) -> u64 {
        let days_passed = (now_secs() - self.grant_timestamp) / 86400;

        let mut unlocked_pct = VESTING_IMMEDIATE_PCT;
        for (days, pct) in &VESTING_SCHEDULE {
            if days_passed >= *days {
                unlocked_pct = *pct;
            }
        }

        let total_unlocked = (self.total_drops as f64 * unlocked_pct) as u64;
        total_unlocked.min(self.unlocked_drops)
    }

    /// Потратить из vesting
    pub fn spend(&mut self, amount: u64) -> Result<(), EconomyError> {
        let available = self.available_drops();
        if available < amount {
            return Err(EconomyError::VestingLocked { available });
        }
        self.unlocked_drops = self.unlocked_drops.saturating_sub(amount);
        Ok(())
    }

    /// Проверить полностью ли разблокирован vesting (365 дней)
    pub fn is_fully_unlocked(&self) -> bool {
        let days_passed = (now_secs() - self.grant_timestamp) / 86400;
        days_passed >= 365
    }

    /// Процент разблокировки (0.0 — 100.0)
    pub fn percent_unlocked(&self) -> f64 {
        self.available_drops() as f64 / self.total_drops as f64 * 100.0
    }

    /// Дней до полной разблокировки
    pub fn days_until_full(&self) -> u64 {
        let days_passed = (now_secs() - self.grant_timestamp) / 86400;
        if days_passed >= 365 {
            0
        } else {
            365 - days_passed
        }
    }

    /// Заблокировано SAP (для UI)
    pub fn locked_sap(&self) -> f64 {
        let locked = self.total_drops.saturating_sub(self.available_drops());
        locked as f64 / DROPS_PER_SAP as f64
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
