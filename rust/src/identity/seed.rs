// ============================================================
// ROOT v2.0 — identity/seed.rs
// SecretSeed — обёртка над seed с автоочисткой памяти
// ============================================================

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Обёртка над 64-байтным seed
/// Автоматически обнуляет память при drop()
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretSeed(pub [u8; 64]);
