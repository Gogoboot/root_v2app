// ============================================================
// ROOT v2.0 — api/state.rs
// Глобальное состояние приложения
//
// В мобильном приложении одно состояние на всё время работы.
// Mutex обеспечивает безопасный доступ из разных потоков.
// ============================================================

use crate::economy::Ledger;
use crate::identity::Identity;
use crate::storage::Database;
use std::sync::Mutex;

use super::types::MessageInfo;

lazy_static::lazy_static! {
    pub static ref CURRENT_IDENTITY: Mutex<Option<Identity>> = Mutex::new(None);
    pub static ref CURRENT_DB:       Mutex<Option<Database>>  = Mutex::new(None);
    pub static ref CURRENT_LEDGER:   Mutex<Option<Ledger>>    = Mutex::new(None);
    pub static ref PANIC_ACTIVATED:  Mutex<bool>              = Mutex::new(false);

    /// Канал отправки P2P сообщений в Gossipsub
    pub static ref P2P_SENDER: Mutex<Option<tokio::sync::mpsc::Sender<String>>> = Mutex::new(None);

    /// Количество подключённых пиров (обновляется из transport)
    pub static ref PEER_COUNT: Mutex<u32> = Mutex::new(0);

    /// Очередь входящих P2P сообщений — читается polling'ом из Flutter
    pub static ref INCOMING_QUEUE: Mutex<Vec<MessageInfo>> = Mutex::new(Vec::new());
}
