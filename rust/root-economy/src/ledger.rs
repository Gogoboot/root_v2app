// ============================================================
// ROOT v2.0 — economy/ledger.rs
// Главный движок экономики ROOT
// ============================================================

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::account::Account;
use super::consensus::witness_config_for_reward;
use super::constants::{
    DROPS_PER_SAP, EMISSION_STAGE_1_SAP, GENESIS_BONUS_DROPS, GENESIS_MAX_NODES, HARD_CAP_DROPS,
    MAX_RELAY_REWARD_DROPS, MIN_REPUTATION_FOR_P2P, MIN_STAKE_DROPS, P2P_BURN_PCT, SLASH_PCT,
    SLASH_REP, TX_FEE_PERCENT, VESTING_IMMEDIATE_PCT, WITNESS_REWARD_PCT,
};
use super::protection::PersonhoodRegistry;
use super::treasury::Treasury;
use super::types::{EconomyError, Transaction, TxType};
use super::vesting::VestingSchedule;

pub struct Ledger {
    pub accounts: HashMap<String, Account>,
    pub treasury: Treasury,
    pub total_supply_drops: u64,
    /// Сожжено навсегда через P2P burn
    pub burned_supply_drops: u64,
    /// Текущий лимит эмиссии (разблокируется по DAU)
    pub emission_limit_drops: u64,
    pub transactions: Vec<Transaction>,
    pub genesis_nodes_count: u32,
    pub personhood: PersonhoodRegistry,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: HashMap::new(),
            treasury: Treasury::new(),
            total_supply_drops: 0,
            burned_supply_drops: 0,
            emission_limit_drops: EMISSION_STAGE_1_SAP * DROPS_PER_SAP,
            transactions: Vec::new(),
            genesis_nodes_count: 0,
            personhood: PersonhoodRegistry::new(),
        }
    }

    /// Разблокировать следующий этап эмиссии по количеству DAU
    pub fn unlock_emission_stage(&mut self, dau: u64) {
        use super::constants::{EMISSION_STAGE_2_SAP, EMISSION_STAGE_3_SAP, EMISSION_STAGE_4_SAP};
        let new_sap = if dau >= 1_000_000_000 {
            EMISSION_STAGE_1_SAP
                + EMISSION_STAGE_2_SAP
                + EMISSION_STAGE_3_SAP
                + EMISSION_STAGE_4_SAP
        } else if dau >= 100_000_000 {
            EMISSION_STAGE_1_SAP + EMISSION_STAGE_2_SAP + EMISSION_STAGE_3_SAP
        } else if dau >= 10_000_000 {
            EMISSION_STAGE_1_SAP + EMISSION_STAGE_2_SAP
        } else {
            EMISSION_STAGE_1_SAP
        };
        let new_drops = new_sap * DROPS_PER_SAP;
        if new_drops > self.emission_limit_drops {
            println!(
                "  🔓 Эмиссия разблокирована до {} SAP (DAU={})",
                new_sap, dau
            );
            self.emission_limit_drops = new_drops;
        }
    }

    /// Получить или создать аккаунт
    pub fn get_or_create(&mut self, key: &str) -> &mut Account {
        self.accounts
            .entry(key.to_string())
            .or_insert_with(|| Account::new(key.to_string()))
    }

    /// Эмиссия новых токенов (внутренняя)
    fn mint(&mut self, to: &str, amount: u64) -> Result<(), EconomyError> {
        if self.total_supply_drops + amount > HARD_CAP_DROPS {
            return Err(EconomyError::HardCapExceeded {
                current: self.total_supply_drops,
                cap: HARD_CAP_DROPS,
            });
        }
        if self.total_supply_drops + amount > self.emission_limit_drops {
            return Err(EconomyError::InvalidTransaction(format!(
                "Превышен лимит эмиссии: {} SAP",
                self.emission_limit_drops / DROPS_PER_SAP
            )));
        }
        self.get_or_create(to).balance_drops += amount;
        self.total_supply_drops += amount;
        Ok(())
    }

    fn calc_fee(amount: u64) -> u64 {
        ((amount as f64) * TX_FEE_PERCENT) as u64
    }

    // ── Обычный перевод ──────────────────────────────────────

    pub fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount_drops: u64,
    ) -> Result<Transaction, EconomyError> {
        if self
            .accounts
            .get(from)
            .map(|a| a.is_banned)
            .unwrap_or(false)
        {
            return Err(EconomyError::NodeBanned);
        }
        if let Some(acc) = self.accounts.get(from) {
            acc.anomaly.check_frozen()?;
        }
        self.get_or_create(from).check_rate_limit()?;

        let fee = Self::calc_fee(amount_drops);
        let total = amount_drops + fee;
        let balance = self.get_or_create(from).balance_drops;
        if balance < total {
            return Err(EconomyError::InsufficientFunds {
                need: total,
                have: balance,
            });
        }

        self.accounts.get_mut(from).unwrap().balance_drops -= total;
        self.get_or_create(to).balance_drops += amount_drops;
        self.treasury.deposit(fee, false);

        let tx = Transaction::new(
            from.to_string(),
            to.to_string(),
            amount_drops,
            fee,
            0,
            TxType::Transfer,
        );
        self.accounts.get_mut(from).unwrap().add_tx(tx.id.clone());
        self.transactions.push(tx.clone());

        println!(
            "  ✅ Перевод: {:.4} SAP | {}... → {}... | fee: {} Drops",
            tx.amount_sap(),
            &from[..8],
            &to[..8],
            fee
        );
        Ok(tx)
    }

    // ── P2P обмен (с защитой от серых схем) ─────────────────

    pub fn p2p_exchange(
        &mut self,
        from: &str,
        to: &str,
        amount_drops: u64,
    ) -> Result<Transaction, EconomyError> {
        if self
            .accounts
            .get(from)
            .map(|a| a.is_banned)
            .unwrap_or(false)
        {
            return Err(EconomyError::NodeBanned);
        }
        if let Some(acc) = self.accounts.get(from) {
            acc.anomaly.check_frozen()?;
        }

        // Проверка репутации для P2P
        let reputation = self.accounts.get(from).map(|a| a.reputation).unwrap_or(0);
        if reputation < MIN_REPUTATION_FOR_P2P {
            return Err(EconomyError::InsufficientReputation {
                required: MIN_REPUTATION_FOR_P2P,
                have: reputation,
            });
        }

        // Velocity Limit
        {
            self.get_or_create(from)
                .velocity
                .check_and_record(amount_drops)?;
        }

        // Vesting проверка
        {
            let acc = self.get_or_create(from);
            if let Some(vesting) = &mut acc.vesting
                && !vesting.is_fully_unlocked()
            {
                vesting.spend(amount_drops)?;

                // if !vesting.is_fully_unlocked() {
                //     vesting.spend(amount_drops)?;
            }
        }

        // Burn 1% при P2P
        let burn_amount = (amount_drops as f64 * P2P_BURN_PCT) as u64;
        let fee = Self::calc_fee(amount_drops);
        let total_needed = amount_drops + fee + burn_amount;

        let balance = self.get_or_create(from).balance_drops;
        if balance < total_needed {
            return Err(EconomyError::InsufficientFunds {
                need: total_needed,
                have: balance,
            });
        }

        self.accounts.get_mut(from).unwrap().balance_drops -= total_needed;
        self.get_or_create(to).balance_drops += amount_drops;
        self.treasury.deposit(fee, false);

        // Burn — уменьшаем total_supply навсегда
        self.total_supply_drops -= burn_amount;
        self.burned_supply_drops += burn_amount;
        self.treasury.total_burned_drops += burn_amount;

        // Детектор аномалий
        let total_balance = self
            .accounts
            .get(from)
            .map(|a| a.balance_drops)
            .unwrap_or(0);
        let genesis_age = self.accounts.get(from).and_then(|a| a.genesis_age_secs());
        if let Some(acc) = self.accounts.get_mut(from) {
            acc.anomaly
                .record_sale(amount_drops, total_balance, genesis_age);
        }

        let tx = Transaction::new(
            from.to_string(),
            to.to_string(),
            amount_drops,
            fee,
            burn_amount,
            TxType::P2PExchange,
        );
        self.accounts.get_mut(from).unwrap().add_tx(tx.id.clone());
        self.transactions.push(tx.clone());

        println!(
            "  🔄 P2P обмен: {:.4} SAP | {}... → {}... | burn: {} Drops 🔥",
            tx.amount_sap(),
            &from[..8],
            &to[..8],
            burn_amount
        );
        Ok(tx)
    }

    // ── Stake / Unstake ──────────────────────────────────────

    pub fn stake(&mut self, key: &str) -> Result<(), EconomyError> {
        if self.accounts.get(key).map(|a| a.is_banned).unwrap_or(false) {
            return Err(EconomyError::NodeBanned);
        }
        let balance = self.get_or_create(key).balance_drops;
        if balance < MIN_STAKE_DROPS {
            return Err(EconomyError::InsufficientStake {
                need: MIN_STAKE_DROPS,
            });
        }
        let acc = self.accounts.get_mut(key).unwrap();
        acc.balance_drops -= MIN_STAKE_DROPS;
        acc.staked_drops += MIN_STAKE_DROPS;
        println!("  🔒 Stake: 10 SAP | узел {}...", &key[..8]);
        Ok(())
    }

    pub fn unstake(&mut self, key: &str) -> Result<(), EconomyError> {
        let acc = self
            .accounts
            .get_mut(key)
            .ok_or_else(|| EconomyError::NodeNotFound(key.to_string()))?;
        let staked = acc.staked_drops;
        acc.staked_drops = 0;
        acc.balance_drops += staked;
        println!(
            "  🔓 Unstake: {} SAP | {}...",
            staked / DROPS_PER_SAP,
            &key[..8]
        );
        Ok(())
    }

    // ── Proof-of-Relay вознаграждение ────────────────────────

    pub fn reward_relay(
        &mut self,
        relay_node: &str,
        relayed_bytes: u64,
        witnesses: Vec<String>,
    ) -> Result<Transaction, EconomyError> {
        if !self
            .accounts
            .get(relay_node)
            .map(|a| a.is_active_node())
            .unwrap_or(false)
        {
            return Err(EconomyError::InsufficientStake {
                need: MIN_STAKE_DROPS,
            });
        }

        let base = ((relayed_bytes / 10_240) * 100).clamp(1, MAX_RELAY_REWARD_DROPS);

        let multiplier = self.treasury.reward_multiplier(self.total_supply_drops);
        let reward = (base as f64 * multiplier) as u64;
        if reward == 0 {
            return Err(EconomyError::TreasuryReserveLocked);
        }

        let cfg = witness_config_for_reward(reward);
        let witness_reward = (reward as f64 * WITNESS_REWARD_PCT) as u64;
        let total_payout = reward + witness_reward * witnesses.len() as u64;

        self.treasury
            .withdraw(total_payout, self.total_supply_drops)?;
        self.get_or_create(relay_node).balance_drops += reward;
        if let Some(acc) = self.accounts.get_mut(relay_node) {
            acc.reputation = (acc.reputation + 1).min(100);
        }
        for w in &witnesses {
            self.get_or_create(w).balance_drops += witness_reward;
            if let Some(acc) = self.accounts.get_mut(w.as_str()) {
                acc.reputation = (acc.reputation + 1).min(100);
            }
        }

        let tx = Transaction::new(
            "TREASURY".to_string(),
            relay_node.to_string(),
            reward,
            0,
            0,
            TxType::RelayReward {
                relayed_bytes,
                witnesses: witnesses.clone(),
            },
        );
        self.transactions.push(tx.clone());
        println!(
            "  📡 Relay reward: {} Drops → {}... | {} свид. (кворум {})",
            reward,
            &relay_node[..8],
            cfg.count,
            cfg.quorum
        );
        Ok(tx)
    }

    // ── Прогрессивный Slash ──────────────────────────────────

    pub fn slash(&mut self, offender: &str) -> Result<(), EconomyError> {
        let offense_count = {
            let acc = self.get_or_create(offender);
            acc.offense_count += 1;
            acc.offense_count
        };
        let idx = ((offense_count - 1) as usize).min(3);
        let slash_pct = SLASH_PCT[idx];
        let rep_penalty = SLASH_REP[idx];
        let staked = self
            .accounts
            .get(offender)
            .map(|a| a.staked_drops)
            .unwrap_or(0);
        let slash_amount = (staked as f64 * slash_pct) as u64;

        let acc = self.accounts.get_mut(offender).unwrap();
        acc.reputation = acc.reputation.saturating_sub(rep_penalty);
        if slash_amount > 0 {
            acc.staked_drops -= slash_amount;
            self.treasury.deposit(slash_amount, true);
        }
        if offense_count >= 4 {
            acc.is_banned = true;
            println!("  🚫 БАН: {}... | 100% slash → Treasury", &offender[..8]);
        } else {
            println!(
                "  ⚠️  Нарушение #{} | {}... | slash {:.0}% | rep: {}",
                offense_count,
                &offender[..8],
                slash_pct * 100.0,
                self.accounts[offender].reputation
            );
        }
        Ok(())
    }

    // ── Genesis с Proof of Personhood + Vesting ──────────────

    pub fn claim_genesis_bonus(
        &mut self,
        key: &str,
        ip: &str,
        device_id: &str,
    ) -> Result<(), EconomyError> {
        if self.genesis_nodes_count >= GENESIS_MAX_NODES {
            return Err(EconomyError::GenesisEnded(GENESIS_MAX_NODES));
        }
        if self
            .accounts
            .get(key)
            .map(|a| a.genesis_claimed)
            .unwrap_or(false)
        {
            return Err(EconomyError::InvalidTransaction(
                "Genesis бонус уже получен".to_string(),
            ));
        }

        self.personhood.check_and_register(ip, device_id)?;
        self.mint(key, GENESIS_BONUS_DROPS)?;

        let acc = self.accounts.get_mut(key).unwrap();
        acc.genesis_claimed = true;
        acc.genesis_timestamp = Some(now_secs());
        acc.vesting = Some(VestingSchedule::new(GENESIS_BONUS_DROPS));

        self.genesis_nodes_count += 1;
        println!(
            "  🎁 Genesis #{}: 100 SAP → {}... | доступно сразу: {} SAP | осталось: {}",
            self.genesis_nodes_count,
            &key[..8],
            (GENESIS_BONUS_DROPS as f64 * VESTING_IMMEDIATE_PCT) as u64 / DROPS_PER_SAP,
            GENESIS_MAX_NODES - self.genesis_nodes_count
        );
        Ok(())
    }

    // ── Статистика ───────────────────────────────────────────

    pub fn print_stats(&self) {
        let supply = self.total_supply_drops as f64 / DROPS_PER_SAP as f64;
        let burned = self.burned_supply_drops as f64 / DROPS_PER_SAP as f64;
        let cap = HARD_CAP_DROPS as f64 / DROPS_PER_SAP as f64;
        let limit = self.emission_limit_drops as f64 / DROPS_PER_SAP as f64;
        let pct = (self.total_supply_drops as f64 / self.emission_limit_drops as f64) * 100.0;

        println!("\n  ╔══════════════════════════════════════════════╗");
        println!("  ║       СТАТИСТИКА ЭКОНОМИКИ ROOT v2.0         ║");
        println!("  ╠══════════════════════════════════════════════╣");
        println!("  ║ Hard Cap:        {:>14.0} SAP           ║", cap);
        println!("  ║ Лимит эмиссии:   {:>14.0} SAP           ║", limit);
        println!("  ║ Выпущено:        {:>11.4} SAP ({:.2}%)  ║", supply, pct);
        println!("  ║ Сожжено (P2P):   {:>14.6} SAP           ║", burned);
        println!(
            "  ║ Treasury:        {:>14.4} SAP           ║",
            self.treasury.balance_sap()
        );
        println!(
            "  ║ Счетов:          {:>14}               ║",
            self.accounts.len()
        );
        println!(
            "  ║ Транзакций:      {:>14}               ║",
            self.transactions.len()
        );
        println!(
            "  ║ Genesis узлов:   {:>7}/{:<5}               ║",
            self.genesis_nodes_count, GENESIS_MAX_NODES
        );
        println!("  ╚══════════════════════════════════════════════╝\n");
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
