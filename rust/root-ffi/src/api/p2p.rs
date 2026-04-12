// ============================================================
// ROOT v2.0 — ffi/api/p2p.rs
// P2P API — мост между Tauri командами и сетевым слоем
//
// Изменения:
//   - p2p_sender теперь Sender<NodeCommand> вместо Sender<P2pOutMessage>
//   - send_p2p_message отправляет NodeCommand::Publish
//   - добавлен dial_node — ручное подключение по Multiaddr
//   - добавлен get_peers — список активных пиров с протоколами
//   - добавлены get_bootstrap_list / save_bootstrap_list
//   - start_p2p_node читает bootstrap список и передаёт в start_node_channels
// ============================================================

use log::{info, error};

use crate::api::state::APP_STATE;
use crate::api::types::{MessageInfo, PeerInfoDto, ApiError};
use crate::require_state;
use root_core::state::AppPhase;
use root_network::channels::{NodeCommand, start_node_channels};
use root_network::generate_topic_id;


// ── Запуск / остановка ───────────────────────────────────────

pub fn start_p2p_node() -> Result<String, ApiError> {
    require_state!(AppPhase::Ready);

    // Берём key_bytes из identity
    let key_bytes: [u8; 32] = {
        let state = APP_STATE
            .lock()
            .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;
        let identity = state
            .identity
            .as_ref()
            .ok_or_else(|| ApiError::StorageError("Identity not initialized".into()))?;
        let seed = identity.signing_key_bytes();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&seed.0[..32]);
        bytes
    };

    // Читаем bootstrap список из БД
    // Если БД недоступна или список пуст — стартуем без bootstrap (только mDNS)
    let bootstrap_addrs: Vec<String> = {
        let state = APP_STATE
            .lock()
            .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;
        if let Some(db) = state.database.as_ref() {
            db.get_setting("bootstrap_addrs")
                .unwrap_or(None)
                .map(|json: String| {
                    serde_json::from_str::<Vec<String>>(&json).unwrap_or_default()
                })
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    info!("  🔗 Bootstrap адресов: {}", bootstrap_addrs.len());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let handle = tokio::runtime::Handle::try_current()
        .unwrap_or_else(|_| crate::runtime::APP_RUNTIME.handle().clone());

    handle.spawn(async move {
        match start_node_channels(key_bytes, shutdown_rx, bootstrap_addrs).await {
            Ok((tx_cmd, mut rx_in, mut rx_peers)) => {
                {
                    let mut state = APP_STATE.lock().unwrap();
                    state.p2p_sender   = Some(tx_cmd);
                    state.p2p_shutdown = Some(shutdown_tx);
                    // Переход в P2PActive после реального старта — не до
                    state.transition(AppPhase::P2PActive);
                }
                info!("✅ P2P узел запущен, фаза → P2PActive");

                // Фоновый таск: обновляем список пиров в state
                tokio::spawn(async move {
                    while let Some(peers) = rx_peers.recv().await {
                        let mut state = APP_STATE.lock().unwrap();
                        state.peer_list  = peers.clone();
                        state.peer_count = peers.len() as u32;
                    }
                });

                // Основной цикл: читаем входящие сообщения и сохраняем в БД
                while let Some(msg) = rx_in.recv().await {
                    let (my_key, has_db) = {
                        let state = APP_STATE.lock().unwrap();
                        let key: String = state
                            .identity
                            .as_ref()
                            .map(|id: &root_identity::Identity| id.public_key_hex())
                            .unwrap_or_default();
                        let has_db = state.database.is_some();
                        (key, has_db)
                    };

                    if !has_db || my_key.is_empty() {
                        log::warn!("⚠️ БД не открыта — входящее сообщение потеряно");
                        continue;
                    }
                    
                    // Парсим JSON с nonce — достаём только текст для сохранения в БД
                    // Если формат не JSON (старый клиент) — сохраняем как есть
                    let text = serde_json::from_str::<serde_json::Value>(&msg.content)
                        .ok()
                        .and_then(|v| v["text"].as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| msg.content.clone());

                    let message = root_storage::Message::new(
                        msg.from_pubkey.clone(),
                        my_key,
                        text,
                    );

                    let mut state = APP_STATE.lock().unwrap();
                    if let Some(db) = state.database.as_mut() {
                        if let Err(e) = db.save_message(message) {
                            log::error!("❌ Не удалось сохранить входящее сообщение: {}", e);
                        } else {
                            log::info!(
                                "📨 Сохранено от: {}...",
                                &msg.from_peer[..8.min(msg.from_peer.len())]
                            );
                        }
                    }
                }

                // Цикл завершился — возвращаемся в Ready
                APP_STATE.lock().unwrap().transition(AppPhase::Ready);
                info!("🛑 P2P узел остановлен, фаза → Ready");
            }
            Err(e) => {
                error!("❌ Ошибка запуска P2P: {}", e);
            }
        }
    });

    Ok("p2p-node-starting".to_string())
}

pub fn stop_p2p_node() -> Result<(), ApiError> {
    let mut state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    // Дропаем shutdown sender — нода получит сигнал и остановится
    let _ = state.p2p_shutdown.take();
    state.p2p_sender = None;
    state.peer_list  = vec![];
    state.peer_count = 0;

    Ok(())
}

// ── Отправка сообщений ───────────────────────────────────────

pub fn send_p2p_message(recipient_pubkey: String, content: String) -> Result<(), ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);

    let state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    let identity = state
        .identity
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("Identity not initialized".into()))?;

    let own_pubkey = identity.public_key_hex();

    // Приватный топик: hash(sort(own_pubkey, recipient_pubkey))
    let topic = generate_topic_id(&own_pubkey, &recipient_pubkey);

    let sender = state
        .p2p_sender
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("P2P узел не запущен".into()))?;

    sender
        .try_send(NodeCommand::Publish { topic, content })
        .map_err(|e| ApiError::StorageError(format!("{}", e)))?;

    Ok(())
}

