use matrix_sdk::{config::SyncSettings, Client, ruma::{OwnedRoomId, events::room::message::RoomMessageEventContent}};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MatrixState {
    pub client: Arc<Mutex<Option<Client>>>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomInfo {
    room_id: String,
    name: Option<String>,
    topic: Option<String>,
}

#[tauri::command]
async fn matrix_login(
    state: State<'_, MatrixState>,
    homeserver: String,
    username: String,
    password: String,
) -> Result<String, String> {
    let client = Client::builder()
        .homeserver_url(&homeserver)
        .build()
        .await
        .map_err(|e| e.to_string())?;

    client
        .matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("Matrix Client")
        .await
        .map_err(|e| e.to_string())?;

    *state.client.lock().await = Some(client.clone());

    Ok(format!("Logged in as {}", username))
}

#[tauri::command]
async fn matrix_sync(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| e.to_string())?;

    Ok("Synced successfully".to_string())
}

#[tauri::command]
async fn get_rooms(state: State<'_, MatrixState>) -> Result<Vec<RoomInfo>, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    let mut rooms_info = Vec::new();
    
    for room in client.rooms() {
        let name = room.display_name().await.ok().map(|n| n.to_string());
        let topic = room.topic();
        
        rooms_info.push(RoomInfo {
            room_id: room.room_id().to_string(),
            name,
            topic,
        });
    }

    Ok(rooms_info)
}

#[tauri::command]
async fn send_message(
    state: State<'_, MatrixState>,
    room_id: String,
    message: String,
) -> Result<String, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    let room_id: OwnedRoomId = room_id.parse().map_err(|e| format!("Invalid room ID: {}", e))?;
    let room = client.get_room(&room_id).ok_or("Room not found")?;

    // Use the new API for sending text messages
    let content = RoomMessageEventContent::text_plain(&message);
    room.send(content).await.map_err(|e| e.to_string())?;

    Ok("Message sent".to_string())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MatrixState {
            client: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            matrix_login,
            matrix_sync,
            get_rooms,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
