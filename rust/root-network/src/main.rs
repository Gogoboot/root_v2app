// ============================================================
// ROOT v2.0 — Bootstrap нода, точка входа
//
// Запуск:  cargo run -p root-network --bin root-node
// Сборка:  cargo build -p root-network --bin root-node
//
// При первом запуске создаст keypair.json рядом с бинарником.
// Не удаляй keypair.json — иначе PeerID изменится!
// ============================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    root_network::node::start_node().await
}
