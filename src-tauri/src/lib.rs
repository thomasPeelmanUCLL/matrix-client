use matrix_sdk::{
    config::SyncSettings, 
    Client, 
    ruma::{
        OwnedRoomId, 
        events::room::message::RoomMessageEventContent,
    }
};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MatrixState {
    pub client: Arc<Mutex<Option<Client>>>,
    pub user_id: Arc<Mutex<Option<String>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomInfo {
    room_id: String,
    name: Option<String>,
    topic: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    sender: String,
    body: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    success: bool,
    user_id: String,
    message: String,
}

#[tauri::command]
async fn matrix_login(
    state: State<'_, MatrixState>,
    homeserver: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    // Validate inputs
    if homeserver.is_empty() {
        return Err("Homeserver URL is required".to_string());
    }
    if username.is_empty() {
        return Err("Username is required".to_string());
    }
    if password.is_empty() {
        return Err("Password is required".to_string());
    }

    // Validate homeserver URL format
    if !homeserver.starts_with("http://") && !homeserver.starts_with("https://") {
        return Err("Homeserver URL must start with http:// or https://".to_string());
    }

    // Create client
    let client = Client::builder()
        .homeserver_url(&homeserver)
        .build()
        .await
        .map_err(|e| format!("Failed to connect to homeserver: {}", e))?;

    // Attempt login
    let response = client
        .matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("Matrix Client")
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    let user_id = response.user_id.to_string();

    // Store client and user ID
    *state.client.lock().await = Some(client.clone());
    *state.user_id.lock().await = Some(user_id.clone());

    Ok(LoginResponse {
        success: true,
        user_id: user_id.clone(),
        message: format!("Successfully logged in as {}", user_id),
    })
}

#[tauri::command]
async fn check_session(state: State<'_, MatrixState>) -> Result<Option<String>, String> {
    let user_id = state.user_id.lock().await;
    Ok(user_id.clone())
}

#[tauri::command]
async fn logout(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_lock = state.client.lock().await;
    
    if let Some(client) = client_lock.as_ref() {
        client.logout().await.map_err(|e| e.to_string())?;
    }

    drop(client_lock);
    
    *state.client.lock().await = None;
    *state.user_id.lock().await = None;

    Ok("Logged out successfully".to_string())
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
async fn get_messages(
    state: State<'_, MatrixState>,
    room_id: String,
    _limit: u32,
) -> Result<Vec<Message>, String> {
    let _client_lock = state.client.lock().await;
    let _client = _client_lock.as_ref().ok_or("Not logged in")?;

    Ok(vec![])
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
            user_id: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            matrix_login,
            check_session,
            logout,
            matrix_sync,
            get_rooms,
            get_messages,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
