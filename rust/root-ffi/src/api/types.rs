//! DTO структуры и типы ошибок для FFI границы.
//!
//! Этот модуль — переводчик между Rust-миром и Flutter/Tauri.
//!
//! # Два типа ошибок в проекте
//!
//! - [`ApiError`] — ошибки бизнес-логики API слоя
//! - [`crate::error::FfiError`] — технические ошибки инфраструктуры
//!
//! TODO: объединить в один тип в отдельном спринте рефакторинга FFI.

use thiserror::Error;
use zeroize::Zeroizing;

// ─── ApiError ────────────────────────────────────────────────────────────────

/// Ошибки бизнес-логики API слоя.
///
/// Каждый вариант соответствует конкретной ситуации —
/// Flutter использует их чтобы показать правильное сообщение
/// или перенаправить пользователя на нужный экран.
#[derive(Error, Debug)]
pub enum ApiError {
    // ─── Состояние приложения ─────────────────────────────────────────────

    /// Identity не была создана или восстановлена.
    ///
    /// Flutter должен перенаправить на экран создания аккаунта.
    #[error("Identity не инициализирована. Вызови generate_identity() или restore_identity()")]
    IdentityNotInitialized,

    /// БД не открыта — нужно вызвать `unlock_database()` с паролем.
    #[error("База данных не открыта. Вызови unlock_database(password)")]
    DatabaseNotOpen,

    /// Экономический слой не инициализирован.
    #[error("Ledger не инициализирован")]
    LedgerNotInitialized,

    /// Panic Button был активирован — все данные уничтожены.
    ///
    /// После этой ошибки приложение должно завершить работу.
    #[error("Panic Button активирован — перезапусти приложение")]
    PanicActivated,

    /// Операция невозможна в текущем состоянии приложения.
    #[error("Неверное состояние приложения: {0}")]
    InvalidState(String),

    // ─── Ошибки слоёв ────────────────────────────────────────────────────

    /// Ошибка при работе с Identity (генерация, восстановление, подпись).
    #[error("Ошибка Identity: {0}")]
    IdentityError(String),

    /// Ошибка экономического слоя (транзакции, баланс, vesting).
    #[error("Ошибка экономики: {0}")]
    EconomyError(String),

    /// Ошибка хранилища (SQLite, сериализация, Merkle).
    #[error("Ошибка хранилища: {0}")]
    StorageError(String),

    // ─── Входные данные ───────────────────────────────────────────────────

    /// Некорректные входные данные от пользователя или Flutter.
    #[error("Неверные параметры: {0}")]
    InvalidInput(String),

    // ─── Внутренние ───────────────────────────────────────────────────────

    /// Внутренняя ошибка инфраструктуры.
    ///
    /// Например: отравленный мьютекс (`mutex poisoned`).
    /// Пользователь должен перезапустить приложение.
    #[error("Внутренняя ошибка: {0}")]
    InternalError(String),
}

// ─── Маппинг ошибок инфраструктуры → ApiError ────────────────────────────────

use root_storage::StorageError;
use root_identity::IdentityError;

/// Конвертация ошибок хранилища.
///
/// Позволяет писать `.map_err(ApiError::from)?`
/// вместо `.map_err(|e| ApiError::StorageError(e.to_string()))?`.
impl From<StorageError> for ApiError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::NotOpen              => ApiError::DatabaseNotOpen,
            StorageError::PanicButtonActivated => ApiError::PanicActivated,
            other                              => ApiError::StorageError(other.to_string()),
        }
    }
}

/// Конвертация ошибок identity.
impl From<IdentityError> for ApiError {
    fn from(e: IdentityError) -> Self {
        ApiError::IdentityError(e.to_string())
    }
}

// ─── DTO структуры ───────────────────────────────────────────────────────────

