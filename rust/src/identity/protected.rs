// ============================================================
// ROOT v2.0 — identity/protected.rs
// ProtectedKey — XOR маскировка ключа в памяти
// Используется когда приложение уходит в фон
// ============================================================

use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

pub struct ProtectedKey {
    encrypted_bytes: Vec<u8>,
    mask:            [u8; 32],
}

impl ProtectedKey {
    /// Заморозить ключ — XOR с случайной маской
    pub fn freeze(mut key_bytes: [u8; 32]) -> Self {
        let mut mask = [0u8; 32];
        OsRng.fill_bytes(&mut mask);

        let encrypted: Vec<u8> = key_bytes
            .iter()
            .zip(mask.iter())
            .map(|(k, m)| k ^ m)
            .collect();

        key_bytes.zeroize();

        ProtectedKey { encrypted_bytes: encrypted, mask }
    }

    /// Разморозить ключ для использования
    /// После использования вызови .zeroize() на результате
    pub fn thaw(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        for (i, (e, m)) in self.encrypted_bytes.iter().zip(self.mask.iter()).enumerate() {
            result[i] = e ^ m;
        }
        result
    }
}

impl Drop for ProtectedKey {
    fn drop(&mut self) {
        self.encrypted_bytes.zeroize();
        self.mask.zeroize();
    }
}
