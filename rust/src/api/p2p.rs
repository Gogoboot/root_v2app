// ============================================================
// ROOT v2.0 — api/p2p.rs
//
// FFI функции: P2P сеть
// ============================================================

use super::identity::get_public_key;
use super::state::{INCOMING_QUEUE, PEER_COUNT, P2P_SENDER};
use super::types::{ApiError, MessageInfo};

pub fn start_p2p_node() -> Result<String, ApiError> {
    use crate::transport::start_node_channels;

    // Берём ключ до создания runtime — освобождаем Mutex
    let key_bytes = {
        let guard    = super::state::CURRENT_IDENTITY.lock().unwrap();
        let identity = guard.as_ref().ok_or(ApiError::IdentityNotInitialized)?;
        identity.signing_key_bytes()
    };

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    let (tx_out, mut rx_in) = rt
        .block_on(start_node_channels(key_bytes))
        .map_err(|e| ApiError::StorageError(e.to_string()))?;

    *P2P_SENDER.lock().unwrap() = Some(tx_out);

    let _public_key = get_public_key().unwrap_or_default();

    // Фоновый поток — слушает входящие P2P сообщения
    // Создаёт собственный runtime чтобы не блокировать глобальный
    std::thread::spawn(move || {
        let local_rt = tokio::runtime::Runtime::new().unwrap();
        local_rt.block_on(async move {
            while let Some(msg) = rx_in.recv().await {
                println!("📨 ВХОДЯЩЕЕ: от={} текст={}", msg.from_peer, msg.content);

                let info = MessageInfo {
                    id:        0,
                    from_key:  msg.from_peer.clone(),
                    to_key:    String::new(),
                    content:   msg.content.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    is_read:   false,
                    from_name: None,
                };
                INCOMING_QUEUE.lock().unwrap().push(info);
            }
        });
        // Runtime держит фоновый P2P поток живым
        rt.block_on(std::future::pending::<()>());
    });

    println!("  ✅ P2P узел запущен");
    Ok("p2p-node-started".to_string())
}

pub fn send_p2p_message(content: String) -> Result<(), ApiError> {
    let guard  = P2P_SENDER.lock().unwrap();
    let sender = guard.as_ref().ok_or_else(|| {
        ApiError::StorageError("P2P узел не запущен. Вызови start_p2p_node()".to_string())
    })?;

    sender.try_send(content)
        .map_err(|e| ApiError::StorageError(e.to_string()))?;
    Ok(())
}

pub fn is_p2p_running() -> bool {
    P2P_SENDER.lock().unwrap().is_some()
}

/// Получить входящие P2P сообщения из очереди памяти
/// drain() — забирает все сообщения и очищает очередь
pub fn get_incoming_messages() -> Vec<MessageInfo> {
    let mut queue = INCOMING_QUEUE.lock().unwrap();
    queue.drain(..).collect()
}

/// Количество подключённых пиров
pub fn get_peer_count() -> u32 {
    *PEER_COUNT.lock().unwrap()
}
