// ============================================================
// ROOT v2.0 — api/p2p.rs
// FFI функции: P2P сеть
// ============================================================

use root_core::state::IncomingMessage;
use super::state::APP_STATE;
use super::types::{ApiError, MessageInfo};

pub fn start_p2p_node() -> Result<String, ApiError> {
    use crate::transport::start_node_channels;

    // Берём ключ до создания runtime — освобождаем Mutex
    let key_bytes = {
        let state = APP_STATE.lock().unwrap();
        let identity = state.identity.as_ref().ok_or(ApiError::IdentityNotInitialized)?;
        identity.signing_key_bytes()
    };

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    let (tx_out, mut rx_in) = rt
        .block_on(start_node_channels(key_bytes))
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    APP_STATE.lock().unwrap().p2p_sender = Some(tx_out);

    // Фоновый поток — слушает входящие P2P сообщения
    std::thread::spawn(move || {
        let local_rt = tokio::runtime::Runtime::new().unwrap();
        local_rt.block_on(async move {
            while let Some(msg) = rx_in.recv().await {
                println!("📨 ВХОДЯЩЕЕ: от={} текст={}", msg.from_peer, msg.content);
                APP_STATE.lock().unwrap().incoming_queue.push(IncomingMessage {
                    from_peer: msg.from_peer,
                    content:   msg.content,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
            }
        });
        rt.block_on(std::future::pending::<()>());
    });

    println!("  ✅ P2P узел запущен");
    Ok("p2p-node-started".to_string())
}

pub fn send_p2p_message(content: String) -> Result<(), ApiError> {
    let state = APP_STATE.lock().unwrap();
    let sender = state.p2p_sender.as_ref().ok_or_else(|| {
        ApiError::StorageError("P2P узел не запущен. Вызови start_p2p_node()".to_string())
    })?;
    sender
        .try_send(content)
        .map_err(|e| ApiError::StorageError(e.to_string()))?;
    Ok(())
}

pub fn is_p2p_running() -> bool {
    APP_STATE.lock().unwrap().p2p_sender.is_some()
}

/// Забирает все входящие сообщения из очереди и конвертирует в MessageInfo для Flutter
pub fn get_incoming_messages() -> Vec<MessageInfo> {
    let mut state = APP_STATE.lock().unwrap();
    state.incoming_queue
        .drain(..)
        .map(|m| MessageInfo {
            id:        0,
            from_key:  m.from_peer,
            to_key:    String::new(),
            content:   m.content,
            timestamp: m.timestamp,
            is_read:   false,
            from_name: None,
        })
        .collect()
}

pub fn get_peer_count() -> u32 {
    APP_STATE.lock().unwrap().peer_count
}
