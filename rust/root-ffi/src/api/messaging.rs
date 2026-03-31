// ============================================================
// ROOT v2.0 — api/messaging.rs
// FFI функции: отправка и получение сообщений
// ============================================================

use root_network::{P2pOutMessage, generate_topic_id};
use root_storage::{Message, StorageError};
use std::time::{SystemTime, UNIX_EPOCH};
use super::identity::get_public_key;
use super::state::APP_STATE;
use super::types::{ApiError, MessageInfo};


pub fn send_message(to_key: String, content: String) -> Result<u64, ApiError> {
    let from_key = get_public_key()?;
    let msg = Message::new(from_key.clone(), to_key.clone(), content.clone());

    // 1. Сохраняем в БД
    let id = {
        let mut state = APP_STATE.lock().map_err(|_| ApiError::StorageError("Lock poisoned".into()))?;
        let db = state.database.as_mut().ok_or(ApiError::DatabaseNotOpen)?;
        db.save_message(msg)
            .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))?
    };

    // 2. Отправляем в приватный P2P топик, если узел запущен
    {
        // Получаем AppState (новый лок — предыдущий уже освобождён)
        let state = APP_STATE.lock().map_err(|_| ApiError::StorageError("Lock poisoned".into()))?;
        
        // Проверяем, что P2P запущен
        let sender = state.p2p_sender.as_ref()
            .ok_or_else(|| ApiError::StorageError("P2P узел не запущен".into()))?;
        
        // Получаем identity для вычисления топика
        let identity = state.identity.as_ref()
            .ok_or_else(|| ApiError::StorageError("Identity not initialized".into()))?;
        
        // Вычисляем приватный топик: hash(sort(own_pubkey, to_key))
        let own_pubkey = identity.public_key_hex();
        let topic = generate_topic_id(&own_pubkey, &to_key);  // ← to_key, не recipient_pubkey!
        
        // Конструируем P2pOutMessage
        let message = P2pOutMessage {
            topic,              // ← приватный топик (хеш пары ключей)
            content: content.clone(),  // ← content, не payload!
        };
        
        // Отправляем в сеть
        sender.try_send(message)
            .map_err(|e| ApiError::StorageError(format!("{}", e)))?;
    }

    // ✅ Заменили println! на log::info! (задача S-4 из Спринта 5)
    log::info!("✉️ Сообщение #{} сохранено и отправлено", id);
    Ok(id)
}



pub fn get_messages() -> Result<Vec<MessageInfo>, ApiError> {
    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    let messages = db
        .get_messages(&public_key)
        .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))?;
    let contacts = db.get_contacts().unwrap_or_default();
    let infos = messages
        .into_iter()
        .map(|m| {
            let from_name = contacts
                .iter()
                .find(|c| c.public_key == m.from_key)
                .map(|c| c.nickname.clone());
            MessageInfo {
                id: m.id.unwrap_or(0),
                from_key: m.from_key,
                to_key: m.to_key,
                content: m.content,
                timestamp: m.timestamp,
                is_read: m.is_read,
                from_name,
            }
        })
        .collect();
    Ok(infos)
}

pub fn get_unread_count() -> Result<u64, ApiError> {
    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.unread_count(&public_key)
        .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))
}

pub fn mark_message_read(msg_id: u64) -> Result<(), ApiError> {
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;
    db.mark_read(msg_id)
        .map_err(|e: StorageError| ApiError::StorageError(e.to_string()))
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
