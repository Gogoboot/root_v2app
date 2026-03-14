// ============================================================
// ROOT v2.0 — storage/panic.rs
// PanicButton — мгновенное уничтожение ключа при принуждении
// ============================================================

use rusqlite::Connection;

use super::error::StorageError;
use super::key::StorageKey;

pub struct PanicButton;

impl PanicButton {
    /// Активировать Panic Button
    /// Уничтожает ключ → база данных становится нечитаемой навсегда
    pub fn activate(key: &mut StorageKey, db: &mut Option<Connection>) -> StorageError {
        println!("  🆘 PANIC BUTTON АКТИВИРОВАН");
        println!("  ⏱️  Уничтожение ключа...");

        // Шаг 1: Закрываем соединение с базой
        if let Some(conn) = db.take() {
            drop(conn);
        }

        // Шаг 2: Уничтожаем ключ (zeroize — перезапись нулями)
        key.destroy();

        println!("  ✅ Ключ уничтожен");
        println!("  🔒 База данных нечитаема навсегда");
        println!("  ℹ️  Данные на диске есть — расшифровать невозможно");

        StorageError::PanicButtonActivated
    }
}
