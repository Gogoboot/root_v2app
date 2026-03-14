// ============================================================
// ROOT v2.0 — economy/consensus.rs
// Динамические свидетели для Proof-of-Relay
// ============================================================

/// Конфигурация свидетелей для конкретного вознаграждения
#[derive(Debug)]
pub struct WitnessConfig {
    /// Сколько свидетелей нужно
    pub count: usize,
    /// Минимальный кворум для подтверждения
    pub quorum: usize,
    /// Таймаут ожидания подтверждений (секунды)
    pub timeout: u64,
}

/// Рассчитать требования к свидетелям исходя из суммы вознаграждения
///
/// Чем больше вознаграждение — тем строже требования:
/// - до 100 drops:        1 свидетель, кворум 1, таймаут 2с
/// - до 10K drops:        3 свидетеля, кворум 2, таймаут 10с
/// - до 1M drops:         5 свидетелей, кворум 3, таймаут 30с
/// - больше 1M drops:     7 свидетелей, кворум 4, таймаут 120с
pub fn witness_config_for_reward(reward_drops: u64) -> WitnessConfig {
    match reward_drops {
        0..=100            => WitnessConfig { count: 1, quorum: 1, timeout: 2   },
        101..=10_000       => WitnessConfig { count: 3, quorum: 2, timeout: 10  },
        10_001..=1_000_000 => WitnessConfig { count: 5, quorum: 3, timeout: 30  },
        _                  => WitnessConfig { count: 7, quorum: 4, timeout: 120 },
    }
}
