// ============================================================
// ROOT v2.0 — root-ffi/src/api/p2p.rs
// FFI функции: P2P сеть (исправлено: единый Runtime)
// ============================================================

use crate::runtime::runtime_handle;
use crate::api::state::APP_STATE;
use crate::api::types::{ApiError, MessageInfo};
use root_core::state::IncomingMessage;  // ✅ Правильный тип из root_core
use log::{info, error};
use tokio::sync::oneshot;

pub fn start_p2p_node() -> Result<String, ApiError> {
    // 1. Получаем ключ из AppState (SecretSeed автоматически затирается)
    let key_bytes: [u8; 32] = {
        let state = APP_STATE.lock()
            .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;
        let identity = state.identity.as_ref()
            .ok_or_else(|| ApiError::StorageError("Identity not initialized".into()))?;
        let seed = identity.signing_key_bytes();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&seed.0[..32]);
        bytes
    };

    // 2. Создаём канал для остановки
    let (shutdown_tx, shutdown_rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();

    // 3. Берём Handle единого Runtime
    let handle = runtime_handle();

    // 4. Запускаем P2P асинхронно — БЕЗ block_on и БЕЗ нового Runtime!
    handle.spawn(async move {
        match root_network::channels::start_node_channels(key_bytes, shutdown_rx).await {
            Ok((tx_out, mut rx_in)) => {
                // 5. Сохраняем sender в AppState
                {
                    let mut state = APP_STATE.lock().unwrap();
                    state.p2p_sender = Some(tx_out);
                    state.p2p_shutdown = Some(shutdown_tx);
                }

                info!("✅ P2P узел запущен");

                // 6. Слушаем входящие сообщения в том же Runtime
                while let Some(msg) = rx_in.recv().await {
                    info!("📨 ВХОДЯЩЕЕ: от={} текст={}", msg.from_peer, msg.content);
                    
                    // ✅ Используем IncomingMessage из root_core
                    APP_STATE.lock().unwrap().incoming_queue.push(IncomingMessage {
                        from_peer: msg.from_peer,
                        content: msg.content,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    });
                }
            }
            Err(e) => {
                error!("❌ Ошибка запуска P2P: {}", e);
            }
        }
    });

    Ok("p2p-node-started".to_string())
}

pub fn send_p2p_message(content: String) -> Result<(), ApiError> {
    let state = APP_STATE.lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;
    let sender = state.p2p_sender.as_ref()
        .ok_or_else(|| ApiError::StorageError("P2P узел не запущен. Вызови start_p2p_node()".into()))?;
    
    sender
        .try_send(content)
        .map_err(|e| ApiError::StorageError(format!("{}", e)))?;
    Ok(())
}

pub fn is_p2p_running() -> bool {
    APP_STATE.lock().unwrap().p2p_sender.is_some()
}

pub fn get_incoming_messages() -> Vec<MessageInfo> {
    let mut state = APP_STATE.lock().unwrap();
    state.incoming_queue
        .drain(..)
        .map(|m| MessageInfo {
            id: 0,  // TODO: добавить id в IncomingMessage или генерировать
            from_key: m.from_peer,
            to_key: String::new(),  // TODO: добавить to_peer в IncomingMessage
            content: m.content,
            timestamp: m.timestamp,
            is_read: false,
            from_name: None,  // TODO: подставлять имя из контактов
        })
        .collect()
}

pub fn get_peer_count() -> u32 {
    APP_STATE.lock().unwrap().peer_count
}

/// 🔴 НОВАЯ ФУНКЦИЯ: Корректная остановка P2P узла
pub fn stop_p2p_node() -> Result<(), ApiError> {
    let mut state = APP_STATE.lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;
    
    if let Some(tx) = state.p2p_shutdown.take() {
        let _ = tx.send(());  // Игнорируем ошибку, если receiver уже упал
        info!("🛑 Сигнал остановки P2P отправлен");
    }
    
    state.p2p_sender = None;
    Ok(())
}
