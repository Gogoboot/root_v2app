// ============================================================
// ROOT v2.0 — CLI
// Терминальный интерфейс: пользователи + bootstrap сервер
// ============================================================

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "root-cli")]
#[command(about = "ROOT v2.0 — Decentralized P2P CLI")]
#[command(version = "2.0.0-alpha")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Управление идентичностью
    Identity {
        #[command(subcommand)]
        action: commands::identity::IdentityAction,
    },
    /// P2P узел (пользовательский режим)
    Node {
        #[command(subcommand)]
        action: commands::node::NodeAction,
    },
    /// Bootstrap сервер (VPS режим)
    Server {
        #[command(subcommand)]
        action: commands::server::ServerAction,
    },
    /// Управление контактами
    Contacts {
        #[command(subcommand)]
        action: commands::contacts::ContactsAction,
    },
    /// Сообщения
    Messages {
        #[command(subcommand)]
        action: commands::messages::MessagesAction,
    },
    /// База данных
    Db {
        #[command(subcommand)]
        action: commands::db::DbAction,
    },
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     ROOT v2.0  —  P2P CLI            ║");
    println!("║     Decentralized  •  E2E             ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Identity { action } => {
            commands::identity::run(action).await
        }
        Commands::Node { action } => {
            commands::node::run(action).await
        }
        Commands::Server { action } => {
            commands::server::run(action).await
        }
        Commands::Contacts { action } => {
            commands::contacts::run(action).await
        }
        Commands::Messages { action } => {
            commands::messages::run(action).await
        }
        Commands::Db { action } => {
            commands::db::run(action).await
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Ошибка: {}", e);
        std::process::exit(1);
    }
}
