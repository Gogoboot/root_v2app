// ============================================================
// ROOT v2.0 — CLI: команды identity
// ============================================================

use clap::Subcommand;

#[derive(Subcommand)]
pub enum IdentityAction {
    /// Сгенерировать новый ключ и мнемонику
    Generate,
    /// Восстановить ключ из мнемоники
    Restore {
        /// 24 слова через пробел
        mnemonic: String,
    },
    /// Показать текущий публичный ключ
    Show,
}

pub async fn run(action: IdentityAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        IdentityAction::Generate  => cmd_generate(),
        IdentityAction::Restore { mnemonic } => cmd_restore(mnemonic),
        IdentityAction::Show      => cmd_show(),
    }
    Ok(())
}

fn cmd_generate() {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use rand::RngCore;

    println!("🔑 Генерируем новую идентичность...\n");

    let signing_key   = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let public_key    = hex::encode(verifying_key.as_bytes());

    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .expect("Ошибка генерации мнемоники");

    println!("✅ Публичный ключ:");
    println!("   {}\n", public_key);
    println!("✅ Мнемоника (24 слова) — ЗАПИШИТЕ НА БУМАГУ:");
    println!("   {}\n", mnemonic);
    println!("⚠️  Никогда не сохраняйте мнемонику в цифровом виде!");
}

fn cmd_restore(mnemonic: String) {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;

    println!("🔑 Восстанавливаем идентичность...\n");

    match mnemonic.parse::<Mnemonic>() {
        Ok(parsed) => {
            let entropy = parsed.to_entropy();
            let seed    = &entropy[..32.min(entropy.len())];
            let mut key_bytes = [0u8; 32];
            key_bytes[..seed.len()].copy_from_slice(seed);
            let signing_key   = SigningKey::from_bytes(&key_bytes);
            let verifying_key = signing_key.verifying_key();
            let public_key    = hex::encode(verifying_key.as_bytes());

            println!("✅ Публичный ключ восстановлен:");
            println!("   {}", public_key);
        }
        Err(e) => eprintln!("❌ Неверная мнемоника: {}", e),
    }
}

fn cmd_show() {
    println!("ℹ️  Используй 'root-cli db unlock' чтобы загрузить идентичность из БД");
    println!("   Затем ключ будет доступен в 'root-cli identity show'");
}
