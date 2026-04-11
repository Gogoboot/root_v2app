// ============================================================
// ROOT v2.0 — network/channels.rs
// P2P узел с каналами управления — Tauri/FFI интерфейс
//
// Изменения относительно предыдущей версии:
//   - P2pOutMessage → NodeCommand (Publish | Dial)
//   - Добавлен WS + DNS транспорт для ngrok bootstrap
//   - PeerInfo вместо u32 — плоский список с протоколом
//   - Автодиал bootstrap адресов при старте
// ============================================================

use futures::StreamExt;
use libp2p::{
    SwarmBuilder, gossipsub, mdns, noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};

use super::behaviour::{
    ROOT_TOPIC, RootBehaviour, RootBehaviourEvent,
    build_gossipsub, verify_message_sender,
};

// ── Публичные типы данных ────────────────────────────────────

/// Входящее P2P сообщение
#[derive(Debug, Clone)]
pub struct P2pMessage {
    /// libp2p PeerID отправителя — для роутинга (routing — маршрутизации)
    pub from_peer:   String,
    /// Ed25519 публичный ключ отправителя — для идентификации в UI
    /// Извлекается из подписи gossipsub — гарантированно правильный
    pub from_pubkey: String,
    pub content:     String,
    pub timestamp:   u64,
}
/// Информация об активном пире — для отображения в UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct PeerInfo {
    /// Полный PeerID строкой
    pub peer_id:      String,
    /// Протокол соединения: "TCP", "WS", "QUIC", "mDNS"
    pub protocol:     String,
    /// UNIX timestamp момента подключения — для сортировки
    pub connected_at: u64,
}

/// Команда управления узлом — отправляется из Tauri в swarm loop
#[derive(Debug)]
pub enum NodeCommand {
    /// Опубликовать сообщение в топик gossipsub
    Publish {
        topic:   String,
        content: String,
    },
    /// Подключиться к пиру по Multiaddr строке
    /// Пример: "/dns4/example.ngrok-free.app/tcp/443/wss/p2p/12D3..."
    Dial(String),
}

// ── Запуск узла ──────────────────────────────────────────────

