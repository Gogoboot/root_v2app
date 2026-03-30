// ============================================================
// root-core — AppState
// Единое состояние приложения
// ============================================================

use root_identity::Identity;
use root_storage::Database;
use root_economy::Ledger;

/// Входящее P2P сообщение — простой тип без зависимости на root-ffi
pub struct IncomingMessage {
    pub from_peer: String,
    pub content:   String,
    pub timestamp: u64,
}

/// Единое состояние приложения.
/// Живёт за Arc<Mutex<AppState>> в root-ffi.
pub struct AppState {
    pub identity:        Option<Identity>,
    pub database:        Option<Database>,
    pub ledger:          Option<Ledger>,
    pub panic_activated: bool,
    pub p2p_sender:      Option<tokio::sync::mpsc::Sender<String>>,
    pub p2p_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    pub peer_count:      u32,
    pub incoming_queue:  Vec<IncomingMessage>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            identity:        None,
            database:        None,
            ledger:          None,
            panic_activated: false,
            p2p_sender:      None,
            p2p_shutdown: None,
            peer_count:      0,
            incoming_queue:  Vec::new(),
        }
    }

    /// Проверка что identity инициализирована
    pub fn require_identity(&self) -> Option<&Identity> {
        self.identity.as_ref()
    }

    /// Проверка что БД открыта
    pub fn require_database(&self) -> Option<&Database> {
        self.database.as_ref()
    }

    /// Проверка что ledger инициализирован
    pub fn require_ledger(&mut self) -> Option<&mut Ledger> {
        self.ledger.as_mut()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
