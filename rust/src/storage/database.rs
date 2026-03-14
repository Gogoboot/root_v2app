// ============================================================
// ROOT v2.0 — storage/database.rs
// Database — главный движок SQLite
// ============================================================

use rusqlite::{Connection, params};
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::StorageError;
use super::key::StorageKey;
use super::merkle::MerkleTree;
use super::models::{Contact, Message};
use super::panic::PanicButton;

pub struct Database {
    conn: Option<Connection>,
    key: StorageKey,
    pub merkle: MerkleTree,
    db_path: String,
    pub panicked: bool,
}

impl Database {
    /// Открыть или создать зашифрованную базу данных
    pub fn open(path: &str, password: &str) -> Result<Self, StorageError> {
        println!("  🔑 Генерация ключа через Argon2id...");
        let start = now_millis();
        let key = StorageKey::from_password(password)?;
        let elapsed = now_millis() - start;
        println!("  ✅ Ключ готов за {}ms", elapsed);
        println!(
            "  🔐 SQLCipher ключ: x'{}...' (AES-256)",
            &key.to_hex()[..16]
        );

        let conn = Connection::open(path)?;

        Ok(Database {
            conn: Some(conn),
            key,
            merkle: MerkleTree::new(),
            db_path: path.to_string(),
            panicked: false,
        })
    }

    fn conn(&self) -> Result<&Connection, StorageError> {
        self.conn.as_ref().ok_or(StorageError::NotOpen)
    }

