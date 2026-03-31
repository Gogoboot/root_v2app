// ============================================================
// ROOT v2.0 — network/behaviour.rs
// RootBehaviour — общее поведение P2P узла
// Используется и в Flutter (channels.rs) и в CLI (node.rs)
// ============================================================

use libp2p::{gossipsub, mdns};
use std::time::Duration;
use sha2::{Sha256, Digest};

/// Поведение узла ROOT — Gossipsub + mDNS
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct RootBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// Событие от RootBehaviour
pub use libp2p::swarm::derive_prelude::*;

/// Создать Gossipsub с настройками ROOT
pub fn build_gossipsub(local_key: &libp2p::identity::Keypair) -> gossipsub::Behaviour {
    // SHA-256 вместо DefaultHasher — стабильный ID между перезапусками (S2-T3)
    let message_id_fn = |message: &gossipsub::Message| {
        let mut hasher = Sha256::new();
        hasher.update(&message.data);
        let hash = hasher.finalize();
        // Берём первые 8 байт как ID
        gossipsub::MessageId::from(hex::encode(&hash[..8]))
    };

    let config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .message_id_fn(message_id_fn)
        .mesh_n_low(4)
        .mesh_n(6)
        .mesh_n_high(12)
        .max_transmit_size(1024 * 1024)
        // Strict — Gossipsub проверяет подпись каждого сообщения (S1-T6)
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Ошибка конфигурации Gossipsub");

    gossipsub::Behaviour::new(
        // Signed — каждое сообщение подписывается нашим ключом
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        config,
    )
    .expect("Ошибка создания Gossipsub")
}

/// Проверить что отправитель сообщения совпадает с заявленным from_key
/// Вызывается в channels.rs при получении входящего сообщения
pub fn verify_message_sender(
    propagation_source: &libp2p::PeerId,
    message: &gossipsub::Message,
) -> bool {
    // Если source из сообщения совпадает с тем кто его прислал — всё чисто
    // Gossipsub::Strict уже проверил подпись, нам остаётся проверить source
    match &message.source {
        Some(source) => source == propagation_source,
        // Нет source — сообщение анонимное, не принимаем
        None => {
            println!("  ⚠️ Отклонено анонимное сообщение");
            false
        }
    }
}

/// Генерировать имя приватного топика для пары пользователей.
/// Топик одинаковый с обеих сторон — ключи сортируются перед хешированием.
///
/// Пример: hash("alice_pubkey:bob_pubkey") → "a3f9c2b1e8d4..."
pub fn private_topic(pubkey_a: &str, pubkey_b: &str) -> String {
    let mut keys = [pubkey_a, pubkey_b];
    keys.sort(); // hash(A,B) == hash(B,A)

    let mut hasher = Sha256::new();
    hasher.update(keys[0].as_bytes());
    hasher.update(b":");
    hasher.update(keys[1].as_bytes());
    let hash = hasher.finalize();
    hex::encode(&hash[..16])
}

/// Название топика сети ROOT
pub const ROOT_TOPIC: &str = "root-network-v2";
