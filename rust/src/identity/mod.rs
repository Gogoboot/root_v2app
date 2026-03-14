// ============================================================
// ROOT v2.0 — identity/mod.rs
//
// Подмодули:
//   seed      — SecretSeed (обёртка с zeroize)
//   keys      — Identity (Ed25519 генерация, подпись)
//   shamir    — ShamirVault (разделение ключа 3/5)
//   protected — ProtectedKey (XOR маскировка в памяти)
// ============================================================

pub mod seed;
pub mod keys;
pub mod shamir;
pub mod protected;

pub use seed::SecretSeed;
pub use keys::Identity;
pub use shamir::{ShamirVault, ShamirError};
pub use protected::ProtectedKey;
