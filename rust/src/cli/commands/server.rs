// ============================================================
// ROOT v2.0 — CLI: bootstrap сервер (VPS режим)
// ============================================================

use clap::Subcommand;
use futures::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux, SwarmBuilder,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Subcommand)]
pub enum ServerAction {
    /// Запустить bootstrap сервер
    Start {
        /// Порт для прослушивания (по умолчанию 7001)
        #[arg(short, long, default_value = "7001")]
        port: u16,
    },
    /// Показать статус сервера
    Status,
}

pub async fn run(action: ServerAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ServerAction::Start { port } => cmd_server_start(port).await,
        ServerAction::Status         => { cmd_server_status(); Ok(()) }
    }
}

async fn cmd_server_start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Bootstrap сервер ROOT v2.0");
    println!("📡 Порт: {}", port);
    println!("   Ctrl+C для остановки\n");

    let local_key    = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();

    let message_id_fn = |message: &gossipsub::Message| {
        let mut hasher = DefaultHasher::new();
        message.data.hash(&mut hasher);
        gossipsub::MessageId::from(hasher.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .message_id_fn(message_id_fn)
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id(),
            )?;
            Ok(ServerBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(300))
        })
        .build();

    let topic = gossipsub::IdentTopic::new("root-network-v2");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let addr = format!("/ip4/0.0.0.0/tcp/{}", port);
    swarm.listen_on(addr.parse()?)?;

    println!("🔑 PeerID: {}", local_peer_id);
    println!("👂 Ожидаем подключений...\n");

    let mut peer_count = 0u32;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("📡 Адрес: {}/p2p/{}", address, local_peer_id);
                println!("   ↑ Добавь этот адрес как bootstrap в настройках ROOT\n");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                peer_count += 1;
                println!("🤝 [{}] Подключился: {}... | всего пиров: {}",
                    timestamp(), &peer_id.to_string()[..8], peer_count);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                peer_count = peer_count.saturating_sub(1);
                println!("🔌 [{}] Отключился: {}... | всего пиров: {}",
                    timestamp(), &peer_id.to_string()[..8], peer_count);
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
            }
            SwarmEvent::Behaviour(ServerBehaviourEvent::Gossipsub(
                gossipsub::Event::Message { propagation_source, message, .. }
            )) => {
                let size = message.data.len();
                println!("📨 [{}] Relay: {} байт от {}...",
                    timestamp(), size, &propagation_source.to_string()[..8]);
            }
            SwarmEvent::Behaviour(ServerBehaviourEvent::Mdns(
                mdns::Event::Discovered(peers)
            )) => {
                for (peer_id, _) in peers {
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
            }
            _ => {}
        }
    }
}

fn cmd_server_status() {
    println!("ℹ️  Статус сервера доступен через:");
    println!("   curl http://localhost:7001/status  (если реализован HTTP API)");
    println!("   или запусти: root-cli server start --port 7001");
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{:02}:{:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60, secs % 60)
}

// ── Поведение узла ───────────────────────────────────────────

#[derive(libp2p::swarm::NetworkBehaviour)]
struct ServerBehaviour {
    gossipsub: libp2p::gossipsub::Behaviour,
    mdns:      libp2p::mdns::tokio::Behaviour,
}
