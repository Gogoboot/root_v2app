// ============================================================
// ROOT v2.0 — CLI: команды messages
// ============================================================

use clap::Subcommand;

#[derive(Subcommand)]
pub enum MessagesAction {
    /// Показать историю сообщений
    List,
    /// Отправить сообщение контакту
    Send {
        /// Публичный ключ получателя
        to: String,
        /// Текст сообщения
        text: String,
    },
}

pub async fn run(action: MessagesAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MessagesAction::List => cmd_list(),
        MessagesAction::Send { to, text } => cmd_send(to, text),
    }
    Ok(())
}

fn cmd_list() {
    println!("📨 История сообщений:");
    println!("   (TODO: загрузить через api::messaging::get_messages)");
    println!("   Используй 'root-cli db unlock' для доступа к БД");
}

fn cmd_send(to: String, text: String) {
    if to.len() != 64 || !to.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("❌ Неверный публичный ключ получателя");
        return;
    }
    println!("📤 Отправка: → {}... | \"{}\"", &to[..16], text);
    println!("   (TODO: отправить через api::messaging::send_message + P2P)");
}