/// Запустить P2P узел с каналами управления
///
/// Аргументы:
///   key_bytes       — Ed25519 приватный ключ (32 байта) из identity
///   shutdown_rx     — сигнал остановки
///   bootstrap_addrs — список Multiaddr строк для автодиала при старте
///
/// Возвращает:
///   tx_cmd    — канал команд (Publish / Dial)
///   rx_in     — входящие сообщения
///   rx_peers  — обновлённый список пиров при каждом изменении
pub async fn start_node_channels(
    key_bytes:       [u8; 32],
    shutdown_rx:     oneshot::Receiver<()>,
    bootstrap_addrs: Vec<String>,
) -> Result<
    (
        mpsc::Sender<NodeCommand>,
        mpsc::Receiver<P2pMessage>,
        mpsc::Receiver<Vec<PeerInfo>>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let (tx_cmd,  mut rx_cmd) = mpsc::channel::<NodeCommand>(100);
    let (tx_in,       rx_in)  = mpsc::channel::<P2pMessage>(100);
    let (tx_peers, rx_peers)  = mpsc::channel::<Vec<PeerInfo>>(32);

    let tx_in_clone = tx_in.clone();

    tokio::spawn(async move {
        // ── Keypair из стабильного seed ──────────────────────
        let secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(key_bytes)
            .expect("Неверный Ed25519 ключ");
        let ed_keypair = libp2p::identity::ed25519::Keypair::from(secret);
        let local_key  = libp2p::identity::Keypair::from(ed_keypair);
        let local_peer = local_key.public().to_peer_id();

        let gossipsub = build_gossipsub(&local_key);

        // ── SwarmBuilder: TCP + QUIC + WebSocket + DNS ───────
        // DNS нужен для резолва доменных имён в bootstrap адресах (ngrok)
        // WS нужен для подключения через HTTPS туннель ngrok (порт 443/wss)
        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .expect("TCP транспорт: ошибка инициализации")
            .with_quic()
            .with_dns()
            .expect("DNS резолвер: ошибка инициализации")
            // WS через noise (без TLS) — ngrok сам обеспечивает TLS снаружи
            .with_websocket(
                noise::Config::new,
                yamux::Config::default,
            )
            .await
            .expect("WebSocket транспорт: ошибка инициализации")
            .with_behaviour(|key: &libp2p::identity::Keypair| {
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;
                Ok(RootBehaviour { gossipsub, mdns })
            })
            .expect("Behaviour: ошибка инициализации")
            .with_swarm_config(|cfg: libp2p::swarm::Config| {
                cfg.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        // ── Подписка на основной топик и старт слушателя ─────
        let topic = gossipsub::IdentTopic::new(ROOT_TOPIC);
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        // Слушаем на случайном TCP порту (0 = OS выбирает)
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        // Слушаем WS на случайном порту
        swarm.listen_on("/ip4/0.0.0.0/tcp/0/ws".parse().unwrap()).unwrap();

        println!("  🚀 P2P узел запущен | PeerID: {}", local_peer);

        // ── Автодиал bootstrap адресов ───────────────────────
        // Диалим сразу при старте — не ждём команды из UI
        for addr_str in &bootstrap_addrs {
            match addr_str.parse::<libp2p::Multiaddr>() {
                Ok(addr) => {
                    match swarm.dial(addr.clone()) {
                        Ok(_)  => println!("  🔗 Bootstrap диал: {}", addr_str),
                        Err(e) => println!("  ⚠️  Bootstrap диал ошибка {}: {}", addr_str, e),
                    }
                }
                Err(e) => println!("  ❌ Неверный bootstrap адрес '{}': {}", addr_str, e),
            }
        }

        let mut shutdown = shutdown_rx;

        // Таблица активных пиров: peer_id → PeerInfo
        // Живёт внутри async задачи, не требует Arc<Mutex>
        let mut active_peers: std::collections::HashMap<String, PeerInfo> =
            std::collections::HashMap::new();

        // ── Вспомогательная функция: отправить обновлённый список ──
        // Определяем как closure — будем вызывать после каждого изменения
        macro_rules! emit_peers {
            () => {{
                let list: Vec<PeerInfo> = active_peers.values().cloned().collect();
                let _ = tx_peers.send(list).await;
            }};
        }

        // ── Главный event loop ───────────────────────────────
        loop {
            tokio::select! {
                // Сигнал остановки
                _ = &mut shutdown => {
                    println!("  🛑 P2P узел остановлен");
                    break;
                }

                // Команды из Tauri (Publish или Dial)
                Some(cmd) = rx_cmd.recv() => {
                    match cmd {
                        NodeCommand::Publish { topic: topic_str, content } => {
                            let topic = gossipsub::IdentTopic::new(&topic_str);
                            // Подписываемся на топик если ещё не подписаны
                            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
                            match swarm.behaviour_mut().gossipsub.publish(topic, content.as_bytes()) {
                                Ok(_)  => println!(
                                    "  📤 Отправлено в топик {}: {}",
                                    &topic_str[..topic_str.len().min(8)],
                                    &content[..content.len().min(50)]
                                ),
                                Err(e) => println!("  ❌ Ошибка отправки: {}", e),
                            }
                        }

                        NodeCommand::Dial(addr_str) => {
                            // Парсим Multiaddr — строгий формат
                            // Ошибку возвращаем в лог, UI узнает через dial_error событие
                            match addr_str.parse::<libp2p::Multiaddr>() {
                                Ok(addr) => {
                                    match swarm.dial(addr) {
                                        Ok(_)  => println!("  🔗 Диал запущен: {}", addr_str),
                                        Err(e) => println!("  ❌ Диал ошибка: {}", e),
                                    }
                                }
                                Err(e) => println!(
                                    "  ❌ Неверный Multiaddr '{}': {}",
                                    addr_str, e
                                ),
                            }
                        }
                    }
                }

                // События swarm
                event = swarm.select_next_some() => {
                    match event {

                        // ── Входящее gossipsub сообщение ──────
                        SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { propagation_source, message, .. }
                        )) => {
                            if !verify_message_sender(&propagation_source, &message) {
                                println!("  ⚠️ Сообщение отклонено — подмена отправителя");
                                continue;
                            }
                            let content   = String::from_utf8_lossy(&message.data).to_string();
                            let timestamp = now_unix();

                            // Извлекаем Ed25519 публичный ключ из PeerID отправителя
                            // to_bytes() возвращает байты multihash — из них достаём ключ
                            // Gossipsub::Strict уже проверил подпись — ключ гарантированно правильный
                            let from_pubkey = message.source
                                .as_ref()
                                .and_then(|peer_id| {
                                    libp2p::identity::PublicKey::try_decode_protobuf(
                                        &peer_id.to_bytes()
                                    ).ok()
                                })
                                .map(|pubkey| {
                                    let full = hex::encode(pubkey.encode_protobuf());
                                    // Убираем protobuf prefix "08011220" (4 байта)
                                    // Оставляем только 32 байта Ed25519 ключа
                                    if full.starts_with("08011220") {
                                        full[8..].to_string()
                                    } else {
                                        full
                                    }
                                })
                                .unwrap_or_else(|| propagation_source.to_string());

                            // Временный лог — удалим после отладки
                            println!("  🔑 from_pubkey: {}", &from_pubkey);
                            println!("  🔑 from_peer:   {}", &propagation_source);

                            let msg = P2pMessage {
                                from_peer:   propagation_source.to_string(),
                                from_pubkey,
                                content,
                                timestamp,
                            };
                            println!("  📨 Входящее от {}...", &msg.from_peer[..8.min(msg.from_peer.len())]);
                            let _ = tx_in_clone.send(msg).await;
                        }

                        // ── mDNS: найден локальный пир ────────
                        SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                            mdns::Event::Discovered(peers)
                        )) => {
                            for (peer_id, _addr) in peers {
                                println!("  🔍 mDNS пир: {}...", &peer_id.to_string()[..8]);
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                // Добавляем в таблицу с протоколом mDNS
                                active_peers.insert(peer_id.to_string(), PeerInfo {
                                    peer_id:      peer_id.to_string(),
                                    protocol:     "mDNS".to_string(),
                                    connected_at: now_unix(),
                                });
                            }
                            emit_peers!();
                        }

                        // ── mDNS: пир истёк ───────────────────
                        SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                            mdns::Event::Expired(peers)
                        )) => {
                            for (peer_id, _) in peers {
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                active_peers.remove(&peer_id.to_string());
                            }
                            emit_peers!();
                        }

                        // ── Соединение установлено ────────────
                        // endpoint содержит информацию о транспорте
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            let protocol = detect_protocol(endpoint.get_remote_address());
                            println!(
                                "  🤝 Подключён [{}]: {}...",
                                protocol,
                                &peer_id.to_string()[..8.min(peer_id.to_string().len())]
                            );
                            active_peers.insert(peer_id.to_string(), PeerInfo {
                                peer_id:      peer_id.to_string(),
                                protocol,
                                connected_at: now_unix(),
                            });
                            emit_peers!();
                        }

                        // ── Соединение закрыто ────────────────
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            println!(
                                "  🔌 Отключён: {}...",
                                &peer_id.to_string()[..8.min(peer_id.to_string().len())]
                            );
                            active_peers.remove(&peer_id.to_string());
                            emit_peers!();
                        }

                        // ── Ошибка диала ─────────────────────
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            println!(
                                "  ❌ Диал не удался {:?}: {}",
                                peer_id, error
                            );
                            // peer_id может быть None если адрес не содержал /p2p/...
                        }

                        // ── Новый адрес слушателя ─────────────
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("  🌐 Слушаем: {}/p2p/{}", address, local_peer);
                        }

                        _ => {}
                    }
                }
            }
        }
    });

    Ok((tx_cmd, rx_in, rx_peers))
}

// ── Вспомогательные функции ──────────────────────────────────

/// Текущее время в секундах UNIX
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Определяем тип протокола по Multiaddr удалённого конца
/// Используется для отображения бейджа в UI
fn detect_protocol(addr: &libp2p::Multiaddr) -> String {
    let addr_str = addr.to_string();
    if addr_str.contains("/wss") || addr_str.contains("/ws") {
        "WS".to_string()
    } else if addr_str.contains("/quic") {
        "QUIC".to_string()
    } else if addr_str.contains("/tcp") {
        "TCP".to_string()
    } else {
        "Unknown".to_string()
    }
}
