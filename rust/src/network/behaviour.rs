// ============================================================
// ROOT v2.0 — network/behaviour.rs
// RootBehaviour — общее поведение P2P узла
// Используется и в Flutter (channels.rs) и в CLI (node.rs)
// ============================================================

use libp2p::{gossipsub, mdns};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// Поведение узла ROOT — Gossipsub + mDNS
/// Вынесено в отдельный файл чтобы не дублировать в CLI и Flutter
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct RootBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// Создать Gossipsub с настройками ROOT
pub fn build_gossipsub(local_key: &libp2p::identity::Keypair) -> gossipsub::Behaviour {
    let message_id_fn = |message: &gossipsub::Message| {
        let mut hasher = DefaultHasher::new();
        message.data.hash(&mut hasher);
        gossipsub::MessageId::from(hasher.finish().to_string())
    };

    let config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .message_id_fn(message_id_fn)
        .mesh_n_low(4)
        .mesh_n(6)
        .mesh_n_high(12)
        .max_transmit_size(1024 * 1024)
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Ошибка конфигурации Gossipsub");

    gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        config,
    )
    .expect("Ошибка создания Gossipsub")
}

/// Название топика сети ROOT
pub const ROOT_TOPIC: &str = "root-network-v2";
