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

pub mod constants;
pub mod types;
pub mod vesting;
pub mod protection;
pub mod account;
pub mod treasury;
pub mod ledger;
pub mod consensus;

// ── Реэкспорт для удобства ───────────────────────────────────
// Снаружи можно писать: use crate::economy::Ledger;
// вместо: use crate::economy::ledger::Ledger;

pub use constants::*;
pub use types::{EconomyError, Transaction, TxType};
pub use vesting::VestingSchedule;
pub use protection::{VelocityTracker, AnomalyDetector, PersonhoodRegistry};
pub use account::Account;
pub use treasury::Treasury;
pub use ledger::Ledger;
pub use consensus::{WitnessConfig, witness_config_for_reward};
