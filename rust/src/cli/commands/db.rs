// ============================================================
// ROOT v2.0 — CLI: команды db
// ============================================================

use clap::Subcommand;

#[derive(Subcommand)]
pub enum DbAction {
    /// Разблокировать БД паролем
    Unlock {
        /// Пароль
        #[arg(short, long)]
        password: String,
        /// Путь к файлу БД
        #[arg(short, long, default_value = "root.db")]
        path: String,
    },
    /// Проверить целостность БД
    Verify,
}

pub async fn run(action: DbAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        DbAction::Unlock { password, path } => cmd_unlock(password, path),
        DbAction::Verify => cmd_verify(),
    }
    Ok(())
}

fn cmd_unlock(password: String, path: String) {
    println!("🔓 Открываем БД: {}", path);
    println!("   (TODO: вызвать api::database::unlock_database)");
    println!("   Пароль получен: {} символов", password.len());
}

fn cmd_verify() {
    println!("🔍 Проверка целостности БД...");
    println!("   (TODO: вызвать api::database::verify_db_integrity)");
}
