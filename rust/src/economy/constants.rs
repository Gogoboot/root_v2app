// ============================================================
// ROOT v2.0 — economy/constants.rs
// Все константы экономики в одном месте
// ============================================================

// ── Базовая экономика ────────────────────────────────────────

/// Жёсткий лимит эмиссии: 1 миллиард SAP
pub const HARD_CAP_DROPS: u64         = 1_000_000_000 * 100_000_000;
/// 1 SAP = 100,000,000 drops (микро-единицы)
pub const DROPS_PER_SAP: u64          = 100_000_000;
/// Минимальный stake для relay-узла: 10 SAP
pub const MIN_STAKE_DROPS: u64        = 10 * DROPS_PER_SAP;
/// Максимальное вознаграждение за relay за одно сообщение
pub const MAX_RELAY_REWARD_DROPS: u64 = 100_000;
/// Комиссия за транзакцию: 0.1% → Treasury
pub const TX_FEE_PERCENT: f64         = 0.001;
/// Максимум транзакций в секунду (защита от спама)
pub const MAX_TXS_PER_SECOND: u32     = 10;
/// Genesis бонус первым 1000 узлам: 100 SAP
pub const GENESIS_BONUS_DROPS: u64    = 100 * DROPS_PER_SAP;
/// Максимум Genesis узлов
pub const GENESIS_MAX_NODES: u32      = 1000;
/// Минимальный резерв Treasury (10% от общей эмиссии)
pub const TREASURY_MIN_RESERVE_PCT: f64  = 0.10;
/// Порог низкого резерва Treasury (20%)
pub const TREASURY_LOW_THRESHOLD_PCT: f64 = 0.20;
/// Доля вознаграждения свидетелям (10% от relay reward)
pub const WITNESS_REWARD_PCT: f64     = 0.10;

// ── Поэтапная эмиссия по DAU ─────────────────────────────────
/// Этап 1: до 100M SAP (до 10K DAU)
pub const EMISSION_STAGE_1_SAP: u64 = 100_000_000;
/// Этап 2: до 150M SAP (до 50K DAU)
pub const EMISSION_STAGE_2_SAP: u64 = 150_000_000;
/// Этап 3: до 250M SAP (до 200K DAU)
pub const EMISSION_STAGE_3_SAP: u64 = 250_000_000;
/// Этап 4: до 500M SAP (до 1M DAU)
pub const EMISSION_STAGE_4_SAP: u64 = 500_000_000;

// ── Прогрессивный Slash ──────────────────────────────────────
/// Процент slash за 1/2/3/4 нарушение: 0% / 10% / 50% / 100%
pub const SLASH_PCT: [f64; 4] = [0.00, 0.10, 0.50, 1.00];
/// Снижение репутации за 1/2/3/4 нарушение
pub const SLASH_REP: [u8; 4]  = [20,   30,   50,   100];

// ── Механизм 1: Velocity Limit ───────────────────────────────
/// Максимум SAP для отправки/продажи за сутки
pub const VELOCITY_LIMIT_DROPS_PER_DAY: u64   = 100 * DROPS_PER_SAP;
/// Максимум за неделю
pub const VELOCITY_LIMIT_DROPS_PER_WEEK: u64  = 500 * DROPS_PER_SAP;
/// Максимум за месяц
pub const VELOCITY_LIMIT_DROPS_PER_MONTH: u64 = 1000 * DROPS_PER_SAP;

// ── Механизм 2: Vesting Genesis ──────────────────────────────
/// Доступно сразу при Genesis: 10%
pub const VESTING_IMMEDIATE_PCT: f64 = 0.10;
/// Расписание разблокировки: (дни, накопленный %)
pub const VESTING_SCHEDULE: [(u64, f64); 4] = [
    (30,  0.25),  // 1 месяц  → 25%
    (90,  0.50),  // 3 месяца → 50%
    (180, 0.75),  // 6 месяцев → 75%
    (365, 1.00),  // 12 месяцев → 100%
];

// ── Механизм 3: Burn при P2P ─────────────────────────────────
/// 1% от суммы P2P сделки сжигается навсегда
pub const P2P_BURN_PCT: f64 = 0.01;

// ── Механизм 4: Стабфонд Treasury ────────────────────────────
/// Доля Treasury под стабилизацию курса
pub const STABFUND_RESERVE_PCT: f64   = 0.20;
/// Порог падения курса для автовыкупа (30% за 24 часа)
pub const PRICE_DROP_TRIGGER_PCT: f64 = 0.30;

// ── Механизм 5: Proof of Personhood ──────────────────────────
/// Максимум Genesis бонусов с одного IP
pub const MAX_GENESIS_PER_IP: u32     = 3;
/// Максимум Genesis бонусов с одного устройства
pub const MAX_GENESIS_PER_DEVICE: u32 = 1;

// ── Механизм 6: Детектор аномалий ────────────────────────────
/// Порог подозрительности: продажа > 50% баланса за 24ч
pub const ANOMALY_SELL_PCT_THRESHOLD: f64 = 0.50;
/// Заморозка при аномалии: 72 часа
pub const ANOMALY_FREEZE_SECONDS: u64     = 72 * 3600;
/// Минимальная репутация для P2P торговли
pub const MIN_REPUTATION_FOR_P2P: u8      = 70;
