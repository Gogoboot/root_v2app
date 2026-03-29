// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — storage/panic.rs
// Panic Button — экстренная очистка ключей
// ═══════════════════════════════════════════════════════════

use rusqlite::Connection;
use zeroize::Zeroize;  // ← ОБЯЗАТЕЛЬНО добавить!
use crate::crypto::SecureKey;
use super::error::StorageError;

pub struct PanicButton;

impl PanicButton {
    /// Активировать Panic Button
    /// 
    /// # Действия
    /// 1. Обнулить ключ шифрования в памяти (zeroize)
    /// 2. Закрыть соединение с БД
    /// 3. Вернуть ошибку для обработки
    /// 
    /// # Аргументы
    /// * `key` — ключ шифрования (будет обнулён)
    /// * `db` — соединение с БД (будет закрыто)
    /// 
    /// # Возвращает
    /// * `StorageError::PanicButtonActivated` — для обработки в API
 pub fn activate(key: &mut SecureKey, db: &mut Option<Connection>) -> StorageError {
        // 🔐 Обнулить ключ в памяти
        key.zeroize();  // ← теперь работает
        
        // 🔐 Закрыть соединение с БД
        *db = None;
        
        StorageError::PanicButtonActivated
    }
}