    /// Создать таблицы при первом запуске
    pub fn initialize(&self) -> Result<(), StorageError> {
        self.conn()?.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                from_key    TEXT    NOT NULL,
                to_key      TEXT    NOT NULL,
                content     TEXT    NOT NULL,
                timestamp   INTEGER NOT NULL,
                is_read     INTEGER NOT NULL DEFAULT 0,
                merkle_hash TEXT    NOT NULL
            );
            CREATE TABLE IF NOT EXISTS contacts (
                public_key  TEXT    PRIMARY KEY,
                nickname    TEXT    NOT NULL,
                added_at    INTEGER NOT NULL,
                reputation  INTEGER NOT NULL DEFAULT 50
            );
            CREATE TABLE IF NOT EXISTS merkle_roots (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                root_hash   TEXT    NOT NULL,
                msg_count   INTEGER NOT NULL,
                timestamp   INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_messages_to
                ON messages(to_key, timestamp);
            CREATE INDEX IF NOT EXISTS idx_messages_from
                ON messages(from_key, timestamp);
            CREATE TABLE IF NOT EXISTS identity (
                id          INTEGER PRIMARY KEY,
                public_key  TEXT    NOT NULL,
                mnemonic    TEXT    NOT NULL,
                created_at  INTEGER NOT NULL
            );
        ",
        )?;
        println!("  📋 Таблицы инициализированы");
        Ok(())
    }

    // ── Сообщения ────────────────────────────────────────────

    pub fn save_message(&mut self, mut msg: Message) -> Result<u64, StorageError> {
        let hash = msg.hash();
        let hash_hex = hex::encode(hash);

        let conn = self.conn.as_ref().ok_or(StorageError::NotOpen)?;
        conn.execute(
            "INSERT INTO messages (from_key, to_key, content, timestamp, is_read, merkle_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                msg.from_key,
                msg.to_key,
                msg.content,
                msg.timestamp,
                msg.is_read as i32,
                hash_hex
            ],
        )?;

        let id = conn.last_insert_rowid() as u64;
        msg.id = id;

        self.merkle.add_leaf(hash);

        let root = self.merkle.root().map(hex::encode).unwrap_or_default();
        conn.execute(
            "INSERT INTO merkle_roots (root_hash, msg_count, timestamp) VALUES (?1, ?2, ?3)",
            params![root, self.merkle.len() as i64, now_secs() as i64],
        )?;

        Ok(id)
    }

    pub fn get_messages(&self, public_key: &str) -> Result<Vec<Message>, StorageError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, from_key, to_key, content, timestamp, is_read
             FROM messages
             WHERE to_key = ?1 OR from_key = ?1
             ORDER BY timestamp ASC",
        )?;

        let messages = stmt
            .query_map(params![public_key], |row| {
                Ok(Message {
                    id: row.get::<_, i64>(0)? as u64,
                    from_key: row.get(1)?,
                    to_key: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: row.get::<_, i64>(4)? as u64,
                    is_read: row.get::<_, i32>(5)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    pub fn mark_read(&self, msg_id: u64) -> Result<(), StorageError> {
        let updated = self.conn()?.execute(
            "UPDATE messages SET is_read = 1 WHERE id = ?1",
            params![msg_id as i64],
        )?;
        if updated == 0 {
            return Err(StorageError::MessageNotFound(msg_id));
        }
        Ok(())
    }

    pub fn unread_count(&self, public_key: &str) -> Result<u64, StorageError> {
        let count: i64 = self.conn()?.query_row(
            "SELECT COUNT(*) FROM messages WHERE to_key = ?1 AND is_read = 0",
            params![public_key],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    // ── Identity ─────────────────────────────────────────────

    pub fn save_identity(&self, public_key: &str, mnemonic: &str) -> Result<(), StorageError> {
        self.conn()?.execute(
            "INSERT OR REPLACE INTO identity (id, public_key, mnemonic, created_at)
             VALUES (1, ?1, ?2, ?3)",
            params![public_key, mnemonic, now_secs()],
        )?;
        Ok(())
    }

    pub fn load_identity(&self) -> Result<Option<(String, String)>, StorageError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT public_key, mnemonic FROM identity WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    // ── Контакты ─────────────────────────────────────────────

    pub fn add_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        // Проверка дубликатов по имени
        let count: i64 = self.conn()?.query_row(
            "SELECT COUNT(*) FROM contacts WHERE nickname = ?1 AND public_key != ?2",
            params![contact.nickname, contact.public_key],
            |row| row.get(0),
        )?;
        if count > 0 {
            // Имя уже занято другим контактом — добавляем суффикс
            println!(
                "  ⚠️  Имя '{}' уже занято — используй другое",
                contact.nickname
            );
        }

        self.conn()?.execute(
            "INSERT OR REPLACE INTO contacts (public_key, nickname, added_at, reputation)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                contact.public_key,
                contact.nickname,
                contact.added_at as i64,
                contact.reputation as i32
            ],
        )?;
        Ok(())
    }

    pub fn get_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("SELECT public_key, nickname, added_at, reputation FROM contacts")?;
        let contacts = stmt
            .query_map([], |row| {
                Ok(Contact {
                    public_key: row.get(0)?,
                    nickname: row.get(1)?,
                    added_at: row.get::<_, i64>(2)? as u64,
                    reputation: row.get::<_, i32>(3)? as u8,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(contacts)
    }

    // ── Merkle верификация ───────────────────────────────────

    pub fn verify_integrity(&self) -> Result<bool, StorageError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT merkle_hash FROM messages ORDER BY id ASC")?;
        let db_hashes: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

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

    // ── Panic Button ─────────────────────────────────────────

    pub fn panic_destroy(&mut self) -> StorageError {
        self.panicked = true;
        PanicButton::activate(&mut self.key, &mut self.conn)
    }

    // ── Статистика ───────────────────────────────────────────

    pub fn print_stats(&self) {
        println!("\n  ╔══════════════════════════════════════════════╗");
        println!("  ║         СТАТИСТИКА STORAGE ROOT v2.0         ║");
        println!("  ╠══════════════════════════════════════════════╣");
        println!("  ║ Файл БД:        {:>26}  ║", self.db_path);
        println!("  ║ Шифрование:     {:>26}  ║", "AES-256 (SQLCipher)");
        println!("  ║ KDF:            {:>26}  ║", "Argon2id (64MB, 3 iter)");
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