// ── Ручной диал ─────────────────────────────────────────────

/// Подключиться к пиру по Multiaddr строке
/// Пример: "/dns4/host.ngrok-free.app/tcp/443/wss/p2p/12D3..."
pub fn dial_node(addr: String) -> Result<(), ApiError> {
    require_state!(AppPhase::P2PActive);

    let state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    let sender = state
        .p2p_sender
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("P2P узел не запущен".into()))?;

    sender
        .try_send(NodeCommand::Dial(addr))
        .map_err(|e| ApiError::StorageError(format!("{}", e)))?;

    Ok(())
}

// ── Список пиров ─────────────────────────────────────────────

pub fn is_p2p_running() -> bool {
    APP_STATE.lock().unwrap().p2p_sender.is_some()
}

pub fn get_peer_count() -> u32 {
    APP_STATE.lock().unwrap().peer_count
}

/// Список активных пиров с протоколами — для UI вкладки Сеть
pub fn get_peers() -> Vec<PeerInfoDto> {
    APP_STATE
        .lock()
        .unwrap()
        .peer_list
        .iter()
        .map(|p| PeerInfoDto {
            peer_id:      p.peer_id.clone(),
            protocol:     p.protocol.clone(),
            connected_at: p.connected_at,
        })
        .collect()
}

// ── Bootstrap список ─────────────────────────────────────────

/// Получить список bootstrap адресов из БД
pub fn get_bootstrap_list() -> Result<Vec<String>, ApiError> {
    let state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    let db = state
        .database
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("БД не открыта".into()))?;

    let list = db
        .get_setting("bootstrap_addrs")
        .unwrap_or(None)
        .map(|json: String| serde_json::from_str::<Vec<String>>(&json).unwrap_or_default())
        .unwrap_or_default();

    Ok(list)
}

/// Сохранить список bootstrap адресов в БД
pub fn save_bootstrap_list(addrs: Vec<String>) -> Result<(), ApiError> {
    let state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    let db = state
        .database
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("БД не открыта".into()))?;

    let json = serde_json::to_string(&addrs)
        .map_err(|e| ApiError::StorageError(format!("Сериализация: {}", e)))?;

    db.set_setting("bootstrap_addrs", &json)
        .map_err(|e: root_storage::StorageError| ApiError::StorageError(e.to_string()))?;

    info!("  💾 Bootstrap список сохранён: {} адресов", addrs.len());
    Ok(())
}

// ── Входящие сообщения ───────────────────────────────────────

pub fn get_incoming_messages() -> Vec<MessageInfo> {
    let state = APP_STATE.lock().unwrap();

    let my_key: String = match state.identity.as_ref() {
        Some(id) => id.public_key_hex(),
        None => return vec![],
    };

    let db: &root_storage::Database = match state.database.as_ref() {
        Some(db) => db,
        None => return vec![],
    };

    db.get_messages(&my_key, 0, 50)
        .unwrap_or_default()
        .into_iter()
        .map(|m| MessageInfo {
            id:        m.id.unwrap_or(0),
            from_key:  m.from_key,
            to_key:    m.to_key,
            content:   m.content,
            timestamp: m.timestamp,
            is_read:   m.is_read,
            from_name: None,
        })
        .collect()
}

// ── Вспомогательные функции ──────────────────────────────────
