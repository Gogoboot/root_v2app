// ============================================================
// root-identity — домен идентичности
//
// Ed25519 ключи, BIP-39 мнемоника, Shamir, защита в памяти
// ============================================================

pub mod constants;
pub mod keys;
pub mod protected;
pub mod seed;
pub mod shamir;
pub mod error;  // ← новый модуль

pub use constants::{BIP39_PREFIX, DERIVATION_INDEX};
pub use error::IdentityError;
pub use keys::Identity;
pub use protected::ProtectedKey;
pub use seed::SecretSeed;
pub use shamir::{ShamirError, ShamirVault};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let (identity, mnemonic) = Identity::generate().unwrap();
        let key = hex::encode(identity.verifying_key.as_bytes());
        assert_eq!(key.len(), 64);
        assert_eq!(mnemonic.to_string().split_whitespace().count(), 24);
    }

    #[test]
    fn test_restore_from_mnemonic() {
        let (original, mnemonic) = Identity::generate().unwrap();
        let restored = Identity::from_mnemonic(&mnemonic).unwrap();
        assert_eq!(
            original.verifying_key.as_bytes(),
            restored.verifying_key.as_bytes()
        );
    }

    #[test]
    fn test_sign_and_verify() {
        use ed25519_dalek::Verifier;
        let (identity, _) = Identity::generate().unwrap();
        let message = b"hello root";
        let signature = identity.sign(message);
        assert!(identity.verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_shamir_split_and_recover() {
        let vault = ShamirVault::new();
        let secret = [42u8; 32];
        let shards = vault.split(&secret).unwrap();
        assert_eq!(shards.len(), 5);
        let recovered = vault.recover(&shards[0..3]).unwrap();
        assert_eq!(secret, recovered);
    }
}
