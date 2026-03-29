// ============================================================
// ROOT v2.0 — economy/mod.rs
// Модуль экономики SAP
//
// Подмодули:
//   constants  — все константы (Hard Cap, Velocity, Vesting...)
//   types      — Transaction, TxType, EconomyError
//   vesting    — VestingSchedule
//   protection — VelocityTracker, AnomalyDetector, PersonhoodRegistry
//   account    — Account
//   treasury   — Treasury
//   ledger     — Ledger (основной движок)
//   consensus  — WitnessConfig, Proof-of-Relay
// ============================================================

pub mod account;
pub mod consensus;
pub mod constants;
pub mod ledger;
pub mod protection;
pub mod treasury;
pub mod types;
pub mod vesting;

// ── Реэкспорт для удобства ───────────────────────────────────
// Снаружи можно писать: use crate::economy::Ledger;
// вместо: use crate::economy::ledger::Ledger;

pub use account::Account;
pub use consensus::{WitnessConfig, witness_config_for_reward};
pub use constants::*;
pub use ledger::Ledger;
pub use protection::{AnomalyDetector, PersonhoodRegistry, VelocityTracker};
pub use treasury::Treasury;
pub use types::{EconomyError, Transaction, TxType};
pub use vesting::VestingSchedule;
