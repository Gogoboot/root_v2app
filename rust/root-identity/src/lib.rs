// ============================================================
// root-identity — домен идентичности
//
// Ed25519 ключи, BIP-39 мнемоника, Shamir, защита в памяти
// ============================================================

pub mod keys;
pub mod protected;
pub mod seed;
pub mod shamir;

pub use keys::Identity;
pub use protected::ProtectedKey;
pub use seed::SecretSeed;
pub use shamir::{ShamirError, ShamirVault};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let (identity, mnemonic) = Identity::generate();
        let key = hex::encode(identity.verifying_key.as_bytes());
        // публичный ключ — 32 байта = 64 hex символа
        assert_eq!(key.len(), 64);
        // мнемоника — 24 слова
        assert_eq!(mnemonic.to_string().split_whitespace().count(), 24);
    }

    #[test]
    fn test_restore_from_mnemonic() {
        let (original, mnemonic) = Identity::generate();
        let restored = Identity::from_mnemonic(&mnemonic);
        // восстановленный ключ совпадает с оригиналом
        assert_eq!(
            original.verifying_key.as_bytes(),
            restored.verifying_key.as_bytes()
        );
    }

    #[test]
    fn test_sign_and_verify() {
        use ed25519_dalek::Verifier;
        let (identity, _) = Identity::generate();
        let message = b"hello root";
        let signature = identity.sign(message);
        // подпись верифицируется публичным ключом
        assert!(identity.verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_shamir_split_and_recover() {
        let vault = ShamirVault::new();
        let secret = [42u8; 32];
        let shards = vault.split(&secret).unwrap();
        assert_eq!(shards.len(), 5);
        // берём любые 3 из 5 шардов
        let recovered = vault.recover(&shards[0..3]).unwrap();
        assert_eq!(secret, recovered);
    }
}
