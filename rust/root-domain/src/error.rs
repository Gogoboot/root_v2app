// root-domain/src/error.rs
// DomainError — единый тип ошибки для всего проекта.
// Остальные крейты конвертируют свои ошибки сюда через From.
// ============================================================

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Ошибка хранилища: {0}")]
    Storage(String),

    #[error("Ошибка криптографии: {0}")]
    Crypto(String),

    #[error("Ошибка сети: {0}")]
    Network(String),

    #[error("Ошибка валидации поля '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("{entity} с id '{id}' не найден")]
    NotFound { entity: &'static str, id: String },

    #[error("Неверное состояние приложения: {0}")]
    InvalidState(String),

    #[error("Доступ запрещён: {0}")]
    PermissionDenied(String),
}
