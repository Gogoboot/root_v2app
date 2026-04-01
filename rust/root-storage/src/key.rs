use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use std::fs;
use std::io::Write; // Вынесено наверх
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

// Ошибки модуля ключей
#[derive(Debug)]
pub enum KeyError {
    KeyringError(String),
    IoError(std::io::Error),
    MigrationFailed(String),
    NotInitialized,
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyError::KeyringError(e) => write!(f, "Keyring error: {}", e),
            KeyError::IoError(e) => write!(f, "IO error: {}", e),
            KeyError::MigrationFailed(e) => write!(f, "Migration failed: {}", e),
            KeyError::NotInitialized => write!(f, "Salt not initialized"),
        }
    }
}

impl std::error::Error for KeyError {}

// Константы для Keychain
const KEYRING_SERVICE: &str = "root_app";
const KEYRING_USER: &str = "device_salt";
const SALT_FILE_NAME: &str = "salt.bin";
const SALT_SIZE: usize = 32; // 256 бит

/// Менеджер соли, использующий OS Keychain с поддержкой миграции с файла.
pub struct SaltManager {
    salt: Option<Zeroizing<[u8; SALT_SIZE]>>,
}

impl SaltManager {
    pub fn new(app_data_dir: &Path) -> Result<Self, KeyError> {
        let mut manager = Self { salt: None };

        // Попытка загрузить или создать соль
        let salt = manager.load_or_create_salt(app_data_dir)?;
        manager.salt = Some(salt);

        Ok(manager)
    }

    /// Возвращает ссылку на массив байт [u8; 32]
    /// Возвращает ссылку на массив байт [u8; 32]
    pub fn get_salt(&self) -> Result<&[u8; SALT_SIZE], KeyError> {
        match &self.salt {
            Some(s) => Ok(&**s), // ✅ Явно и понятно
            None => Err(KeyError::NotInitialized),
        }
    }

    /// Основная логика: Keychain -> Миграция с файла -> Генерация новой
    fn load_or_create_salt(
        &self,
        app_data_dir: &Path,
    ) -> Result<Zeroizing<[u8; SALT_SIZE]>, KeyError> {
        // 1. Попытка получить из Keychain
        match self.read_from_keyring() {
            Ok(salt) => {
                log::info!("Salt loaded successfully from OS Keychain.");
                return Ok(salt);
            }
            Err(KeyError::KeyringError(ref msg))
                if msg.contains("not found")
                    || msg.contains("No entry")
                    || msg.contains("NotFound")
                    || msg.contains("No matching entry") =>
            {
                // Не найдено в Keychain, пробуем миграцию
                log::info!("Salt not found in Keychain, checking for legacy file migration...");
            }
            Err(e) => {
                // Другая ошибка Keychain
                log::error!("Critical Keychain error: {}", e);
                return Err(e);
            }
        }

        // 2. Попытка миграции со старого файла salt.bin
        let salt_file_path = app_data_dir.join(SALT_FILE_NAME);
        if salt_file_path.exists() {
            log::warn!("Legacy salt file detected. Starting migration to Keychain...");
            return self.migrate_from_file(&salt_file_path);
        }

        // 3. Генерация новой соли (первый запуск или чистая установка)
        log::info!("No salt found. Generating new secure salt...");
        let mut new_salt = Zeroizing::new([0u8; SALT_SIZE]);
        rand::thread_rng().fill_bytes(new_salt.as_mut_slice());

        self.save_to_keyring(&new_salt)?;

        Ok(new_salt)
    }

