// ═══════════════════════════════════════════════════════════
// ROOT v2.0 — storage/database.rs
// Версия: 2.1 (Интеграция SaltManager #17)
// ═══════════════════════════════════════════════════════════
//Title: Убрать suppress dead_code для salt_manager после #37
//Description:
//Поле salt_manager временно имеет #[expect(dead_code)]. Нужно:
//1. Добавить использование после завершения задачи #37
//2. Удалить атрибут когда поле станет активно использоваться

use super::error::StorageError;
use super::merkle::MerkleTree;
use super::models::{Contact, Message};
use super::panic::PanicButton;
use crate::key::SaltManager; // <-- Импортируем новый менеджер
use root_crypto::{
    Salt, SecureKey, decrypt, derive_key, encrypt, pack_for_storage, unpack_from_storage,
};
use rusqlite::{Connection, params};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, Zeroizing};

pub struct Database {
    conn: Option<Connection>,
    key: SecureKey,
    // Соль теперь управляется внутри salt_manager, но мы можем оставить копию для отладки/логики,
    // если нужно. В данной реализации мы храним менеджер, чтобы он жил столько же, сколько БД.
    #[expect(dead_code)] // С пометкой "это временно"
    salt_manager: SaltManager,
    merkle: MerkleTree,
    db_path: String,
    panicked: bool,
}

impl Database {
    /// Открывает существующую базу данных и деривирует ключ шифрования.
    ///
    /// # Аргументы
    /// * `path` — путь к файлу SQLite базы данных.
    /// * `password` — мастер-пароль пользователя для деривации ключа (Argon2id).
    ///
    /// # Возвращает
    /// * `Ok(Database)` — если база успешно открыта и ключ деривирован.
    /// * `Err(StorageError)` — если файл не найден, пароль неверный или произошла ошибка БД.
    ///
    /// # ⚠️ Безопасность
    /// Эта функция **не затирает** переданный `password` после использования.
    /// Если пароль содержит чувствительные данные (например, мнемонику),
    /// вызывающий код обязан затирать его самостоятельно после вызова `open()`.
    ///
    /// ## Пример безопасного использования:
    /// ```rust
    /// use zeroize::Zeroizing;
    ///
    /// // 1. Оборачиваем пароль в защищённый буфер
    ///
    /// // 2. Открываем базу
    /// let password = Zeroizing::new(password_string);
    /// let db = Database::open("/path/to/db.sqlite", &password)?;
    ///
    /// // 3. Пароль автоматически затрётся при выходе из области видимости (drop)
    /// //    или можно затереть явно:
    /// // drop(password);
    ///
    ///

    /// Загружает Merkle tree из существующих сообщений в БД.
    /// Вызывается один раз при открытии базы.
    fn load_merkle_tree(conn: &Connection) -> Result<MerkleTree, StorageError> {
        let mut tree = MerkleTree::new();

        let mut stmt = conn
            .prepare("SELECT merkle_hash FROM messages ORDER BY id ASC")
            .map_err(StorageError::Database)?;

        let hashes: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .map_err(StorageError::Database)?;

        for hash_hex in hashes {
            let hash_bytes =
                hex::decode(&hash_hex).map_err(|e| StorageError::Crypto(e.to_string()))?;
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);
            tree.add_leaf(hash);
        }

