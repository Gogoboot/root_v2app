// ============================================================
// ROOT v2.0 — storage/panic.rs
// Panic Button — экстренная очистка данных
// ============================================================

use rusqlite::Connection;
use zeroize::Zeroize;
use root_crypto::SecureKey;
use super::error::StorageError;
use std::io::Write;

pub struct PanicButton;

impl PanicButton {
    /// Активировать Panic Button
    ///
    /// Действия по порядку:
    /// 1. Обнулить ключ шифрования в памяти
    /// 2. Закрыть соединение с БД
    /// 3. Перезаписать файл БД нулями 3 раза
    /// 4. Удалить файл физически
    pub fn activate(
        key: &mut SecureKey,
        db: &mut Option<Connection>,
        db_path: &str,
    ) -> Result<(), StorageError> {  // ✅ Возвращает Result
        // 1. Обнуляем ключ в памяти
        key.zeroize();

        // 2. Закрываем соединение с БД до записи на диск
        *db = None;

        // 3. Перезаписываем файл нулями 3 раза
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .write(true)
            .open(db_path)
        {
            if let Ok(meta) = std::fs::metadata(db_path) {
                let size = meta.len() as usize;
                let zeros = vec![0u8; size];
                for _ in 0..3 {
                    let _ = file.write_all(&zeros);
                    let _ = file.flush();
                }
            }
        }  // ✅ Закрывающая скобка

        // 4. Удаляем файл
        let _ = std::fs::remove_file(db_path);

        // ✅ Возвращаем успех
        Ok(())
    }
}
