// ============================================================
// ROOT v2.0 — identity/shamir.rs
// ShamirVault — разделение ключа 3/5 (Shamir Secret Sharing)
// ============================================================

use sharks::{Share, Sharks};
use zeroize::Zeroize;

#[derive(Debug)]
pub enum ShamirError {
    NotEnoughShares { got: usize, need: usize },
    InvalidShare,
    RecoveryFailed,
}

/// Разделение приватного ключа на 5 частей
/// Для восстановления достаточно любых 3 из 5
pub struct ShamirVault {
    threshold: u8,
    total: u8,
}

impl ShamirVault {
    pub fn new() -> Self {
        ShamirVault {
            threshold: 3,
            total: 5,
        }
    }

    /// Разделить ключ на 5 шардов
    pub fn split(&self, key_bytes: &[u8; 32]) -> Vec<Vec<u8>> {
        let sharks = Sharks(self.threshold);
        let dealer = sharks.dealer(key_bytes);
        dealer
            .take(self.total as usize)
            .map(|share| Vec::from(&share))
            .collect()
    }

    /// Восстановить ключ из любых 3 шардов
    pub fn recover(&self, shares: &[Vec<u8>]) -> Result<[u8; 32], ShamirError> {
        if shares.len() < self.threshold as usize {
            return Err(ShamirError::NotEnoughShares {
                got: shares.len(),
                need: self.threshold as usize,
            });
        }

        let sharks = Sharks(self.threshold);

        let parsed: Result<Vec<Share>, _> = shares
            .iter()
            .map(|s| Share::try_from(s.as_slice()))
            .collect();

        let parsed = parsed.map_err(|_| ShamirError::InvalidShare)?;

        let mut secret = sharks
            .recover(&parsed)
            .map_err(|_| ShamirError::RecoveryFailed)?;

        let mut result = [0u8; 32];
        result.copy_from_slice(&secret[..32]);
        secret.zeroize();

        Ok(result)
    }
}
