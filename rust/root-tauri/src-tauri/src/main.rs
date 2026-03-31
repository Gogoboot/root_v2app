// ============================================================
// ROOT v2.0 — Tauri Desktop Application
// ============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


// ============================================================
// Команды для фронтенда (Tauri Commands)
// ============================================================

/// Простая тестовая команда
#[tauri::command]
fn ping() -> String {
    log::info!("🏓 Ping received from frontend");
    "pong".to_string()
}

/// Получить статус P2P узла
#[tauri::command]
fn get_p2p_status() -> bool {
    root_ffi::api::p2p::is_p2p_running()
}

/// Запустить P2P узел
#[tauri::command]
fn start_p2p_node() -> Result<String, String> {
    root_ffi::api::p2p::start_p2p_node()
        .map_err(|e: root_ffi::api::types::ApiError| e.to_string())
}

/// Остановить P2P узел
#[tauri::command]
fn stop_p2p_node() -> Result<(), String> {
    root_ffi::api::p2p::stop_p2p_node()
        .map_err(|e: root_ffi::api::types::ApiError| e.to_string())
}

/// Отправить сообщение
#[tauri::command]
fn send_message(to_key: String, content: String) -> Result<u64, String> {
    root_ffi::api::messaging::send_message(to_key, content)
        .map_err(|e: root_ffi::api::types::ApiError| e.to_string())
}

/// Получить входящие сообщения
#[tauri::command]
fn get_incoming_messages() -> Vec<root_ffi::api::types::MessageInfo> {
    root_ffi::api::p2p::get_incoming_messages()
}

/// Получить количество пиров
#[tauri::command]
fn get_peer_count() -> u32 {
    root_ffi::api::p2p::get_peer_count()
}

// ============================================================
// Главное приложение
// ============================================================

fn main() {
    // Инициализация логирования
    let _ = env_logger::try_init();
    
    log::info!("🚀 ROOT Desktop v2.0 starting...");

    // Сборка приложения
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ping,
            get_p2p_status,
            start_p2p_node,
            stop_p2p_node,
            send_message,
            get_incoming_messages,
            get_peer_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
