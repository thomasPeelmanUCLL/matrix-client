use matrix_sdk::{
    config::SyncSettings,
    Client,
    room::MessagesOptions,
    ruma::{
        OwnedRoomId,
        events::room::message::RoomMessageEventContent,
    },
};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager};
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

mod state;
mod auth;
mod sync_mod;
mod rooms;
mod messages;
mod verification;

pub use state::*;
pub use auth::*;
pub use sync_mod::*;
pub use rooms::*;
pub use messages::*;
pub use verification::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;
            println!("Using data directory: {:?}", data_dir);
            app.manage(MatrixState::new(data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            matrix_login,
            check_session,
            logout,
            matrix_sync,
            get_rooms,
            get_messages,
            send_message,
            check_verification_status,
            request_verification,
            get_verification_emoji,
            confirm_verification,
            cancel_verification,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
