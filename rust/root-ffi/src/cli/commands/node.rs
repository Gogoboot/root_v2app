// ============================================================
// ROOT v2.0 — CLI: команды node (пользовательский режим)
// ============================================================

use clap::Subcommand;
use futures::StreamExt;
use libp2p::{SwarmBuilder, gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use root_network::behaviour::{RootBehaviour, RootBehaviourEvent, build_gossipsub, verify_message_sender};

#[derive(Subcommand)]
pub enum NodeAction {
    /// Запустить узел и ждать подключений (bootstrap режим)
    Listen {
        /// Порт для прослушивания (по умолчанию 7001)
        #[arg(long, default_value = "7001")]
        port: u16,
    },
    /// Подключиться к узлу по адресу
    Connect {
        /// Multiaddr: /ip4/1.2.3.4/tcp/7001/p2p/PEERID
        addr: String,
    },
    /// Показать статус
    Status,
}

pub async fn run(action: NodeAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        NodeAction::Listen { port } => cmd_listen(port).await,
        NodeAction::Connect { addr } => cmd_connect(addr).await,
        NodeAction::Status => {
            cmd_status();
            Ok(())
        }
    }
}

async fn cmd_listen(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запускаем bootstrap узел на порту {}...", port);
    println!("📝 Введите сообщение + Enter для отправки");
    println!("   Ctrl+C для выхода\n");

    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    let gossipsub = build_gossipsub(&local_key);

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(RootBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new("root-network-v2");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Фиксированный порт — нужен для ngrok туннеля
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

    println!("📋 Мой PeerID: {}", local_peer_id);
    println!("🌐 Для ngrok: ngrok tcp {}\n", port);

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
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("📡 Адрес: {}/p2p/{}", address, local_peer_id);
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source, message, .. }
                    )) => {
                        // Проверяем подпись отправителя
                        if verify_message_sender(&propagation_source, &message) {
                            let text = String::from_utf8_lossy(&message.data);
                            println!("💬 [{}...]: {}", &propagation_source.to_string()[..8], text);
                        }
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, _) in peers {
                            println!("🔍 Найден: {}...", &peer_id.to_string()[..8]);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
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
        }
    }
}

async fn cmd_connect(target_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    use libp2p::Multiaddr;

    println!("🔗 Подключаемся к: {}", target_addr);
    println!("📝 Введите сообщение + Enter для отправки\n");

    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    let gossipsub = build_gossipsub(&local_key);

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(RootBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

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
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("📡 Мой адрес: {}/p2p/{}", address, local_peer_id);
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source, message, .. }
                    )) => {
                        if verify_message_sender(&propagation_source, &message) {
                            let text = String::from_utf8_lossy(&message.data);
                            println!("💬 [{}...]: {}", &propagation_source.to_string()[..8], text);
                        }
                    }
                    SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, _) in peers {
                            println!("🔍 Найден: {}...", &peer_id.to_string()[..8]);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
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
        }
    }
}

fn cmd_status() {
    println!("ℹ️  Запусти 'root-cli node listen' чтобы увидеть статус пиров");
}