        Ok(tree)
    }

    /// ```
    /// Открывает БД, деривирует ключ через Argon2id используя соль из Keychain (или мигрирует файл).
    pub fn open(path: &str, password:  &Zeroizing<String>) -> Result<Self, StorageError> {
        println!("  🔑 Инициализация SaltManager (Keychain/Migration)...");

        // Определяем директорию приложения для SaltManager.
        // Обычно это родительская директория от файла БД или специфичный путь.
        // Если БД лежит в ~/.local/share/root_app/db.sqlite, то dir будет ~/.local/share/root_app/
        let db_path_obj = std::path::PathBuf::from(path);
        let app_data_dir = db_path_obj
            .parent()
            .ok_or(StorageError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some("Invalid DB path".to_string()),
            )))?
            .to_path_buf();

        // 1. Создаем менеджер соли (выполняет чтение из Keychain или миграцию)
        let salt_manager =
            SaltManager::new(&app_data_dir).map_err(|e| StorageError::KeyError(e.to_string()))?;

        println!("  🔑 Генерация ключа через Argon2id...");
        let start = now_millis();

        // 2. Получаем соль для деривации
        let salt_bytes = salt_manager
            .get_salt()
            .map_err(|e| StorageError::KeyError(e.to_string()))?;

        // Копируем соль в Zeroizing буфер для безопасной передачи в derive_key
        let mut salt = Salt::default();
        salt.copy_from_slice(salt_bytes);

        // 3. Деривируем ключ
        let key = derive_key(password, &salt).map_err(|_| StorageError::KeyDerivationFailed)?;

        // Очищаем локальную копию соли сразу после деривации (она больше не нужна в стеке)
        salt.zeroize();

        let elapsed = now_millis() - start;
        println!("  ✅ Ключ готов за {}ms", elapsed);

        // 4. Открываем SQLite соединение
        // Примечание: Сама БД теперь содержит только зашифрованные данные, SQLCipher не нужен.
        let conn = Connection::open(path).map_err(StorageError::Database)?;

        let merkle = Self::load_merkle_tree(&conn)?;

        Ok(Database {
            conn: Some(conn),
            key,
            salt_manager, // Сохраняем менеджер в структуре
            merkle,
            db_path: path.to_string(),
            panicked: false,
        })
    }

    fn conn(&self) -> Result<&Connection, StorageError> {
        self.conn.as_ref().ok_or(StorageError::NotOpen)
    }

    pub fn initialize(&mut self) -> Result<(), StorageError> {
        // ✅ Проверка panic (консистентность)
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }

        let conn = self.conn.as_mut().ok_or(StorageError::NotOpen)?;
        let tx = conn.transaction().map_err(StorageError::Database)?;

        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_key TEXT NOT NULL,
                to_key TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                is_read INTEGER NOT NULL DEFAULT 0,
                merkle_hash TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS contacts (
                public_key TEXT PRIMARY KEY,
                nickname TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                reputation INTEGER NOT NULL DEFAULT 50
            );
            CREATE TABLE IF NOT EXISTS merkle_roots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                root_hash TEXT NOT NULL,
                msg_count INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS identity (
                id INTEGER PRIMARY KEY,
                public_key TEXT NOT NULL,
                mnemonic TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- ✅ Уникальный индекс на nickname
            CREATE UNIQUE INDEX IF NOT EXISTS idx_contacts_nickname ON contacts(nickname);

            CREATE INDEX IF NOT EXISTS idx_messages_to ON messages(to_key, timestamp);
            CREATE INDEX IF NOT EXISTS idx_messages_from ON messages(from_key, timestamp);",
        )
        .map_err(StorageError::Database)?;

        tx.commit().map_err(StorageError::Database)?;
        println!("  📋 Таблицы инициализированы");
        Ok(())
    }

    pub fn save_message(&mut self, mut msg: Message) -> Result<u64, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }

        let conn = self.conn.as_mut().ok_or(StorageError::NotOpen)?;
        let tx = conn.transaction().map_err(StorageError::Database)?;

        let hash = msg.hash();
        let hash_hex = hex::encode(hash);

        let plaintext = msg.content.clone().into_bytes();
        let encrypted =
            encrypt(&self.key, &plaintext).map_err(|_| StorageError::EncryptionFailed)?;
        let blob = pack_for_storage(&encrypted);
        let content_to_save = hex::encode(blob);

        // Шаг 1: Вставляем сообщение
        tx.execute(
            "INSERT INTO messages (from_key, to_key, content, timestamp, is_read, merkle_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                msg.from_key,
                msg.to_key,
                content_to_save,
                msg.timestamp,
                msg.is_read as i32,
                hash_hex
            ],
        )
        .map_err(StorageError::Database)?;

        let id = tx.last_insert_rowid() as u64;
        msg.id = Some(id);

        // ✅ Шаг 2: СНАЧАЛА добавляем лист в дерево
        self.merkle.add_leaf(hash);

        // ✅ Шаг 3: Теперь root и len соответствуют друг другу
        let root = self
            .merkle
            .root()
            .map(hex::encode)
            .ok_or(StorageError::Crypto("Merkle tree empty".to_string()))?;

        tx.execute(
            "INSERT INTO merkle_roots (root_hash, msg_count, timestamp) VALUES (?1, ?2, ?3)",
            params![root, self.merkle.len() as i64, now_secs()? as i64], // ← Без +1
        )
        .map_err(StorageError::Database)?;

        // ✅ Шаг 4: Коммит
        tx.commit().map_err(StorageError::Database)?;

        // ✅ Коммит успешен — данные сохранены в БД
        //    self.merkle уже обновлён (до коммита), всё синхронизировано
        Ok(id)
    }

    pub fn get_messages(&self, public_key: &str) -> Result<Vec<Message>, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }

        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, from_key, to_key, content, timestamp, is_read
             FROM messages WHERE to_key = ?1 OR from_key = ?1 ORDER BY timestamp ASC",
            )
            .map_err(StorageError::Database)?;

        let messages = stmt
            .query_map(params![public_key], |row| {
                let id: i64 = row.get(0)?;
                let from_key: String = row.get(1)?;
                let to_key: String = row.get(2)?;
                let content_encrypted: String = row.get(3)?;
                let timestamp: i64 = row.get(4)?;
                let is_read: i32 = row.get(5)?;

                let blob = hex::decode(&content_encrypted).map_err(|_| {
                    rusqlite::Error::InvalidColumnName("hex decode failed".to_string())
                })?;
                let encrypted = unpack_from_storage(&blob)
                    .map_err(|_| rusqlite::Error::InvalidColumnName("unpack failed".to_string()))?;
                let plaintext = decrypt(&self.key, &encrypted).map_err(|_| {
                    rusqlite::Error::InvalidColumnName("decrypt failed".to_string())
                })?;
                let content = String::from_utf8(plaintext)
                    .map_err(|_| rusqlite::Error::InvalidColumnName("utf8 failed".to_string()))?;

                Ok(Message {
                    id: Some(id as u64),
                    from_key,
                    to_key,
                    content,
                    timestamp: timestamp as u64,
                    is_read: is_read != 0,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .map_err(StorageError::Database)?;

        Ok(messages)
    }

    pub fn mark_read(&self, msg_id: u64) -> Result<(), StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }
        let updated = self
            .conn()?
            .execute(
                "UPDATE messages SET is_read = 1 WHERE id = ?1",
                params![msg_id as i64],
            )
            .map_err(StorageError::Database)?;
        if updated == 0 {
            return Err(StorageError::MessageNotFound(msg_id));
        }
        Ok(())
    }

    pub fn unread_count(&self, public_key: &str) -> Result<u64, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }
        let count: i64 = self
            .conn()?
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE to_key = ?1 AND is_read = 0",
                params![public_key],
                |row| row.get(0),
            )
            .map_err(StorageError::Database)?;
        Ok(count as u64)
    }

    pub fn save_identity(&self, public_key: &str, mnemonic: &str) -> Result<(), StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }

        // Шифруем мнемонику перед записью в БД
        let encrypted =
            encrypt(&self.key, mnemonic.as_bytes()).map_err(|_| StorageError::EncryptionFailed)?;
        let blob = pack_for_storage(&encrypted);
        let mnemonic_hex = hex::encode(blob);

        self.conn()?.execute(
        "INSERT OR REPLACE INTO identity (id, public_key, mnemonic, created_at) VALUES (1, ?1, ?2, ?3)",
        params![public_key, mnemonic_hex, now_secs()?],
    ).map_err(StorageError::Database)?;

        Ok(())
    }

    pub fn load_identity(&self) -> Result<Option<(String, String)>, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }

        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT public_key, mnemonic FROM identity WHERE id = 1")
            .map_err(StorageError::Database)?;
        let mut rows = stmt.query([]).map_err(StorageError::Database)?;

        if let Some(row) = rows.next().map_err(StorageError::Database)? {
            let public_key: String = row.get(0).map_err(StorageError::Database)?;
            let mnemonic_hex: String = row.get(1).map_err(StorageError::Database)?;

            // Расшифровываем мнемонику при чтении
            let blob =
                hex::decode(&mnemonic_hex).map_err(|e| StorageError::Crypto(e.to_string()))?;
            let encrypted =
                unpack_from_storage(&blob).map_err(|e| StorageError::Crypto(e.to_string()))?;
            let plaintext =
                decrypt(&self.key, &encrypted).map_err(|_| StorageError::EncryptionFailed)?;
            let mnemonic =
                String::from_utf8(plaintext).map_err(|e| StorageError::Crypto(e.to_string()))?;

            Ok(Some((public_key, mnemonic)))
        } else {
            Ok(None)
        }
    }

    pub fn add_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }
        let count: i64 = self
            .conn()?
            .query_row(
                "SELECT COUNT(*) FROM contacts WHERE nickname = ?1 AND public_key != ?2",
                params![contact.nickname, contact.public_key],
                |row| row.get(0),
            )
            .map_err(StorageError::Database)?;
        if count > 0 {
            println!("  ⚠️  Имя '{}' уже занято", contact.nickname);
        }
        self.conn()?.execute(
            "INSERT OR REPLACE INTO contacts (public_key, nickname, added_at, reputation) VALUES (?1, ?2, ?3, ?4)",
            params![contact.public_key, contact.nickname, contact.added_at as i64, contact.reputation as i32],
        ).map_err(StorageError::Database)?;
        Ok(())
    }

    pub fn get_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT public_key, nickname, added_at, reputation FROM contacts")
            .map_err(StorageError::Database)?;
        let contacts = stmt
            .query_map([], |row| {
                Ok(Contact {
                    public_key: row.get(0)?,
                    nickname: row.get(1)?,
                    added_at: row.get::<_, i64>(2)? as u64,
                    reputation: row.get::<_, i32>(3)? as u8,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .map_err(StorageError::Database)?;
        Ok(contacts)
    }

    pub fn verify_integrity(&self) -> Result<bool, StorageError> {
        if self.panicked {
            return Err(StorageError::PanicButtonActivated);
        }
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT merkle_hash FROM messages ORDER BY id ASC")
            .map_err(StorageError::Database)?;
        let db_hashes: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .map_err(StorageError::Database)?;

        let mut verify_tree = MerkleTree::new();
        for hash_hex in &db_hashes {
            let hash_bytes =
                hex::decode(hash_hex).map_err(|e| StorageError::Crypto(e.to_string()))?;
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);
            verify_tree.add_leaf(hash);
        }
        Ok(verify_tree.root() == self.merkle.root())
    }

    pub fn panic_destroy(&mut self) -> Result<(), StorageError> {
        self.panicked = true;
        PanicButton::activate(&mut self.key, &mut self.conn, &self.db_path)?;
        Ok(())
    }

    pub fn print_stats(&self) {
        println!("\n  ╔══════════════════════════════════════════════╗");
        println!("  ║         СТАТИСТИКА STORAGE ROOT v2.0         ║");
        println!("  ╠══════════════════════════════════════════════╣");
        println!("  ║ Файл БД:        {:>26}  ║", self.db_path);
        println!(
            "  ║ Шифрование:     {:>26}  ║",
            "ChaCha20-Poly1305 (app-level)"
        );
        println!("  ║ KDF:            {:>26}  ║", "Argon2id (19MB, 2 iter)");
        println!(
            "  ║ Соль:           {:>26}  ║",
            "OS Keychain/Keystore (#17 DONE)"
        );
        println!("  ║ Merkle листьев: {:>26}  ║", self.merkle.len());
        println!(
            "  ║ Panic Button:   {:>26}  ║",
            if self.panicked {
                "🆘 АКТИВИРОВАН"
            } else {
                "✅ Не активирован"
            }
        );
        println!("  ╚══════════════════════════════════════════════╝\n");
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Обнуляем ключ
        self.key.zeroize();

        // SaltManager при дропе автоматически очистит внутреннюю соль,
        // так как она хранится в Zeroizing<[u8; 32]> внутри него.
        // Явное действие не требуется, но можно добавить логирование.
    }
}

fn now_secs() -> Result<u64, StorageError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| {
            StorageError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!("System time error: {}", e)),
            ))
        })
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_millis() as u64
}
