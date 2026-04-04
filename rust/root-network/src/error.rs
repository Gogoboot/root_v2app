// root-network/src/error.rs
use thiserror::Error;

/// Ошибки сетевого модуля (P2P, libp2p, каналы, доставка сообщений).
/// 
/// Все ошибки реализуют `std::error::Error` и могут быть 
/// автоматически конвертированы в `FfiError` через `#[from]`.
#[derive(Error, Debug)]
pub enum NetworkError {
    // ─── Подключение и транспорт ─────────────────────────
    
    /// Не удалось установить соединение с пирам
    #[error("Не удалось подключиться к пиру: {0}")]
    ConnectionFailed(String),
    
    /// Соединение разорвано в процессе работы
    #[error("Соединение с пиром {peer_id} разорвано")]
    ConnectionLost { peer_id: String },
    
    /// Таймаут при ожидании ответа от пира
    #[error("Таймаут ожидания ответа от пира: {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    /// Ошибка на уровне транспорта (TCP, QUIC, WebSocket)
    #[error("Транспортная ошибка: {0}")]
    Transport(String),

    // ─── Протокол и валидация ───────────────────────────
    
    /// Получено сообщение, нарушающее протокол
    #[error("Нарушение протокола: {0}")]
    ProtocolViolation(String),
    
    /// Сообщение не прошло валидацию (подпись, формат, версия)
    #[error("Невалидное сообщение: {0}")]
    InvalidMessage(String),
    
    /// Неизвестный тип сообщения или команды
    #[error("Неизвестный тип сообщения: {type_id}")]
    UnknownMessageType { type_id: u8 },

    // ─── Доставка и каналы ──────────────────────────────
    
    /// Не удалось отправить сообщение (канал переполнен / закрыт)
    #[error("Не удалось отправить сообщение: {0}")]
    SendFailed(String),
    
    /// Ошибка при получении сообщения из канала
    #[error("Не удалось получить сообщение: {0}")]
    ReceiveFailed(String),
    
    /// Канал связи с пиром закрыт
    #[error("Канал с пиром {peer_id} закрыт")]
    ChannelClosed { peer_id: String },

    // ─── Обнаружение пиров и DHT ────────────────────────
    
    /// Пир не найден в таблице маршрутизации
    #[error("Пир {peer_id} не найден в DHT")]
    PeerNotFound { peer_id: String },
    
    /// Ошибка при запросе к DHT
    #[error("Ошибка DHT-запроса: {0}")]
    DhtQueryFailed(String),

    // ─── Крипто-зависимости (делегирование) ─────────────
    
    /// Ошибка из крипто-подсистемы (подпись, шифрование канала)
    #[error("Крипто: {0}")]
    Crypto(#[from] root_crypto::CryptoError),

    // ─── Общие ───────────────────────────────────────────
    
    /// Динамическая ошибка с описанием (для редких случаев)
    #[error("Сетевая ошибка: {0}")]
    Other(String),
}

// 🔧 Методы-помощники для агрегатора / логирования
impl NetworkError {
    /// Код ошибки для метрик / API-ответов
    pub fn code(&self) -> &'static str {
        match self {
            NetworkError::ConnectionFailed(_) => "network.connect_failed",
            NetworkError::ConnectionLost { .. } => "network.connection_lost",
            NetworkError::Timeout { .. } => "network.timeout",
            NetworkError::Transport(_) => "network.transport",
            NetworkError::ProtocolViolation(_) => "network.protocol",
            NetworkError::InvalidMessage(_) => "network.invalid_msg",
            NetworkError::UnknownMessageType { .. } => "network.unknown_type",
            NetworkError::SendFailed(_) => "network.send_failed",
            NetworkError::ReceiveFailed(_) => "network.recv_failed",
            NetworkError::ChannelClosed { .. } => "network.channel_closed",
            NetworkError::PeerNotFound { .. } => "network.peer_not_found",
            NetworkError::DhtQueryFailed(_) => "network.dht_query",
            NetworkError::Crypto(e) => e.code(),  // делегируем в CryptoError
            NetworkError::Other(_) => "network.other",
        }
    }

    /// Можно ли повторить операцию при этой ошибке?
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            NetworkError::ConnectionFailed(_) |    // можно попробовать другой адрес
            NetworkError::Timeout { .. } |         // временная проблема сети
            NetworkError::Transport(_) |           // возможно, временный сбой
            NetworkError::SendFailed(_) |          // канал может освободиться
            NetworkError::ReceiveFailed(_) |
            NetworkError::PeerNotFound { .. } |    // пир может появиться позже
            NetworkError::DhtQueryFailed(_) |
            NetworkError::Other(_)
        )
    }
}
