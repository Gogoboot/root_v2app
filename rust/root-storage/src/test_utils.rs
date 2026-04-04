//! Вспомогательные структуры для тестирования хранилища.
//!
//! Этот модуль предназначен **только для тестов** — он не используется
//! в production-сборке. Содержит реализацию [`StoragePort`] и
//! [`AccountStoragePort`] поверх обычных `Vec` в памяти, без SQLite
//! и без файловой системы.
//!
//! # Зачем это нужно
//!
//! Реальная [`Database`] требует файл на диске. В тестах это неудобно:
//! нужно создавать временные файлы, чистить их после теста и т.д.
//! [`InMemoryStorage`] решает проблему — данные живут только пока
//! существует переменная, и удаляются автоматически.

// Стандартный примитив синхронизации.
// Mutex гарантирует что только один поток одновременно читает или пишет данные.
use std::sync::Mutex;

// Макрос async_trait позволяет использовать async fn внутри трейтов.
// В Rust трейты не поддерживают async напрямую — этот крейт добавляет такую возможность.
use async_trait::async_trait;

use crate::{
    error::StorageError,
    ports::{AccountStoragePort, StoragePort},
    models::{Contact, StoredAccount, StoredMessage},
};

// ─── InMemoryStorage ────────────────────────────────────────────────────────

/// Реализация хранилища в оперативной памяти для использования в тестах.
///
/// Хранит сообщения, контакты и аккаунт в обычных `Vec`/`Option`,
/// завёрнутых в [`Mutex`] для безопасного доступа из нескольких потоков.
///
/// # Пример
///
/// ```rust
/// let storage = InMemoryStorage::new();
/// storage.save_message(&msg).await?;
/// ```
///
/// # Важно
///
/// Не использовать в production. Данные не сохраняются между запусками.
pub struct InMemoryStorage {
    /// Список всех сохранённых сообщений.
    /// Mutex нужен потому что async-тесты могут выполняться из разных потоков.
    messages: Mutex<Vec<StoredMessage>>,

    /// Список контактов. При повторном сохранении с тем же `peer_id`
    /// контакт обновляется, а не дублируется.
    contacts: Mutex<Vec<Contact>>,

    /// Текущий аккаунт. `None` означает что аккаунт ещё не создан.
    account: Mutex<Option<StoredAccount>>,
}

impl InMemoryStorage {
    /// Создаёт новое пустое хранилище.
    ///
    /// Все коллекции пусты, аккаунт отсутствует.
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
            contacts: Mutex::new(Vec::new()),
            account: Mutex::new(None),
        }
    }
}

/// Позволяет создавать `InMemoryStorage` через `InMemoryStorage::default()`.
/// Делегирует вызов в [`InMemoryStorage::new`].
impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

// ─── StoragePort ────────────────────────────────────────────────────────────

/// Реализация трейта [`StoragePort`] для тестового хранилища в памяти.
///
/// Методы имитируют поведение реальной БД, но работают с `Vec` внутри `Mutex`.
#[async_trait]
impl StoragePort for InMemoryStorage {
    /// Сохраняет сообщение в список.
    ///
    /// Сообщение добавляется в конец — порядок вставки сохраняется.
    /// Дубликаты **не** проверяются (в отличие от контактов).
    async fn save_message(&self, msg: &StoredMessage) -> Result<(), StorageError> {
        self.messages
            .lock()
            // map_err превращает ошибку блокировки мьютекса в нашу StorageError.
            // "mutex poisoned" означает что другой поток упал внутри lock() —
            // в тестах это почти невозможно, но обработать нужно.
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?
            .push(msg.clone());
        Ok(())
    }

    /// Возвращает сообщения по теме (`topic`) с поддержкой пагинации.
    ///
    /// # Параметры
    ///
    /// - `topic` — фильтр: возвращаются только сообщения с этой темой
    /// - `limit` — максимальное количество результатов
    /// - `offset` — сколько сообщений пропустить с начала (для постраничной загрузки)
    ///
    /// # Пример пагинации
    ///
    /// ```text
    /// Все сообщения: [A, B, C, D, E]
    /// get_messages(topic, limit=2, offset=0) → [A, B]
    /// get_messages(topic, limit=2, offset=2) → [C, D]
    /// ```
    async fn get_messages(
        &self,
        topic: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>, StorageError> {
        let guard = self
            .messages
            .lock()
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?;

        let result = guard
            .iter()
            .filter(|m| m.topic == topic)  // оставляем только нужную тему
            .skip(offset as usize)          // пропускаем первые `offset` элементов
            .take(limit as usize)           // берём не более `limit` элементов
            .cloned()                       // клонируем, т.к. guard владеет данными
            .collect();

        Ok(result)
    }

    /// Сохраняет контакт. Если контакт с таким `peer_id` уже существует —
    /// обновляет его. Если нет — добавляет новый.
    ///
    /// Это поведение называется **upsert** (update + insert).
    async fn save_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let mut guard = self
            .contacts
            .lock()
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?;

        // Ищем контакт с таким же peer_id среди уже сохранённых.
        // iter_mut() нужен чтобы получить изменяемую ссылку для обновления.
        if let Some(existing) = guard.iter_mut().find(|c| c.peer_id == contact.peer_id) {
            // Контакт найден — перезаписываем данные.
            // Разыменование (*existing) нужно чтобы заменить значение целиком,
            // а не ссылку на него.
            *existing = contact.clone();
        } else {
            // Контакт новый — просто добавляем в конец списка.
            guard.push(contact.clone());
        }

        Ok(())
    }

