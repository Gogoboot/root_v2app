// ============================================================
// ROOT v2.0 — network/node.rs
// start_node — интерактивный режим (CLI / тестирование)
// ============================================================

use futures::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux, SwarmBuilder,
};
use std::time::Duration;
use tokio::select;
use tracing::{info, warn};

use super::behaviour::{build_gossipsub, RootBehaviour, RootBehaviourEvent, ROOT_TOPIC};

/// Запустить интерактивный P2P узел
/// Читает сообщения из stdin, выводит входящие в stdout
pub async fn start_node() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🚀 Запуск ROOT узла...");

    let local_key     = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    info!("📋 Мой PeerId: {}", local_peer_id);

    let gossipsub = build_gossipsub(&local_key);

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id(),
            )?;
            Ok(RootBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(60))
        })
        .build();

    let topic = gossipsub::IdentTopic::new(ROOT_TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    info!("👂 Слушаем входящие соединения...");

    loop {
        select! {
            line = read_stdin() => {
                if let Ok(msg) = line {
                    if msg.is_empty() { continue; }
                    match swarm.behaviour_mut().gossipsub.publish(topic.clone(), msg.as_bytes()) {
                        Ok(id)  => info!("📤 Отправлено: {:?}", id),
                        Err(e)  => warn!("❌ Ошибка отправки: {e}"),
                    }
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source, message, .. }
                    )) => {
                        let text = String::from_utf8_lossy(&message.data);
                        info!("📨 от {}: {}", propagation_source, text);
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                        mdns::Event::Discovered(peers)
                    )) => {
                        for (peer_id, addr) in peers {
                            info!("🔍 Найден: {} @ {}", peer_id, addr);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                        mdns::Event::Expired(peers)
                    )) => {
                        for (peer_id, _) in peers {
                            warn!("👋 Отключился: {}", peer_id);
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("📡 Адрес: {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("🤝 Соединение: {}", peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        warn!("🔌 Закрыто: {} причина: {:?}", peer_id, cause);
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn read_stdin() -> Result<String, tokio::io::Error> {
    use tokio::io::AsyncBufReadExt;
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut line   = String::new();
    reader.read_line(&mut line).await?;
    Ok(line.trim().to_string())
}
