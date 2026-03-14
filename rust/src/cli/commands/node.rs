// ============================================================
// ROOT v2.0 — CLI: команды node (пользовательский режим)
// ============================================================

use clap::Subcommand;
use futures::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux, SwarmBuilder,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;

#[derive(Subcommand)]
pub enum NodeAction {
    /// Запустить узел и ждать подключений
    Listen,
    /// Подключиться к узлу по адресу
    Connect {
        /// Multiaddr: /ip4/1.2.3.4/tcp/7001/p2p/PEERID
        addr: String,
    },
    /// Показать количество пиров
    Status,
}

pub async fn run(action: NodeAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        NodeAction::Listen           => cmd_listen().await,
        NodeAction::Connect { addr } => cmd_connect(addr).await,
        NodeAction::Status           => { cmd_status(); Ok(()) }
    }
}

async fn cmd_listen() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запускаем P2P узел...");
    println!("📝 Введите сообщение + Enter для отправки");
    println!("   Ctrl+C для выхода\n");

    let (mut swarm, local_peer_id) = build_swarm()?;
    let topic = gossipsub::IdentTopic::new("root-network-v2");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("📋 Мой PeerID: {}", local_peer_id);

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = reader.next_line() => {
                if line.is_empty() { continue; }
                match swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                    Ok(_)  => println!("📤 Отправлено: {}", line),
                    Err(e) => println!("❌ Ошибка отправки: {}", e),
                }
            }
            event = swarm.select_next_some() => handle_event(&mut swarm, event),
        }
    }
}

async fn cmd_connect(target_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    use libp2p::Multiaddr;

    println!("🔗 Подключаемся к: {}", target_addr);
    println!("📝 Введите сообщение + Enter для отправки\n");

    let (mut swarm, local_peer_id) = build_swarm()?;
    let topic = gossipsub::IdentTopic::new("root-network-v2");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let addr: Multiaddr = target_addr.parse()?;
    swarm.dial(addr)?;

    println!("📋 Мой PeerID: {}", local_peer_id);
    println!("⏳ Подключаемся...\n");

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            Ok(Some(line)) = reader.next_line() => {
                if line.is_empty() { continue; }
                match swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                    Ok(_)  => println!("📤 Отправлено: {}", line),
                    Err(e) => println!("❌ Ошибка отправки: {}", e),
                }
            }
            event = swarm.select_next_some() => handle_event(&mut swarm, event),
        }
    }
}

fn cmd_status() {
    println!("ℹ️  Запусти 'root-cli node listen' чтобы увидеть статус пиров в реальном времени");
}

// ── Обработчик событий Swarm ─────────────────────────────────

fn handle_event(
    swarm: &mut libp2p::Swarm<NodeBehaviour>,
    event: SwarmEvent<NodeBehaviourEvent>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            let peer_id = swarm.local_peer_id();
            println!("📡 Адрес: {}/p2p/{}", address, peer_id);
            println!("   ↑ Скопируй в другой терминал: root-cli node connect <адрес>\n");
        }
        SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
            gossipsub::Event::Message { propagation_source, message, .. }
        )) => {
            let text = String::from_utf8_lossy(&message.data);
            println!("💬 [{}...]: {}", &propagation_source.to_string()[..8], text);
        }
        SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(
            mdns::Event::Discovered(peers)
        )) => {
            for (peer_id, _) in peers {
                println!("🔍 Найден узел: {}...", &peer_id.to_string()[..8]);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(
            mdns::Event::Expired(peers)
        )) => {
            for (peer_id, _) in peers {
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            println!("🤝 Подключился: {}...", &peer_id.to_string()[..8]);
        }
        SwarmEvent::ConnectionClosed { peer_id, .. } => {
            println!("🔌 Отключился: {}...", &peer_id.to_string()[..8]);
        }
        _ => {}
    }
}

// ── Построение Swarm ─────────────────────────────────────────

fn build_swarm() -> Result<
    (libp2p::Swarm<NodeBehaviour>, libp2p::PeerId),
    Box<dyn std::error::Error>,
> {
    let local_key     = libp2p::identity::Keypair::generate_ed25519();
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

    let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id(),
            )?;
            Ok(NodeBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(60))
        })
        .build();

    Ok((swarm, local_peer_id))
}

// ── Поведение узла ───────────────────────────────────────────
// Вынесено в один модуль — не дублируется с transport.rs

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct NodeBehaviour {
    pub gossipsub: libp2p::gossipsub::Behaviour,
    pub mdns:      libp2p::mdns::tokio::Behaviour,
}
