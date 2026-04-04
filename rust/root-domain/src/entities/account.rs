//! Доменная сущность аккаунта пользователя.
//!
//! Аккаунт — это идентификация пользователя в сети RooT.
//! Основан на BIP-39 мнемонике из которой derive-ятся Ed25519 ключи.
//!
//! # Жизненный цикл
//!
//! ```text
//! root-crypto: генерирует BIP-39 мнемонику
//!      ↓
//! root-crypto: шифрует мнемонику паролем пользователя → Vec<u8>
//!      ↓
//! root-domain: Account::new(public_key, encrypted_mnemonic, ...)
//!      ↓
//! root-storage: сохраняет в SQLite
//! ```

use serde::{Deserialize, Serialize};

/// Уникальный идентификатор аккаунта.
///
/// Паттерн **newtype** над публичным ключом Ed25519 в hex-формате.
/// Ключ неизменен — поэтому он и является идентификатором.
///
/// `Hash` + `Eq` позволяют использовать как ключ в `HashMap`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl AccountId {
    /// Создаёт `AccountId` из публичного ключа Ed25519 в hex-формате.
    pub fn new(public_key: String) -> Self {
        Self(public_key)
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Account ─────────────────────────────────────────────────────────────────

/// Доменная сущность аккаунта пользователя RooT Messenger.
///
/// Создаётся только через [`Account::new`] или [`Account::new_mock`].
/// Поля приватные — изменение только через методы.
///
/// # Идентификация в сети
///
/// `public_key` — это Ed25519 ключ derived из BIP-39 мнемоники.
/// Он же используется как `peer_id` в libp2p.
///
/// # Хранение мнемоники
///
/// Мнемоника хранится **зашифрованной** (`encrypted_mnemonic`).
/// Расшифровка только в `root-crypto` при вводе пароля пользователем.
/// Domain никогда не видит мнемонику в открытом виде.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Публичный ключ Ed25519 в hex-формате.
    ///
    /// Основной идентификатор в P2P сети.
    /// Derive-ится из BIP-39 мнемоники в `root-crypto`.
    public_key: String,

    /// Зашифрованная BIP-39 мнемоника (24 слова).
    ///
    /// Шифруется паролем пользователя в `root-crypto`
    /// перед передачей сюда. Domain получает уже зашифрованные байты.
    ///
    /// TODO: в MVP заполняется пустым вектором до подключения
    /// root-crypto генерации мнемоники (Sprint Bootstrap relay).
    encrypted_mnemonic: Vec<u8>,

    /// Время создания аккаунта — Unix timestamp в миллисекундах.
    created_at: u64,
}

impl Account {
    /// Создаёт аккаунт с настоящей зашифрованной мнемоникой.
    ///
    /// Вызывается из `root-crypto` после генерации BIP-39 мнемоники
    /// и её шифрования паролем пользователя.
    ///
    /// # Пример
    ///
    /// ```rust
    /// // В root-crypto:
    /// let mnemonic = bip39::generate();
    /// let encrypted = encrypt(mnemonic, user_password);
    /// let public_key = derive_ed25519(mnemonic).public_key_hex();
    ///
    /// // В root-domain:
    /// let account = Account::new(public_key, encrypted, timestamp);
    /// ```
    pub fn new(
        public_key: String,
        encrypted_mnemonic: Vec<u8>,
        created_at: u64,
    ) -> Self {
        Self {
            public_key,
            encrypted_mnemonic,
            created_at,
        }
    }

    /// Создаёт аккаунт-заглушку для MVP тестирования.
    ///
    /// `encrypted_mnemonic` — пустой вектор. P2P идентификация
    /// с таким аккаунтом не работает.
    ///
    /// TODO: удалить когда root-crypto генерация мнемоники будет готова.
    #[cfg(debug_assertions)]
    pub fn new_mock(public_key: String, created_at: u64) -> Self {
        Self {
            public_key,
            encrypted_mnemonic: Vec::new(), // заглушка
            created_at,
        }
    }

    /// Возвращает `AccountId` для этого аккаунта.
    ///
    /// Строится из публичного ключа — они всегда совпадают.
    pub fn id(&self) -> AccountId {
        AccountId::new(self.public_key.clone())
    }

    // ─── Геттеры ─────────────────────────────────────────────────────────

    /// Возвращает публичный ключ Ed25519 в hex-формате.
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Возвращает зашифрованную мнемонику в виде байт.
    ///
    /// Расшифровка только в `root-crypto` — domain не знает пароль.
    pub fn encrypted_mnemonic(&self) -> &[u8] {
        &self.encrypted_mnemonic
    }

    /// Возвращает время создания аккаунта (Unix ms).
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    // ─── Проверки состояния ──────────────────────────────────────────────

    /// Проверяет что мнемоника присутствует (не заглушка MVP).
    ///
    /// Используй перед операциями требующими P2P идентификации.
    ///
    /// ```rust
    /// if !account.has_mnemonic() {
    ///     return Err(DomainError::InvalidState(
    ///         "аккаунт создан без мнемоники — P2P недоступен".into()
    ///     ));
    /// }
    /// ```
    pub fn has_mnemonic(&self) -> bool {
        !self.encrypted_mnemonic.is_empty()
    }
}
