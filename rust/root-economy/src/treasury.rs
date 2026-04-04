// ============================================================
// ROOT v2.0 — economy/treasury.rs
// Казначейство: комиссии, slash, стабфонд
// ============================================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::{
    DROPS_PER_SAP, PRICE_DROP_TRIGGER_PCT, STABFUND_RESERVE_PCT, TREASURY_LOW_THRESHOLD_PCT,
    TREASURY_MIN_RESERVE_PCT,
};
use super::types::EconomyError;


#[derive(Debug, Serialize, Deserialize)]
pub struct Treasury {
    pub balance_drops: u64,
    pub total_fees_drops: u64,
    pub total_slash_drops: u64,
    /// Сожжено через P2P burn навсегда
    pub total_burned_drops: u64,
    pub total_paid_drops: u64,
    /// Стабфонд — 20% от баланса зарезервировано
    pub stabfund_drops: u64,
    /// Последний зафиксированный курс SAP
    pub last_price: f64,
    /// Timestamp последнего обновления курса
    pub last_price_ts: u64,
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new()
    }
}

impl Treasury {
    pub fn new() -> Self {
        Treasury {
            balance_drops: 0,
            total_fees_drops: 0,
            total_slash_drops: 0,
            total_burned_drops: 0,
            total_paid_drops: 0,
            stabfund_drops: 0,
            last_price: 1.0,
            last_price_ts: now_secs(),
        }
    }

    pub fn balance_sap(&self) -> f64 {
        self.balance_drops as f64 / DROPS_PER_SAP as f64
    }

    /// Пополнить Treasury (от комиссий или slash)
    pub fn deposit(&mut self, amount: u64, from_slash: bool) {
        self.balance_drops += amount;
        // 20% каждого поступления → стабфонд
        let stab_share = (amount as f64 * STABFUND_RESERVE_PCT) as u64;
        self.stabfund_drops += stab_share;

        if from_slash {
            self.total_slash_drops += amount;
        } else {
            self.total_fees_drops += amount;
        }
    }

    /// Вывести из Treasury (с проверкой минимального резерва)
    pub fn withdraw(&mut self, amount: u64, total_supply: u64) -> Result<(), EconomyError> {
        let min_reserve = (total_supply as f64 * TREASURY_MIN_RESERVE_PCT) as u64;
        if self.balance_drops < min_reserve + amount {
            return Err(EconomyError::TreasuryLocked);
        }
        self.balance_drops -= amount;
        self.total_paid_drops += amount;
        Ok(())
    }

    /// Множитель вознаграждения (1.0 / 0.5 / 0.0) в зависимости от резерва
    pub fn reward_multiplier(&self, total_supply: u64) -> f64 {
        let low = (total_supply as f64 * TREASURY_LOW_THRESHOLD_PCT) as u64;
        let min = (total_supply as f64 * TREASURY_MIN_RESERVE_PCT) as u64;
        if self.balance_drops < min {
            0.0
        } else if self.balance_drops < low {
            0.5
        } else {
            1.0
        }
    }

    /// Механизм 4: Стабфонд — автовыкуп SAP при падении курса на 30%+
    pub fn check_stabfund_intervention(&mut self, current_price: f64) -> Option<u64> {
        let now = now_secs();
        // Обновляем не чаще раза в час
        if now - self.last_price_ts < 3600 {
            return None;
        }

        let price_drop = (self.last_price - current_price) / self.last_price;

        if price_drop >= PRICE_DROP_TRIGGER_PCT && self.stabfund_drops > 0 {
            // Выкупаем 10% стабфонда
            let buyback = (self.stabfund_drops as f64 * 0.10) as u64;
            self.stabfund_drops -= buyback;
            self.balance_drops -= buyback.min(self.balance_drops);
            self.last_price = current_price;
            self.last_price_ts = now;

            println!(
                "  📈 Стабфонд: выкуп {:.4} SAP (курс упал на {:.1}%)",
                buyback as f64 / DROPS_PER_SAP as f64,
                price_drop * 100.0
            );
            return Some(buyback);
        }

        self.last_price = current_price;
        self.last_price_ts = now;
        None
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
