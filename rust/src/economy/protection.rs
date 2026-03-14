// ============================================================
// ROOT v2.0 — economy/protection.rs
// Защита от серых схем:
//   VelocityTracker    — лимит продаж в день/неделю/месяц
//   AnomalyDetector    — заморозка при подозрительной активности
//   PersonhoodRegistry — 1 устройство = 1 Genesis бонус
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::constants::{
    ANOMALY_FREEZE_SECONDS, ANOMALY_SELL_PCT_THRESHOLD, DROPS_PER_SAP,
    MAX_GENESIS_PER_DEVICE, MAX_GENESIS_PER_IP, VELOCITY_LIMIT_DROPS_PER_DAY,
    VELOCITY_LIMIT_DROPS_PER_MONTH, VELOCITY_LIMIT_DROPS_PER_WEEK,
};
use super::types::EconomyError;

// ── Velocity Tracker ─────────────────────────────────────────

/// Счётчик скорости продаж — защита от dump атак
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VelocityTracker {
    pub sold_today_drops:  u64,
    pub sold_week_drops:   u64,
    pub sold_month_drops:  u64,
    pub day_reset_ts:      u64,
    pub week_reset_ts:     u64,
    pub month_reset_ts:    u64,
}

impl VelocityTracker {
    pub fn new() -> Self {
        let now = now_secs();
        VelocityTracker {
            sold_today_drops:  0,
            sold_week_drops:   0,
            sold_month_drops:  0,
            day_reset_ts:   now,
            week_reset_ts:  now,
            month_reset_ts: now,
        }
    }

    /// Проверить лимиты и зарегистрировать продажу
    pub fn check_and_record(&mut self, amount: u64) -> Result<(), EconomyError> {
        let now = now_secs();

        // Сброс счётчиков по истечении периода
        if now - self.day_reset_ts   >= 86400     { self.sold_today_drops = 0; self.day_reset_ts = now; }
        if now - self.week_reset_ts  >= 604800    { self.sold_week_drops = 0;  self.week_reset_ts = now; }
        if now - self.month_reset_ts >= 2_592_000 { self.sold_month_drops = 0; self.month_reset_ts = now; }

        // Проверка дневного лимита
        if self.sold_today_drops + amount > VELOCITY_LIMIT_DROPS_PER_DAY {
            return Err(EconomyError::VelocityLimitExceeded {
                limit: VELOCITY_LIMIT_DROPS_PER_DAY / DROPS_PER_SAP,
            });
        }
        // Проверка недельного лимита
        if self.sold_week_drops + amount > VELOCITY_LIMIT_DROPS_PER_WEEK {
            return Err(EconomyError::VelocityLimitExceeded {
                limit: VELOCITY_LIMIT_DROPS_PER_WEEK / DROPS_PER_SAP,
            });
        }
        // Проверка месячного лимита
        if self.sold_month_drops + amount > VELOCITY_LIMIT_DROPS_PER_MONTH {
            return Err(EconomyError::VelocityLimitExceeded {
                limit: VELOCITY_LIMIT_DROPS_PER_MONTH / DROPS_PER_SAP,
            });
        }

        // Регистрируем
        self.sold_today_drops  += amount;
        self.sold_week_drops   += amount;
        self.sold_month_drops  += amount;
        Ok(())
    }
}

// ── Anomaly Detector ─────────────────────────────────────────

/// Детектор аномальной активности — заморозка при подозрении
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetector {
    /// Заморожен до этого timestamp (0 = не заморожен)
    pub frozen_until: u64,
    /// История продаж за последние 24 часа (timestamp, amount)
    pub recent_sales: Vec<(u64, u64)>,
    /// Флаг: получил Genesis и сразу продал
    pub genesis_then_sold: bool,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        AnomalyDetector {
            frozen_until:      0,
            recent_sales:      Vec::new(),
            genesis_then_sold: false,
        }
    }

    /// Проверить не заморожен ли аккаунт
    pub fn check_frozen(&self) -> Result<(), EconomyError> {
        if now_secs() < self.frozen_until {
            return Err(EconomyError::AccountFrozen {
                until_timestamp: self.frozen_until,
            });
        }
        Ok(())
    }

    /// Записать продажу и проверить на аномалию
    /// Возвращает true если аномалия обнаружена
    pub fn record_sale(
        &mut self,
        amount: u64,
        total_balance: u64,
        genesis_age_secs: Option<u64>,
    ) -> bool {
        let now = now_secs();

        // Очищаем старые записи (старше 24 часов)
        self.recent_sales.retain(|(ts, _)| now - ts < 86400);
        self.recent_sales.push((now, amount));

        let sold_24h: u64 = self.recent_sales.iter().map(|(_, a)| a).sum();

        // Аномалия 1: продажа > 50% баланса за 24 часа
        let anomaly_1 = if total_balance > 0 {
            sold_24h as f64 / total_balance as f64 > ANOMALY_SELL_PCT_THRESHOLD
        } else {
            false
        };

        // Аномалия 2: Genesis получен < 7 дней назад и уже продаёт > 10 SAP
        let anomaly_2 = if let Some(age) = genesis_age_secs {
            age < 7 * 86400 && sold_24h > 10 * DROPS_PER_SAP
        } else {
            false
        };

        if anomaly_2 {
            self.genesis_then_sold = true;
        }

        let is_anomaly = anomaly_1 || anomaly_2;
        if is_anomaly {
            self.frozen_until = now + ANOMALY_FREEZE_SECONDS;
            println!(
                "  🚨 АНОМАЛИЯ: аккаунт заморожен на 72 часа (продажа {:.1}% баланса за 24ч)",
                sold_24h as f64 / total_balance.max(1) as f64 * 100.0
            );
        }

        is_anomaly
    }
}

// ── Personhood Registry ──────────────────────────────────────

/// Реестр Proof of Personhood — 1 устройство = 1 Genesis бонус
#[derive(Debug)]
pub struct PersonhoodRegistry {
    /// IP → количество Genesis бонусов
    pub ip_claims: HashMap<String, u32>,
    /// Device fingerprint → количество Genesis бонусов
    pub device_claims: HashMap<String, u32>,
}

impl PersonhoodRegistry {
    pub fn new() -> Self {
        PersonhoodRegistry {
            ip_claims:     HashMap::new(),
            device_claims: HashMap::new(),
        }
    }

    /// Проверить и зарегистрировать Genesis бонус для устройства
    pub fn check_and_register(
        &mut self,
        ip: &str,
        device_id: &str,
    ) -> Result<(), EconomyError> {
        let ip_count     = self.ip_claims.get(ip).copied().unwrap_or(0);
        let device_count = self.device_claims.get(device_id).copied().unwrap_or(0);

        if ip_count >= MAX_GENESIS_PER_IP {
            return Err(EconomyError::PersonhoodViolation);
        }
        if device_count >= MAX_GENESIS_PER_DEVICE {
            return Err(EconomyError::PersonhoodViolation);
        }

        *self.ip_claims.entry(ip.to_string()).or_insert(0)             += 1;
        *self.device_claims.entry(device_id.to_string()).or_insert(0) += 1;
        Ok(())
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
