// ============================================================
// ROOT v2.0 — network/channels.rs
// P2P узел с каналами — вызывается из api/p2p.rs при старте Flutter
// ============================================================

use futures::StreamExt;
use libp2p::{SwarmBuilder, gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};

use super::behaviour::{ROOT_TOPIC, RootBehaviour, RootBehaviourEvent, build_gossipsub, verify_message_sender};

/// Входящее P2P сообщение — передаётся во Flutter
#[derive(Debug, Clone)]
pub struct P2pMessage {
    pub from_peer: String,
    pub content:   String,
    pub timestamp: u64,
}

/// Исходящее P2P сообщение с указанием топика
#[derive(Debug, Clone)]
pub struct P2pOutMessage {
    /// Имя топика — приватный хеш или общий ROOT_TOPIC
    pub topic:   String,
    /// Содержимое сообщения
    pub content: String,
}

/// Запустить P2P узел с каналами
///
/// Возвращает три канала:
///   tx_out        — Flutter отправляет текст в сеть
///   rx_in         — Flutter получает входящие сообщения
///   rx_peer_count — Flutter получает обновления счётчика пиров
pub async fn start_node_channels(
    key_bytes: [u8; 32],
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<
    (mpsc::Sender<P2pOutMessage>, mpsc::Receiver<P2pMessage>, mpsc::Receiver<u32>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let (tx_out, mut rx_out) = mpsc::channel::<P2pOutMessage>(100);
    let (tx_in, rx_in)               = mpsc::channel::<P2pMessage>(100);
    // Новый канал — отправляет u32 (новое значение счётчика) при каждом изменении
    let (tx_peers, rx_peer_count)    = mpsc::channel::<u32>(32);
    let tx_in_clone                  = tx_in.clone();

    tokio::spawn(async move {
        let secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(key_bytes)
            .expect("Неверный Ed25519 ключ");
        let ed_keypair = libp2p::identity::ed25519::Keypair::from(secret);
        let local_key  = libp2p::identity::Keypair::from(ed_keypair);
        let local_peer = local_key.public().to_peer_id();

        let gossipsub = build_gossipsub(&local_key);

        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)
            .expect("TCP error")
            .with_quic()
            .with_behaviour(|key| {
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;
                Ok(RootBehaviour { gossipsub, mdns })
            })
            .expect("Behaviour error")
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let topic = gossipsub::IdentTopic::new(ROOT_TOPIC);
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        println!("  🚀 P2P узел запущен | PeerID: {}", local_peer);

        let mut shutdown = shutdown_rx;
        // Локальный счётчик — живёт внутри async задачи
        let mut peer_count: u32 = 0;

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    println!("  🛑 P2P узел остановлен");
                    break;
                }

                    Some(msg) = rx_out.recv() => {
                        let topic = gossipsub::IdentTopic::new(&msg.topic);
                        // Подписываемся на топик если ещё не подписаны
                        let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
                        match swarm.behaviour_mut().gossipsub.publish(topic, msg.content.as_bytes()) {
                            Ok(_)  => println!("  📤 P2P отправлено в топик {}: {}", &msg.topic[..8], &msg.content[..msg.content.len().min(50)]),
                            Err(e) => println!("  ❌ Ошибка отправки: {}", e),
                        }
                    }

                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { propagation_source, message, .. }
                        )) => {
                            if !verify_message_sender(&propagation_source, &message) {
                                println!("  ⚠️ Сообщение отклонено — подмена отправителя");
                                continue;
                            }
                            let content   = String::from_utf8_lossy(&message.data).to_string();
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let msg = P2pMessage { from_peer: propagation_source.to_string(), content, timestamp };
                            println!("  📨 Входящее от {}...", &msg.from_peer[..8]);
                            let _ = tx_in_clone.send(msg).await;
                        }

                        SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                            for (peer_id, _addr) in peers {
                                println!("  🔍 Найден узел: {}...", &peer_id.to_string()[..8]);
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            }
                        }

                        SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                            for (peer_id, _) in peers {
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                        }

                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("  🌐 Flutter P2P: {}/p2p/{}", address, local_peer);
                        }

                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("  🤝 Подключён: {}...", &peer_id.to_string()[..8]);
                            // Увеличиваем счётчик и отправляем новое значение
                            peer_count = peer_count.saturating_add(1);
                            let _ = tx_peers.send(peer_count).await;
                        }

                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            println!("  🔌 Отключён: {}...", &peer_id.to_string()[..8]);
                            // Уменьшаем счётчик — saturating_sub не уйдёт ниже 0
                            peer_count = peer_count.saturating_sub(1);
                            let _ = tx_peers.send(peer_count).await;
                        }

                        _ => {}
                    }
                }
            }
        }
    });

    Ok((tx_out, rx_in, rx_peer_count))
}
