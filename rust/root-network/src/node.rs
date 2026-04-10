// ============================================================
// ROOT v2.0 — network/node.rs
// Bootstrap нода — стабильный узел для первого соединения
//
// Что делает:
//   1. Читает Ed25519 ключ из файла keypair.json (или создаёт новый)
//   2. Запускает swarm с TCP + WS + DNS транспортами
//   3. Слушает на фиксированном порту 9000 (WS для ngrok)
//   4. Логирует все события — кто подключился, кто отключился
//
// Запуск:
//   cargo run --bin root-node
//
// Первый запуск создаст keypair.json рядом с бинарником.
// Не удаляй этот файл — иначе PeerID изменится и
// все сохранённые bootstrap адреса станут невалидными.
// ============================================================

use futures::StreamExt;
use libp2p::{
    SwarmBuilder, gossipsub, mdns, noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use std::{path::PathBuf, time::Duration};
use tracing::{info, warn};

use super::behaviour::{
    ROOT_TOPIC, RootBehaviour, RootBehaviourEvent, build_gossipsub,
};

// ── Путь к файлу ключа ───────────────────────────────────────

/// Возвращает путь к keypair.json рядом с исполняемым файлом.
/// Если бинарник в /home/user/root-node, то ключ в /home/user/keypair.json
fn keypair_path() -> PathBuf {
    // current_exe() — путь к самому бинарнику
    // parent() — директория где он лежит
    std::env::current_exe()
        .expect("Не удалось получить путь к бинарнику")
        .parent()
        .expect("Нет родительской директории")
        .join("keypair.json")
}

// ── Загрузка / создание ключа ────────────────────────────────

/// Загрузить ключ из файла или создать новый и сохранить.
///
/// Формат файла keypair.json:
///   { "secret_key_hex": "a1b2c3d4..." }  ← 32 байта в hex (64 символа)
///
/// Почему hex а не base64 — читаемо глазами, легко скопировать для бэкапа.
fn load_or_create_keypair(path: &PathBuf) -> libp2p::identity::Keypair {
    if path.exists() {
        // Файл есть — читаем
        info!("🔑 Загружаем ключ из {:?}", path);

        let json = std::fs::read_to_string(path)
            .expect("Не удалось прочитать keypair.json");

        // Парсим JSON вручную — не тянем serde_json для одного поля
        let value: serde_json::Value = serde_json::from_str(&json)
            .expect("keypair.json — неверный JSON");

        let hex_str = value["secret_key_hex"]
            .as_str()
            .expect("keypair.json — нет поля secret_key_hex");

        // hex → байты → Ed25519 ключ
        let bytes = hex::decode(hex_str)
            .expect("keypair.json — secret_key_hex не является валидным hex");

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes[..32]);

        let secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(key_bytes)
            .expect("Неверный Ed25519 ключ в keypair.json");

        let ed_keypair = libp2p::identity::ed25519::Keypair::from(secret);
        libp2p::identity::Keypair::from(ed_keypair)

    } else {
        // Файла нет — генерируем новый ключ и сохраняем
        info!("🆕 Создаём новый keypair и сохраняем в {:?}", path);

        // Генерируем случайный Ed25519 ключ — только один раз!
        let keypair = libp2p::identity::Keypair::generate_ed25519();

        // Извлекаем байты секретного ключа для сохранения
        let ed_keypair = keypair.clone()
            .try_into_ed25519()
            .expect("Keypair не является Ed25519");

        // to_bytes() возвращает 64 байта: [secret(32) | public(32)]
        // Нам нужны только первые 32 — секретная часть
        let secret_bytes = ed_keypair.secret().as_ref().to_vec();
        let hex_str = hex::encode(&secret_bytes);

        // Сохраняем в JSON
        let json = serde_json::json!({
            "secret_key_hex": hex_str,
            // Публичный ключ для справки — при загрузке не используется
            "peer_id": keypair.public().to_peer_id().to_string(),
            // Предупреждение прямо в файле
            "_warning": "Не удаляй этот файл! PeerID изменится и bootstrap адрес станет невалидным."
        });

        std::fs::write(path, serde_json::to_string_pretty(&json).unwrap())
            .expect("Не удалось сохранить keypair.json");

        info!("💾 Ключ сохранён");
        keypair
    }
}

