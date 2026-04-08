// ============================================================
// ROOT v2.0 — api/messaging.rs
// FFI функции: отправка и получение сообщений
// ============================================================

// NodeCommand вместо P2pOutMessage — generate_topic_id остаётся
use root_network::channels::NodeCommand;
use root_network::generate_topic_id;
use root_storage::Message;
use std::time::{SystemTime, UNIX_EPOCH};
use super::identity::get_public_key;
use super::state::APP_STATE;
use super::types::{ApiError, MessageInfo};
use crate::require_state;
use root_core::state::AppPhase;

pub fn send_message(to_key: String, content: String) -> Result<u64, ApiError> {
    require_state!(root_core::state::AppPhase::Ready | root_core::state::AppPhase::P2PActive);

    let from_key = get_public_key()?;
    let msg = Message::new(from_key.clone(), to_key.clone(), content.clone());

    // 1. Сохраняем в БД
    let id = {
        let mut state = APP_STATE.lock()
            .map_err(|_| ApiError::StorageError("Lock poisoned".into()))?;
        let db = state.database.as_mut().ok_or(ApiError::DatabaseNotOpen)?;
        db.save_message(msg).map_err(ApiError::from)?
    }; // ← state освобождается здесь

    // 2. Отправляем в P2P если запущен — иначе сообщение остаётся только в БД
    {
        let state = APP_STATE.lock()
            .map_err(|_| ApiError::StorageError("Lock poisoned".into()))?;

        if let Some(sender) = state.p2p_sender.as_ref() {
            // P2P запущен — вычисляем топик и отправляем NodeCommand::Publish
            let identity = state.identity.as_ref()
                .ok_or(ApiError::IdentityNotInitialized)?;
            let own_pubkey = identity.public_key_hex();
            let topic = generate_topic_id(&own_pubkey, &to_key);

            // Канал переполнен — не критично, сообщение уже в БД
            if let Err(e) = sender.try_send(NodeCommand::Publish { topic, content: content.clone() }) {
                log::warn!("⚠️ P2P канал переполнен, сообщение только в БД: {}", e);
            }
        } else {
            // P2P не запущен — нормальная ситуация для офлайн режима
            log::info!("📦 P2P не запущен — сообщение сохранено локально (id: {})", id);
        }
    } // ← state освобождается здесь

    log::info!("✉️ Сообщение #{} сохранено и отправлено", id);
    Ok(id)
}

pub fn get_messages() -> Result<Vec<MessageInfo>, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;

    let messages = db
        .get_messages(&public_key, 0, 50)
        .map_err(ApiError::from)?;

    let contacts = db.get_contacts().unwrap_or_default();

    let infos = messages
        .into_iter()
        .map(|m| {
            let from_name = contacts
                .iter()
                .find(|c| c.public_key == m.from_key)
                .map(|c| c.nickname.clone());
            MessageInfo {
                id:        m.id.unwrap_or(0),
                from_key:  m.from_key,
                to_key:    m.to_key,
                content:   m.content,
                timestamp: m.timestamp,
                is_read:   m.is_read,
                from_name,
            }
        })
        .collect();

    Ok(infos)
}

pub fn get_unread_count() -> Result<u64, ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let public_key = get_public_key()?;
    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;

    db.unread_count(&public_key).map_err(ApiError::from)
}

pub fn mark_message_read(msg_id: u64) -> Result<(), ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let state = APP_STATE.lock().unwrap();
    let db = state.database.as_ref().ok_or(ApiError::DatabaseNotOpen)?;

    db.mark_read(msg_id).map_err(ApiError::from)
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