    /// Возвращает список всех сохранённых контактов.
    ///
    /// Возвращает клон вектора, чтобы вызывающий код мог работать
    /// с данными после того как мьютекс освободится.
    async fn get_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let guard = self
            .contacts
            .lock()
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?;
        Ok(guard.clone())
    }
}

// ─── AccountStoragePort ─────────────────────────────────────────────────────

/// Реализация трейта [`AccountStoragePort`] для тестового хранилища.
///
/// Аккаунт хранится как `Option<StoredAccount>`:
/// - `None` — аккаунт не создан
/// - `Some(...)` — аккаунт существует
#[async_trait]
impl AccountStoragePort for InMemoryStorage {
    /// Сохраняет аккаунт, заменяя предыдущий если он был.
    ///
    /// В отличие от контактов, аккаунт один — нет смысла хранить список.
    async fn save_account(&self, account: &StoredAccount) -> Result<(), StorageError> {
        let mut guard = self
            .account
            .lock()
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?;

        // Заменяем Option целиком. Some() оборачивает аккаунт в Option.
        *guard = Some(account.clone());
        Ok(())
    }

    /// Загружает аккаунт если он был сохранён ранее.
    ///
    /// Возвращает `Ok(None)` если аккаунт ещё не создан — это
    /// нормальная ситуация, не ошибка.
    async fn load_account(&self) -> Result<Option<StoredAccount>, StorageError> {
        let guard = self
            .account
            .lock()
            .map_err(|_| StorageError::Internal("mutex poisoned".into()))?;

        // clone() нужен чтобы вернуть значение из-под мьютекса.
        // Нельзя вернуть ссылку — guard освобождается в конце функции.
        Ok(guard.clone())
    }
}

// ─── Тесты ──────────────────────────────────────────────────────────────────

// cfg(test) означает: этот блок компилируется ТОЛЬКО при запуске `cargo test`.
// В release-сборку он не попадает — экономит размер бинарника.
#[cfg(test)]
mod tests {
    // super::* импортирует всё из родительского модуля (test_utils).
    // Так тесты видят InMemoryStorage и все трейты.
    use super::*;

    /// Проверяет что сохранённое сообщение можно прочитать обратно.
    ///
    /// Ключевое: тест проходит **без создания файлов на диске**.
    #[tokio::test]
    async fn test_save_and_get_messages_without_filesystem() {
        let storage = InMemoryStorage::new();

        let msg = StoredMessage {
            id: "abc123".into(),
            topic: "topic/test".into(),
            sender_peer_id: "peer1".into(),
            content: b"hello".to_vec(),
            timestamp: 1_000_000,
        };

        storage.save_message(&msg).await.unwrap();

        let result = storage.get_messages("topic/test", 10, 0).await.unwrap();

        // Убеждаемся что вернулось ровно одно сообщение.
        assert_eq!(result.len(), 1);
        // Убеждаемся что id совпадает с тем что мы сохранили.
        assert_eq!(result[0].id, "abc123");
    }

    /// Проверяет что повторное сохранение контакта с тем же `peer_id`
    /// не создаёт дубликат, а обновляет существующую запись.
    #[tokio::test]
    async fn test_save_contact_deduplication() {
        let storage = InMemoryStorage::new();

        let contact = Contact {
            peer_id: "peer1".into(),
            nickname: b"Alice".to_vec(),
            public_key: vec![1, 2, 3],
        };

        storage.save_contact(&contact).await.unwrap();
        storage.save_contact(&contact).await.unwrap(); // второй раз — обновление, не дубль

        let contacts = storage.get_contacts().await.unwrap();

        // Должен быть ровно один контакт, а не два.
        assert_eq!(contacts.len(), 1);
    }

    /// Проверяет полный цикл работы с аккаунтом:
    /// сначала его нет → сохраняем → он появляется.
    #[tokio::test]
    async fn test_account_round_trip() {
        let storage = InMemoryStorage::new();

        // В начале аккаунта нет — load_account должен вернуть None.
        assert!(storage.load_account().await.unwrap().is_none());

        let acc = StoredAccount {
            id: "acc1".into(),
            // остальные поля по твоей структуре StoredAccount
        };

        storage.save_account(&acc).await.unwrap();

        let loaded = storage.load_account().await.unwrap();

        // После сохранения аккаунт должен существовать.
        assert!(loaded.is_some());
    }
}