// ── Запуск bootstrap ноды ────────────────────────────────────

/// Запустить bootstrap ноду.
///
/// Эта функция блокирующая — работает пока процесс не убьют (Ctrl+C).
/// Предназначена для запуска как отдельный бинарник, не внутри Tauri.
pub async fn start_node() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализируем tracing — логи в stdout с временными метками
    tracing_subscriber::fmt::init();

    info!("🚀 ROOT Bootstrap нода запускается...");

    // ── Загружаем стабильный ключ ────────────────────────────
    let path = keypair_path();
    let local_key = load_or_create_keypair(&path);
    let local_peer = local_key.public().to_peer_id();

    info!("📋 PeerID: {}", local_peer);
    info!("📋 Сохрани этот PeerID — он нужен для bootstrap адреса");

    // ── Строим Gossipsub ─────────────────────────────────────
    let gossipsub = build_gossipsub(&local_key);

    // ── Строим Swarm ─────────────────────────────────────────
    // TCP — для прямых соединений в локальной сети
    // QUIC — быстрый протокол поверх UDP
    // DNS — резолвим доменные имена (нужно для подключения к нам через ngrok)
    // WS — WebSocket через ngrok туннель
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        // WS через noise — ngrok сам терминирует TLS снаружи
        .with_websocket(
            noise::Config::new,
            yamux::Config::default,
        )
        .await?
        .with_behaviour(|key| {
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id(),
            )?;
            Ok(RootBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|cfg| {
            // Держим соединения живыми дольше — bootstrap нода не должна
            // отключать пиров которые долго молчат
            cfg.with_idle_connection_timeout(Duration::from_secs(300))
        })
        .build();

    // ── Подписка на топик ────────────────────────────────────
    let topic = gossipsub::IdentTopic::new(ROOT_TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // ── Слушаем на фиксированных портах ─────────────────────
    // Порт 9000 — основной WS порт который туннелирует ngrok
    // Порт 9001 — TCP для прямых соединений (опционально)
    swarm.listen_on("/ip4/0.0.0.0/tcp/9001".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/9000/ws".parse()?)?;

    info!("👂 Слушаем WS на порту 9000, TCP на порту 9001");
    info!("🌐 Bootstrap адрес будет выведен ниже после старта...");

    // ── Главный цикл событий ─────────────────────────────────
    loop {
        match swarm.select_next_some().await {

            // Новый адрес слушателя — выводим полный bootstrap адрес
            SwarmEvent::NewListenAddr { address, .. } => {
                // Полный multiaddr включает /p2p/<PeerID> в конце
                info!("📡 Слушаем: {}/p2p/{}", address, local_peer);

                // Если это WS адрес — напоминаем как настроить ngrok
                if address.to_string().contains("/ws") {
                    info!("👆 Этот адрес туннелируй через ngrok:");
                    info!("   ngrok http 9000");
                    info!("   Потом замени адрес в настройках приложения");
                }
            }

            // Кто-то подключился
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!(
                    "🤝 Подключился: {} через {}",
                    peer_id,
                    endpoint.get_remote_address()
                );
            }

            // Кто-то отключился
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                warn!("🔌 Отключился: {} причина: {:?}", peer_id, cause);
            }

            // Входящее сообщение — bootstrap нода его логирует но не обрабатывает
            SwarmEvent::Behaviour(RootBehaviourEvent::Gossipsub(
                gossipsub::Event::Message { propagation_source, message, .. }
            )) => {
                let text = String::from_utf8_lossy(&message.data);
                info!("📨 Сообщение от {}: {}", propagation_source, &text[..text.len().min(80)]);
            }

            // mDNS — локальные пиры (на bootstrap ноде обычно никого нет)
            SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                mdns::Event::Discovered(peers)
            )) => {
                for (peer_id, addr) in peers {
                    info!("🔍 mDNS пир: {} @ {}", peer_id, addr);
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
            }

            SwarmEvent::Behaviour(RootBehaviourEvent::Mdns(
                mdns::Event::Expired(peers)
            )) => {
                for (peer_id, _) in peers {
                    warn!("👋 mDNS пир ушёл: {}", peer_id);
                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                }
            }

            // Ошибка исходящего соединения
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                warn!("❌ Не удалось подключиться к {:?}: {}", peer_id, error);
            }

            _ => {}
        }
    }
}
