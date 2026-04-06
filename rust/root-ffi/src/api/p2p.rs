// ============================================================
// ROOT v2.0 — root-ffi/src/api/p2p.rs
// FFI функции: P2P сеть (исправлено: единый Runtime)
// ============================================================

use crate::api::state::APP_STATE;
use crate::api::types::{ApiError, MessageInfo};
use crate::require_state;
use log::{error, info};
use root_core::state::AppPhase;
use root_core::state::IncomingMessage; // ✅ Правильный тип из root_core
use root_network::{P2pOutMessage, generate_topic_id}; // ← Оба импорта!

pub fn start_p2p_node() -> Result<String, ApiError> {
    require_state!(root_core::state::AppPhase::Ready);

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

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let handle = tokio::runtime::Handle::try_current()
        .unwrap_or_else(|_| crate::runtime::APP_RUNTIME.handle().clone());

    handle.spawn(async move {
        match root_network::channels::start_node_channels(key_bytes, shutdown_rx).await {
            Ok((tx_out, mut rx_in, mut rx_peer_count)) => {
                {
                    let mut state = APP_STATE.lock().unwrap();
                    state.p2p_sender = Some(tx_out);
                    state.p2p_shutdown = Some(shutdown_tx);

                    // ✅ Переход ПОСЛЕ реального старта — не до
                    // Раньше это было снаружи spawn — race condition
                    state.transition(root_core::state::AppPhase::P2PActive);
                }
                info!("✅ P2P узел запущен, фаза → P2PActive");

                // Фоновый поток: обновляем счётчик пиров
                tokio::spawn(async move {
                    while let Some(count) = rx_peer_count.recv().await {
                        APP_STATE.lock().unwrap().peer_count = count;
                    }
                });

                // Основной цикл: читаем входящие сообщения
                while let Some(msg) = rx_in.recv().await {
                    // Шаг 1: берём лок, забираем только нужные данные, сразу отпускаем
                    let (my_key, has_db) = {
                        let state = APP_STATE.lock().unwrap();
                        let key = state
                            .identity
                            .as_ref()
                            .map(|id| id.public_key_hex())
                            .unwrap_or_default();
                        let has_db = state.database.is_some();
                        (key, has_db) // лок отпускается здесь автоматически
                    };

                    if !has_db || my_key.is_empty() {
                        log::warn!("⚠️ БД не открыта — входящее сообщение потеряно");
                        continue;
                    }

                    // Шаг 2: берём лок снова только для сохранения
                    let message = root_storage::Message::new(
                        msg.from_peer.clone(),
                        my_key,
                        msg.content.clone(),
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
                    // лок отпускается здесь
                }

                // Цикл завершился — P2P остановлен, возвращаемся в Ready
                APP_STATE
                    .lock()
                    .unwrap()
                    .transition(root_core::state::AppPhase::Ready);
                info!("🛑 P2P узел остановлен, фаза → Ready");
            }
            Err(e) => {
                // Запуск упал — фаза остаётся Ready, не P2PActive
                error!("❌ Ошибка запуска P2P: {}", e);
                // shutdown_tx дропается здесь автоматически — это нормально
            }
        }
    });

    // ℹ️ Возвращаем успех — P2P запускается асинхронно
    // Реальный переход в P2PActive произойдёт чуть позже внутри spawn
    Ok("p2p-node-starting".to_string())
}

// ✅ Исправленная функция:
pub fn send_p2p_message(recipient_pubkey: String, content: String) -> Result<(), ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    // 1. Получаем свою идентичность для вычисления топика
    let identity = state
        .identity
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("Identity not initialized".into()))?;

    // 2. Получаем свой публичный ключ (метод, который мы добавили в keys.rs)
    let own_pubkey = identity.public_key_hex();

    // 3. Вычисляем приватный топик: hash(sort(own_pubkey, recipient_pubkey))
    let topic = generate_topic_id(&own_pubkey, &recipient_pubkey);

    let sender = state
        .p2p_sender
        .as_ref()
        .ok_or_else(|| ApiError::StorageError("P2P узел не запущен".into()))?;

    // 4. Конструируем P2pOutMessage с правильными полями
    let message = P2pOutMessage {
        topic,   // ← ✅ Приватный топик (хеш пары ключей)
        content, // ← ✅ Текст сообщения
    };

    sender
        .try_send(message)
        .map_err(|e| ApiError::StorageError(format!("{}", e)))?;
    Ok(())
}

pub fn is_p2p_running() -> bool {
    APP_STATE.lock().unwrap().p2p_sender.is_some()
}

pub fn get_incoming_messages() -> Vec<MessageInfo> {
    let state = APP_STATE.lock().unwrap();

    // Получаем свой ключ
    let my_key = match state.identity.as_ref() {
        Some(id) => id.public_key_hex(),
        None => return vec![],
    };

    // Читаем из БД — все сообщения где я участник
    let db = match state.database.as_ref() {
        Some(db) => db,
        None => return vec![],
    };

    db.get_messages(&my_key, 0, 50)
        .unwrap_or_default()
        .into_iter()
        .map(|m| MessageInfo {
            id: m.id.unwrap_or(0),
            from_key: m.from_key,
            to_key: m.to_key,
            content: m.content,
            timestamp: m.timestamp,
            is_read: m.is_read,
            from_name: None,
        })
        .collect()
}

pub fn get_peer_count() -> u32 {
    APP_STATE.lock().unwrap().peer_count
}

/// 🔴 НОВАЯ ФУНКЦИЯ: Корректная остановка P2P узла
pub fn stop_p2p_node() -> Result<(), ApiError> {
    require_state!(AppPhase::Ready | AppPhase::P2PActive);
    let mut state = APP_STATE
        .lock()
        .map_err(|_| ApiError::StorageError("AppState lock poisoned".into()))?;

    if let Some(tx) = state.p2p_shutdown.take() {
        let _ = tx.send(()); // Игнорируем ошибку, если receiver уже упал
        info!("🛑 Сигнал остановки P2P отправлен");
    }

    state.p2p_sender = None;
    Ok(())
}
