// ============================================================
// root-core — AppState
// Единое состояние приложения
// ============================================================

use root_identity::Identity;
use root_network::P2pOutMessage;
use root_storage::Database;
use root_economy::Ledger;

/// Фаза жизненного цикла приложения
/// Определяет какие операции разрешены в данный момент
#[derive(Debug, Clone, PartialEq)]
pub enum AppPhase {
    /// S0 — приложение только запущено, ничего не инициализировано
    Fresh,
    /// S1 — БД открыта, ключей нет
    DbOpen,
    /// S2 — ключи есть, БД не открыта
    Identified,
    /// S3 — всё готово к работе
    Ready,
    /// S4 — P2P сеть активна
    P2PActive,
    /// S5 — терминальное состояние, только перезапуск
    Panicked,
}

/// Входящее P2P сообщение
pub struct IncomingMessage {
    pub from_peer: String,
    pub content:   String,
    pub timestamp: u64,
}

/// Единое состояние приложения
pub struct AppState {
    pub phase:           AppPhase,
    pub identity:        Option<Identity>,
    pub database:        Option<Database>,
    pub ledger:          Option<Ledger>,
    pub panic_activated: bool,
    pub p2p_sender:      Option<tokio::sync::mpsc::Sender<P2pOutMessage>>,
    pub p2p_shutdown:    Option<tokio::sync::oneshot::Sender<()>>,
    pub peer_count:      u32,
    pub incoming_queue:  Vec<IncomingMessage>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            phase:           AppPhase::Fresh,
            identity:        None,
            database:        None,
            ledger:          None,
            panic_activated: false,
            p2p_sender:      None,
            p2p_shutdown:    None,
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

    /// Переход в новую фазу с проверкой допустимости
    pub fn transition(&mut self, new_phase: AppPhase) -> bool {
        let allowed = match (&self.phase, &new_phase) {
            (AppPhase::Fresh,      AppPhase::DbOpen)     => true,
            (AppPhase::Fresh,      AppPhase::Identified)  => true,
            (AppPhase::DbOpen,     AppPhase::Ready)       => true,
            (AppPhase::Identified, AppPhase::Ready)       => true,
            (AppPhase::Ready,      AppPhase::P2PActive)   => true,
            (AppPhase::P2PActive,  AppPhase::Ready)       => true,
            // Из любого состояния можно перейти в Panicked
            (_,                    AppPhase::Panicked)    => true,
            _ => false,
        };
        if allowed {
            self.phase = new_phase;
        }
        allowed
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