    /// Чтение из OS Keychain
    fn read_from_keyring(&self) -> Result<Zeroizing<[u8; SALT_SIZE]>, KeyError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| KeyError::KeyringError(e.to_string()))?;

        // ИСПРАВЛЕНО: используем get_password() вместо get_secret()
        let password_str = entry
            .get_password()
            .map_err(|e| KeyError::KeyringError(e.to_string()))?;

        // Декодируем Base64 обратно в байты
        let decoded = BASE64
            .decode(&password_str)
            .map_err(|e| KeyError::KeyringError(format!("Base64 decode failed: {}", e)))?;

        if decoded.len() != SALT_SIZE {
            return Err(KeyError::KeyringError(
                "Invalid salt size in Keychain".to_string(),
            ));
        }

        let mut salt = Zeroizing::new([0u8; SALT_SIZE]);
        salt.copy_from_slice(&decoded);

        Ok(salt)
    }

    /// Сохранение в OS Keychain
    fn save_to_keyring(&self, salt: &[u8; SALT_SIZE]) -> Result<(), KeyError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| KeyError::KeyringError(e.to_string()))?;

        // Кодируем в Base64 для хранения как строки
        let encoded = BASE64.encode(salt);

        // ИСПРАВЛЕНО: используем set_password() вместо set_secret()
        entry
            .set_password(&encoded)
            .map_err(|e| KeyError::KeyringError(e.to_string()))?;

        Ok(())
    }

    /// Логика миграции: Читает файл -> Пишет в Keychain -> Удаляет файл
    fn migrate_from_file(&self, file_path: &Path) -> Result<Zeroizing<[u8; SALT_SIZE]>, KeyError> {
        // Чтение файла
        let file_content = fs::read(file_path).map_err(KeyError::IoError)?;

        if file_content.len() != SALT_SIZE {
            return Err(KeyError::MigrationFailed(
                "Invalid salt file size".to_string(),
            ));
        }

        let mut salt = Zeroizing::new([0u8; SALT_SIZE]);
        salt.copy_from_slice(&file_content);

        // Сохранение в Keychain
        self.save_to_keyring(&salt)?;
        log::info!("Salt successfully saved to Keychain.");

        // Безопасное удаление файла
        // 1. Перезаписываем нулями несколько раз
        let mut file_opts = fs::OpenOptions::new()
            .write(true)
            .open(file_path)
            .map_err(KeyError::IoError)?;

        let zeros = vec![0u8; SALT_SIZE];
        for _ in 0..3 {
            file_opts.write_all(&zeros).map_err(KeyError::IoError)?;
            file_opts.sync_data().map_err(KeyError::IoError)?;
        }
        drop(file_opts);

        // 2. Удаляем файл
        fs::remove_file(file_path).map_err(|e| {
            log::error!("Failed to delete legacy salt file: {}", e);
            KeyError::MigrationFailed(format!("Could not delete old file: {}", e))
        })?;

        log::info!("Legacy salt file securely deleted. Migration complete.");

        Ok(salt)
    }

    // Метод для очистки (используется при Panic Button или выходе)
    pub fn clear_memory(&mut self) {
        if let Some(ref mut s) = self.salt {
            s.zeroize();
        }
        self.salt = None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;  // ✅ Используется

    #[test]
    fn test_salt_manager_creation() {
        let temp_dir = std::env::temp_dir().join("root_test_salt");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let manager = SaltManager::new(&temp_dir);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        let salt = manager.get_salt().unwrap();
        assert_eq!(salt.len(), SALT_SIZE);

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}

// В конце file::storage/key.rs
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_salt_manager_creation() {
        let temp_dir = std::env::temp_dir().join("root_test_salt");
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = SaltManager::new(&temp_dir);
        assert!(manager.is_ok());

        // Проверка получения соли
        let manager = manager.unwrap();
        let salt = manager.get_salt().unwrap();
        assert_eq!(salt.len(), SALT_SIZE);

        // Очистка
        drop(manager);
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_salt_uniqueness() {
        let temp_dir = std::env::temp_dir().join("root_test_salt_2");
        fs::create_dir_all(&temp_dir).unwrap();

        let manager1 = SaltManager::new(&temp_dir).unwrap();
        let salt1 = *manager1.get_salt().unwrap();

        drop(manager1);

        // При повторном создании та же соль должна быть восстановлена
        let manager2 = SaltManager::new(&temp_dir).unwrap();
        let salt2 = *manager2.get_salt().unwrap();

        assert_eq!(salt1, salt2, "Соль должна сохраняться между запусками");

        drop(manager2);
        fs::remove_dir_all(&temp_dir).ok();
    }
}
