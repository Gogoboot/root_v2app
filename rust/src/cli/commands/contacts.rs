// ============================================================
// ROOT v2.0 — CLI: команды contacts
// ============================================================

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ContactsAction {
    /// Добавить контакт
    Add {
        /// Публичный ключ (64 символа hex)
        key: String,
        /// Псевдоним
        name: String,
    },
    /// Список контактов
    List,
}

pub async fn run(action: ContactsAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ContactsAction::Add { key, name } => cmd_add(key, name),
        ContactsAction::List              => cmd_list(),
    }
    Ok(())
}

fn cmd_add(key: String, name: String) {
    // Валидация ключа
    if key.len() != 64 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("❌ Неверный публичный ключ — должен быть 64 символа hex");
        return;
    }
    println!("✅ Контакт добавлен: {} → {}", &key[..16], name);
    println!("   (TODO: сохранить в БД через api::contacts::add_contact)");
}

fn cmd_list() {
    println!("📋 Список контактов:");
    println!("   (TODO: загрузить из БД через api::contacts::get_contacts)");
    println!("   Используй 'root-cli db unlock' для доступа к БД");
}