/// Информация об идентичности для передачи во Flutter/Tauri.
///
/// # Безопасность
///
/// Поле `mnemonic` содержит чувствительные данные.
/// [`Zeroizing`] гарантирует что байты мнемоники
/// будут обнулены в памяти когда структура освобождается.
///
/// Flutter получает мнемонику **один раз** при генерации —
/// после этого она нигде не хранится в открытом виде.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdentityInfo {
    /// Публичный ключ Ed25519 в hex-формате (64 символа).
    pub public_key: String,

    /// 24 слова мнемоники — только при генерации нового аккаунта.
    ///
    /// `None` при восстановлении из существующей мнемоники.
    /// Автоматически обнуляется при освобождении памяти.
    #[serde(serialize_with = "serialize_zeroizing_option")]
    pub mnemonic: Option<Zeroizing<String>>,

    /// Идентификатор сети: `"root-mainnet-v2"`.
    pub network: String,
}

/// Сериализует `Option<Zeroizing<String>>` в JSON.
///
/// Нужен потому что `serde` не умеет сериализовать [`Zeroizing`]
/// автоматически — требуется явный `serialize_with`.
fn serialize_zeroizing_option<S>(
    value: &Option<Zeroizing<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(s) => serializer.serialize_some(s.as_str()),
        None    => serializer.serialize_none(),
    }
}

/// Баланс пользователя для экрана кошелька.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BalanceInfo {
    /// Публичный ключ владельца
    pub public_key: String,
    /// Баланс в SAP токенах
    pub balance_sap: f64,
    /// Баланс в дропах (минимальная единица)
    pub balance_drops: u64,
    /// Застейканные SAP
    pub staked_sap: f64,
    /// Репутация узла (0..=255)
    pub reputation: u8,
    /// Заблокирован ли аккаунт
    pub is_banned: bool,
    /// Доступные для получения SAP из vesting
    pub vesting_available_sap: f64,
    /// Заблокированные в vesting SAP
    pub vesting_locked_sap: f64,
}

/// Сообщение для отображения в UI Flutter.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageInfo {
    /// Внутренний id SQLite
    pub id: u64,
    /// Публичный ключ отправителя
    pub from_key: String,
    /// Публичный ключ получателя
    pub to_key: String,
    /// Содержимое сообщения
    pub content: String,
    /// Unix timestamp в секундах
    pub timestamp: u64,
    /// Прочитано ли сообщение
    pub is_read: bool,
    /// Имя контакта если есть в адресной книге
    pub from_name: Option<String>,
}

/// Статус узла для экрана настроек.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeStatus {
    pub public_key: String,
    /// Активен ли узел в P2P сети
    pub is_active: bool,
    pub reputation: u8,
    pub staked_sap: f64,
    pub offense_count: u8,
    pub genesis_claimed: bool,
    /// Количество транзакций
    pub tx_count: usize,
    /// Количество подключённых пиров
    pub peer_count: u32,
    pub network: String,
    pub version: String,
}

/// Информация о vesting для экрана кошелька.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VestingInfo {
    pub total_sap: f64,
    pub available_sap: f64,
    pub locked_sap: f64,
    /// Процент разблокированных токенов (0.0..=100.0)
    pub percent_unlocked: f64,
    /// Полностью ли разблокирован vesting
    pub fully_unlocked: bool,
    /// Дней до полного разблокирования
    pub days_until_full: u64,
}

/// Результат транзакции.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TxResult {
    pub tx_id: String,
    pub amount_sap: f64,
    /// Комиссия сети
    pub fee_sap: f64,
    /// Сожжённые токены (дефляционный механизм)
    pub burned_sap: f64,
    pub timestamp: u64,
    pub success: bool,
}

/// Предупреждение о небезопасных P2P методах.
///
/// Показывается пользователю перед включением P2P режима.
#[derive(Debug, Clone, serde::Serialize)]
pub struct P2pWarning {
    /// Показывать ли предупреждение
    pub show_warning: bool,
    /// Методы безопасные для P2P
    pub safe_methods: Vec<String>,
    /// Методы раскрывающие IP адрес
    pub unsafe_methods: Vec<String>,
    /// Текст предупреждения для пользователя
    pub message: String,
}
