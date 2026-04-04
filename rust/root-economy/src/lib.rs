// ============================================================
// root-economy — домен экономики SAP
//
// Токен, транзакции, вестинг, защита, консенсус
// ============================================================

pub mod account;
pub mod consensus;
pub mod constants;
pub mod ledger;
pub mod protection;
pub mod treasury;
pub mod types;
pub mod vesting;
pub mod error;

pub use account::Account;
pub use consensus::{WitnessConfig, witness_config_for_reward};
pub use constants::*;
pub use ledger::Ledger;
pub use protection::{AnomalyDetector, PersonhoodRegistry, VelocityTracker};
pub use treasury::Treasury;
pub use types::{Transaction, TxType};
pub use vesting::VestingSchedule;

pub use error::EconomyError;
