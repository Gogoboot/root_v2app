// ============================================================
// ROOT v2.0 — identity/mod.rs
//
// Подмодули:
//   seed      — SecretSeed (обёртка с zeroize)
//   keys      — Identity (Ed25519 генерация, подпись)
//   shamir    — ShamirVault (разделение ключа 3/5)
//   protected — ProtectedKey (XOR маскировка в памяти)
// ============================================================

pub mod keys;
pub mod protected;
pub mod seed;
pub mod shamir;

pub use keys::Identity;
pub use protected::ProtectedKey;
pub use seed::SecretSeed;
pub use shamir::{ShamirError, ShamirVault};
