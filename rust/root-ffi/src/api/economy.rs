// ============================================================
// ROOT v2.0 — api/economy.rs
// FFI функции: баланс, переводы, stake, vesting
// ============================================================

use root_economy::{DROPS_PER_SAP, GENESIS_BONUS_DROPS, VestingSchedule};
use crate::require_state;
use root_core::state::AppPhase;
use super::identity::get_public_key;
use super::messaging::now_secs;
use super::state::APP_STATE;
use super::types::{ApiError, BalanceInfo, NodeStatus, P2pWarning, TxResult, VestingInfo};

pub fn get_balance() -> Result<BalanceInfo, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_ref().ok_or(ApiError::LedgerNotInitialized)?;
    let account = ledger
        .accounts
        .get(&public_key)
        .ok_or_else(|| ApiError::EconomyError("Аккаунт не найден".to_string()))?;
    let (vesting_avail, vesting_locked) = if let Some(v) = &account.vesting {
        let avail = v.available_drops();
        let locked = v.total_drops.saturating_sub(avail);
        (avail as f64 / DROPS_PER_SAP as f64, locked as f64 / DROPS_PER_SAP as f64)
    } else {
        (0.0, 0.0)
    };
    Ok(BalanceInfo {
        public_key,
        balance_sap: account.balance_drops as f64 / DROPS_PER_SAP as f64,
        balance_drops: account.balance_drops,
        staked_sap: account.staked_drops as f64 / DROPS_PER_SAP as f64,
        reputation: account.reputation,
        is_banned: account.is_banned,
        vesting_available_sap: vesting_avail,
        vesting_locked_sap: vesting_locked,
    })
}

pub fn transfer(to_key: String, amount_sap: f64) -> Result<TxResult, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    if amount_sap <= 0.0 {
        return Err(ApiError::InvalidInput("Сумма должна быть больше 0".to_string()));
    }
    let from_key = get_public_key()?;
    let amount_drops = (amount_sap * DROPS_PER_SAP as f64) as u64;
    let mut state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_mut().ok_or(ApiError::LedgerNotInitialized)?;
    let tx = ledger
        .transfer(&from_key, &to_key, amount_drops)
        .map_err(|e| ApiError::EconomyError(e.to_string()))?;
    Ok(TxResult {
        tx_id: tx.id.clone(),
        amount_sap: tx.amount_sap(),
        fee_sap: tx.fee_drops as f64 / DROPS_PER_SAP as f64,
        burned_sap: tx.burned_drops as f64 / DROPS_PER_SAP as f64,
        timestamp: tx.timestamp,
        success: true,
    })
}

pub fn p2p_exchange(to_key: String, amount_sap: f64) -> Result<TxResult, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    if amount_sap <= 0.0 {
        return Err(ApiError::InvalidInput("Сумма должна быть больше 0".to_string()));
    }
    let from_key = get_public_key()?;
    let amount_drops = (amount_sap * DROPS_PER_SAP as f64) as u64;
    let mut state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_mut().ok_or(ApiError::LedgerNotInitialized)?;
    let tx = ledger
        .p2p_exchange(&from_key, &to_key, amount_drops)
        .map_err(|e| ApiError::EconomyError(e.to_string()))?;
    Ok(TxResult {
        tx_id: tx.id.clone(),
        amount_sap: tx.amount_sap(),
        fee_sap: tx.fee_drops as f64 / DROPS_PER_SAP as f64,
        burned_sap: tx.burned_drops as f64 / DROPS_PER_SAP as f64,
        timestamp: tx.timestamp,
        success: true,
    })
}

pub fn get_p2p_warning() -> P2pWarning {
    P2pWarning {
        show_warning: true,
        message: "ROOT не контролирует фиатный канал. Банковский перевод раскрывает вашу личность контрагенту.".to_string(),
        safe_methods: vec![
            "Наличные при личной встрече".to_string(),
            "Monero (XMR)".to_string(),
            "Анонимные предоплаченные карты".to_string(),
        ],
        unsafe_methods: vec![
            "Банковский перевод (раскрывает ФИО)".to_string(),
            "СБП / Faster Payments (раскрывает телефон)".to_string(),
            "PayPal / SWIFT (раскрывает всё)".to_string(),
        ],
    }
}

pub fn get_vesting_info() -> Result<Option<VestingInfo>, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_ref().ok_or(ApiError::LedgerNotInitialized)?;
    let account = ledger
        .accounts
        .get(&public_key)
        .ok_or_else(|| ApiError::EconomyError("Аккаунт не найден".to_string()))?;
    Ok(account.vesting.as_ref().map(|v: &VestingSchedule| {
        let available = v.available_drops();
        let locked = v.total_drops.saturating_sub(available);
        let pct = available as f64 / v.total_drops as f64 * 100.0;
        let days_passed = (now_secs() - v.grant_timestamp) / 86400;
        let days_until_full = 365_u64.saturating_sub(days_passed);
        VestingInfo {
            total_sap: v.total_drops as f64 / DROPS_PER_SAP as f64,
            available_sap: available as f64 / DROPS_PER_SAP as f64,
            locked_sap: locked as f64 / DROPS_PER_SAP as f64,
            percent_unlocked: pct,
            fully_unlocked: v.is_fully_unlocked(),
            days_until_full,
        }
    }))
}

pub fn stake_node() -> Result<bool, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let mut state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_mut().ok_or(ApiError::LedgerNotInitialized)?;
    ledger.stake(&public_key).map_err(|e| ApiError::EconomyError(e.to_string()))?;
    Ok(true)
}

pub fn unstake_node() -> Result<bool, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let mut state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_mut().ok_or(ApiError::LedgerNotInitialized)?;
    ledger.unstake(&public_key).map_err(|e| ApiError::EconomyError(e.to_string()))?;
    Ok(true)
}

pub fn get_node_status() -> Result<NodeStatus, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_ref().ok_or(ApiError::LedgerNotInitialized)?;
    let account = ledger
        .accounts
        .get(&public_key)
        .ok_or_else(|| ApiError::EconomyError("Аккаунт не найден".to_string()))?;
    Ok(NodeStatus {
        public_key: public_key.clone(),
        is_active: account.is_active_node(),
        reputation: account.reputation,
        staked_sap: account.staked_drops as f64 / DROPS_PER_SAP as f64,
        offense_count: account.offense_count,
        genesis_claimed: account.genesis_claimed,
        tx_count: account.tx_history.len(),
        peer_count: state.peer_count,
        network: crate::NETWORK_ID.to_string(),
        version: crate::VERSION.to_string(),
    })
}

pub fn claim_genesis(ip: String, device_id: String) -> Result<f64, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let public_key = get_public_key()?;
    let mut state = APP_STATE.lock().unwrap();
    let ledger = state.ledger.as_mut().ok_or(ApiError::LedgerNotInitialized)?;
    ledger
        .claim_genesis_bonus(&public_key, &ip, &device_id)
        .map_err(|e| ApiError::EconomyError(e.to_string()))?;
    Ok(GENESIS_BONUS_DROPS as f64 / DROPS_PER_SAP as f64)
}
