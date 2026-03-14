// ============================================================
// ROOT v2.0 — identity/shamir.rs
// ShamirVault — разделение ключа 3/5
// Исправлено для sharks 0.5.0 — правильный API
// ============================================================

use sharks::{Share, Sharks};
use zeroize::Zeroize;

#[derive(Debug)]
pub enum ShamirError {
    NotEnoughShares { got: usize, need: usize },
    InvalidThreshold,
    InvalidShare,
    RecoveryFailed,
}

/// Шард как сериализованные байты — официальный API sharks
pub type Shard = Vec<u8>;

pub struct ShamirVault {
    threshold: u8,
    total:     u8,
}

impl Default for ShamirVault {
    fn default() -> Self { Self::new() }
}

impl ShamirVault {
    pub fn new() -> Self {
        ShamirVault { threshold: 3, total: 5 }
    }

    /// Разделить ключ на шарды (sharks 0.5.0 API)
    pub fn split(&self, key_bytes: &[u8; 32]) -> Result<Vec<Shard>, ShamirError> {
        if self.threshold == 0 || self.threshold > self.total {
            return Err(ShamirError::InvalidThreshold);
        }

        // ✅ sharks 0.5: кортежная структура, не ::new()
        let sharks = Sharks(self.threshold);
        
        // ✅ sharks 0.5: метод dealer() возвращает Iterator<Item=Share>
        let shards: Vec<Shard> = sharks
            .dealer(key_bytes.as_slice())
            .take(self.total as usize)
            .map(|share| Vec::from(&share))  // ✅ Официальный API: Share → Vec<u8>
            .collect();

        Ok(shards)
    }

    /// Восстановить ключ из шардов
    pub fn recover(&self, shards: &[Shard]) -> Result<[u8; 32], ShamirError> {
        if shards.len() < self.threshold as usize {
            return Err(ShamirError::NotEnoughShares {
                got: shards.len(),
                need: self.threshold as usize,
            });
        }

        // ✅ sharks 0.5: кортежная структура
        let sharks = Sharks(self.threshold);

        // ✅ Официальный API: Vec<u8> → Share через TryFrom
        let shares_vec: Result<Vec<Share>, _> = shards
            .iter()
            .map(|bytes| Share::try_from(bytes.as_slice()))
            .collect();

        let shares = shares_vec.map_err(|_| ShamirError::InvalidShare)?;

        // ✅ recover() требует IntoIterator<Item=&Share>
        let mut secret = sharks
            .recover(shares.iter())
            .map_err(|_| ShamirError::RecoveryFailed)?;

        if secret.len() < 32 {
            secret.zeroize();
            return Err(ShamirError::RecoveryFailed);
        }

        let mut result = [0u8; 32];
        result.copy_from_slice(&secret[..32]);
        secret.zeroize();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_recover_3_of_5() {
        let vault = ShamirVault::new();
        let key = [42u8; 32];
        let shards = vault.split(&key).unwrap();
        assert_eq!(shards.len(), 5);
        let recovered = vault.recover(&shards[..3]).unwrap();
        assert_eq!(recovered, key);
    }
}