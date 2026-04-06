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

// ── Database ─────────────────────────────────────────────────
#[tauri::command]
fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("root.db").to_string_lossy().to_string())
}

#[tauri::command]
fn unlock_database(password: String, db_path: String) -> Result<bool, String> {
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

// ── P2P ──────────────────────────────────────────────────────
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

#[tauri::command]
fn get_peer_count() -> u32 {
    root_ffi::api::p2p::get_peer_count()
}

#[tauri::command]
fn get_incoming_messages() -> Vec<root_ffi::api::types::MessageInfo> {
    root_ffi::api::p2p::get_incoming_messages()
}

// ── Messaging ────────────────────────────────────────────────
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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Гарантируем, что папка для БД существует до первого вызова
            let _ = app.path().app_data_dir()
                .map(|p| std::fs::create_dir_all(p).ok());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            get_version,
            get_db_path,
            generate_identity,
            restore_identity,
            get_public_key,
            unlock_database,
            panic_button,
            lock_database,
            get_p2p_status,
            start_p2p_node,
            stop_p2p_node,
            get_peer_count,
            get_incoming_messages,
            send_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
