#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// ── Identity ─────────────────────────────────────────────────

#[tauri::command]
fn generate_identity() -> Result<root_ffi::api::types::IdentityInfo, String> {
    root_ffi::api::identity::generate_identity().map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_identity(mnemonic: String) -> Result<root_ffi::api::types::IdentityInfo, String> {
    root_ffi::api::identity::restore_identity(mnemonic).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_public_key() -> Result<String, String> {
    root_ffi::api::identity::get_public_key().map_err(|e| e.to_string())
}

#[tauri::command]
fn confirm_mnemonic() -> Result<(), String> {
    root_ffi::api::identity::confirm_mnemonic().map_err(|e| e.to_string())
}

// ── Database ─────────────────────────────────────────────────

#[tauri::command]
fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("root.db").to_string_lossy().to_string())
}

#[tauri::command]
fn unlock_database(password: String, db_path: String) -> Result<root_ffi::api::types::UnlockResult, String> {
    root_ffi::api::database::unlock_database(password, db_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn panic_button() -> Result<(), String> {
    root_ffi::api::database::panic_button()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn lock_database() -> Result<(), String> {
    root_ffi::api::database::lock_database()
        .map_err(|e| e.to_string())
}

// ── P2P — управление узлом ───────────────────────────────────

#[tauri::command]
fn get_p2p_status() -> bool {
    root_ffi::api::p2p::is_p2p_running()
}

#[tauri::command]
fn start_p2p_node() -> Result<String, String> {
    root_ffi::api::p2p::start_p2p_node()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_p2p_node() -> Result<(), String> {
    root_ffi::api::p2p::stop_p2p_node()
        .map_err(|e| e.to_string())
}

// ── P2P — пиры ───────────────────────────────────────────────

#[tauri::command]
fn get_peer_count() -> u32 {
    root_ffi::api::p2p::get_peer_count()
}

/// Список активных пиров с протоколами — для вкладки Сеть
#[tauri::command]
fn get_peers() -> Vec<root_ffi::api::types::PeerInfoDto> {
    root_ffi::api::p2p::get_peers()
}

/// Ручное подключение к пиру по Multiaddr
/// Пример addr: "/dns4/host.ngrok-free.app/tcp/443/wss/p2p/12D3..."
#[tauri::command]
fn dial_node(addr: String) -> Result<(), String> {
    root_ffi::api::p2p::dial_node(addr)
        .map_err(|e| e.to_string())
}

// ── P2P — bootstrap ──────────────────────────────────────────

/// Получить список bootstrap адресов из БД
#[tauri::command]
fn get_bootstrap_list() -> Result<Vec<String>, String> {
    root_ffi::api::p2p::get_bootstrap_list()
        .map_err(|e| e.to_string())
}

/// Сохранить список bootstrap адресов в БД
#[tauri::command]
fn save_bootstrap_list(addrs: Vec<String>) -> Result<(), String> {
    root_ffi::api::p2p::save_bootstrap_list(addrs)
        .map_err(|e| e.to_string())
}

// ── Contacts ─────────────────────────────────────────────────

#[tauri::command]
fn add_contact(public_key: String, nickname: String) -> Result<(), String> {
    root_ffi::api::contacts::add_contact(public_key, nickname)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_contacts() -> Result<Vec<root_ffi::api::types::ContactInfo>, String> {
    root_ffi::api::contacts::get_contacts()
        .map_err(|e| e.to_string())
}

// ── Messaging ────────────────────────────────────────────────

#[tauri::command]
fn get_incoming_messages() -> Vec<root_ffi::api::types::MessageInfo> {
    root_ffi::api::p2p::get_incoming_messages()
}

#[tauri::command]
fn send_message(to_key: String, content: String) -> Result<u64, String> {
    root_ffi::api::messaging::send_message(to_key, content)
        .map_err(|e| e.to_string())
}

// ── Utils ────────────────────────────────────────────────────

#[tauri::command]
fn get_version() -> String {
    root_ffi::api::utils::get_version()
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

// ── Main ─────────────────────────────────────────────────────

fn main() {
    let _ = env_logger::try_init();
    log::info!("🚀 ROOT Desktop v2.0 starting...");

    println!("🚀 TAURI APP STARTING (not CLI!)");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Гарантируем что папка для БД существует до первого вызова
            let _ = app.path().app_data_dir()
                .map(|p| std::fs::create_dir_all(p).ok());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Utils
            ping,
            get_version,
            get_db_path,
            // Identity
            generate_identity,
            restore_identity,
            get_public_key,
            confirm_mnemonic,
            // Database
            unlock_database,
            panic_button,
            lock_database,
            // P2P — узел
            get_p2p_status,
            start_p2p_node,
            stop_p2p_node,
            // P2P — пиры
            get_peer_count,
            get_peers,
            dial_node,
            // P2P — bootstrap
            get_bootstrap_list,
            save_bootstrap_list,
            // Messaging
            get_incoming_messages,
            send_message,
            // Contacts
            add_contact,
            get_contacts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
