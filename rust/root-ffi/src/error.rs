// root-ffi/src/error.rs
use thiserror::Error;
use root_crypto::CryptoError;
use root_storage::StorageError;
use root_identity::IdentityError;
use root_network::NetworkError;
use root_economy::EconomyError;

/// Единая ошибка FFI-границы.
/// Собирает ошибки всех нижних крейтов и добавляет контекст API-слоя.
#[derive(Error, Debug)]
pub enum FfiError {
    // ─── Ошибки нижних слоёв (автоматическая конвертация) ──
    #[error("Storage: {0}")]
    Storage(#[from] StorageError),

    #[error("Crypto: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Identity: {0}")]
    Identity(#[from] IdentityError),

    #[error("Network: {0}")]
    Network(#[from] NetworkError),

    #[error("Economy: {0}")]
    Economy(#[from] EconomyError),

    // ─── Ошибки, специфичные для FFI/API ──────────────────
    #[error("Неверные входные данные: {0}")]
    Validation(String),

    #[error("Сервис не инициализирован")]
    NotInitialized,

    #[error("Внутренняя ошибка FFI: {0}")]
    Internal(String),

    #[error("Мьютекс состояния повреждён (poisoned)")]
    LockPoisoned,
}

// 🔧 Методы для внешнего мира (мобильные приложения, CLI, логи)
impl FfiError {
    /// Стабильный код ошибки для фронтенда / метрик
    pub fn code(&self) -> &'static str {
        match self {
            FfiError::Storage(e) => e.code(),
            FfiError::Crypto(e) => e.code(),
            FfiError::Identity(e) => e.code(),
            FfiError::Network(e) => e.code(),
            FfiError::Economy(e) => e.code(),
            FfiError::Validation(_) => "validation",
            FfiError::NotInitialized => "not_initialized",
            FfiError::Internal(_) => "internal",
            FfiError::LockPoisoned => "lock_poisoned",
        }
    }

    /// HTTP-статус или код выхода для CLI
    pub fn status_code(&self) -> u16 {
        match self {
            FfiError::Storage(StorageError::MessageNotFound(_)) |
            FfiError::Identity(IdentityError::PrivateKeyNotFound) |
            FfiError::Economy(EconomyError::AccountNotFound { .. }) => 404,
            FfiError::Validation(_) |
            FfiError::Economy(EconomyError::InvalidAmount) |
            FfiError::Economy(EconomyError::InvalidSignature) => 400,
            FfiError::Identity(IdentityError::InvalidMnemonic) |
            FfiError::Economy(EconomyError::InsufficientFunds { .. }) |
            FfiError::Economy(EconomyError::InsufficientReputation { .. }) => 403,
            FfiError::Network(NetworkError::Timeout { .. }) |
            FfiError::Network(NetworkError::ConnectionFailed(_)) => 408,
            FfiError::NotInitialized | FfiError::LockPoisoned => 503,
            _ => 500,
        }
    }

    /// JSON-ответ для мобильных приложений / Web
    pub fn to_json(&self) -> String {
        use serde_json::json;
        json!({
            "error": self.code(),
            "message": self.to_string(),
            "status": self.status_code(),
            "details": match self {
                FfiError::Economy(EconomyError::InsufficientReputation { required, available }) => {
                    Some(json!({ "required": required, "available": available }))
                }
                FfiError::Economy(EconomyError::InsufficientFunds { required, available }) => {
                    Some(json!({ "required": required, "available": available }))
                }
                FfiError::Network(NetworkError::Timeout { timeout_ms }) => {
                    Some(json!({ "timeout_ms": timeout_ms }))
                }
                _ => None
            }
        })
        .to_string()
    }
}
