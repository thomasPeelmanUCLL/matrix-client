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
use tauri::State;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct MatrixState {
    pub client: Arc<RwLock<Option<Client>>>,
    pub user_id: Arc<RwLock<Option<String>>>,
    pub pagination_tokens: Arc<RwLock<HashMap<String, String>>>, // room_id -> token
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    room_id: String,
    name: Option<String>,
    topic: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    sender: String,
    body: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    success: bool,
    user_id: String,
    device_id: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct MessagesResponse {
    messages: Vec<Message>,
    has_more: bool,
    next_token: Option<String>, // Add this field
}



#[tauri::command]
async fn matrix_login(
    state: State<'_, MatrixState>,
    homeserver: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    if homeserver.trim().is_empty() || username.trim().is_empty() || password.is_empty() {
        return Err("All fields are required".to_string());
    }

    if !homeserver.starts_with("http://") && !homeserver.starts_with("https://") {
        return Err("Homeserver URL must start with http:// or https://".to_string());
    }

    let client = Client::builder()
        .homeserver_url(homeserver.trim())
        .build()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let response = client
        .matrix_auth()
        .login_username(username.trim(), &password)
        .initial_device_display_name("Matrix Client (Rust)")
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    let user_id = response.user_id.to_string();
    let device_id = response.device_id.to_string();

    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| format!("Initial sync failed: {}", e))?;

    *state.client.write().await = Some(client);
    *state.user_id.write().await = Some(user_id.clone());

    Ok(LoginResponse {
        success: true,
        user_id,
        device_id,
        message: "Login successful".to_string(),
    })
}

#[tauri::command]
async fn check_session(state: State<'_, MatrixState>) -> Result<Option<String>, String> {
    let user_id = state.user_id.read().await;
    Ok(user_id.clone())
}

#[tauri::command]
async fn logout(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_read = state.client.read().await;
    
    if let Some(client) = client_read.as_ref() {
        client.logout().await.map_err(|e| e.to_string())?;
    }
    drop(client_read);

    *state.client.write().await = None;
    *state.user_id.write().await = None;

    Ok("Logged out successfully".to_string())
}

#[tauri::command]
async fn matrix_sync(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    println!("Starting sync..."); // DEBUG

    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    println!("Sync completed"); // DEBUG

    Ok("Synced successfully".to_string())
}


#[tauri::command]
async fn get_rooms(state: State<'_, MatrixState>) -> Result<Vec<RoomInfo>, String> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    println!("Getting rooms for client..."); // DEBUG

    let mut rooms_info = Vec::new();

    for room in client.rooms() {
        let name = room
            .display_name()
            .await
            .ok()
            .map(|dn| dn.to_string())
            .or_else(|| Some(room.room_id().to_string()));

        let topic = room.topic();

        rooms_info.push(RoomInfo {
            room_id: room.room_id().to_string(),
            name,
            topic,
        });
    }

    println!("Found {} rooms", rooms_info.len()); // DEBUG

    Ok(rooms_info)
}


#[tauri::command]
async fn get_messages(
    state: State<'_, MatrixState>,
    room_id: String,
    limit: u32,
    from_token: Option<String>, // Add pagination token parameter
) -> Result<MessagesResponse, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let room_id_parsed: OwnedRoomId = room_id
        .parse()
        .map_err(|e| format!("Invalid room ID: {}", e))?;
    
    let room = client
        .get_room(&room_id_parsed)
        .ok_or("Room not found")?;

    // Create options with pagination token if provided
    let mut options = MessagesOptions::backward();
    
    if let Some(token) = from_token {
        options = options.from(Some(token.as_str()));
    }

    let messages_response = room
        .messages(options)
        .await
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    let mut result = Vec::new();

    for timeline_event in messages_response.chunk.iter().take(limit as usize) {
        use matrix_sdk::deserialized_responses::TimelineEventKind;
        use matrix_sdk::ruma::events::{AnyTimelineEvent, AnySyncTimelineEvent, AnyMessageLikeEvent, AnySyncMessageLikeEvent};
        use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEvent, SyncRoomMessageEvent};
        
        match &timeline_event.kind {
            TimelineEventKind::Decrypted(decrypted) => {
                if let Ok(AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(msg))) = decrypted.event.deserialize() {
                    if let RoomMessageEvent::Original(original) = msg {
                        let sender = decrypted.encryption_info.sender.to_string();
                        let body = match &original.content.msgtype {
                            MessageType::Text(t) => t.body.clone(),
                            MessageType::Notice(n) => n.body.clone(),
                            MessageType::Emote(e) => format!("* {}", e.body),
                            _ => continue,
                        };

                        let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                        result.push(Message { sender, body, timestamp });
                    }
                }
            }
            TimelineEventKind::PlainText { event } => {
                if let Ok(AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(msg))) = event.deserialize() {
                    if let SyncRoomMessageEvent::Original(original) = msg {
                        let sender = original.sender.to_string();
                        let body = match &original.content.msgtype {
                            MessageType::Text(t) => t.body.clone(),
                            MessageType::Notice(n) => n.body.clone(),
                            MessageType::Emote(e) => format!("* {}", e.body),
                            _ => continue,
                        };

                        let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                        result.push(Message { sender, body, timestamp });
                    }
                }
            }
            _ => continue,
        }
    }

    result.reverse();
    
    // Return the end token for next pagination
    let next_token = messages_response.end.clone();
    let has_more = next_token.is_some() && !result.is_empty();
    
    Ok(MessagesResponse {
        messages: result,
        has_more,
        next_token,
    })
}


#[tauri::command]
async fn send_message(
    state: State<'_, MatrixState>,
    room_id: String,
    message: String,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let room_id: OwnedRoomId = room_id
        .parse()
        .map_err(|e| format!("Invalid room ID: {}", e))?;
    
    let room = client
        .get_room(&room_id)
        .ok_or("Room not found")?;

    let content = RoomMessageEventContent::text_plain(message.trim());
    
    let response = room
        .send(content)
        .await
        .map_err(|e| format!("Failed to send: {}", e))?;


    Ok(response.event_id.to_string())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MatrixState {
            client: Arc::new(RwLock::new(None)),
            user_id: Arc::new(RwLock::new(None)),
            pagination_tokens: Arc::new(RwLock::new(HashMap::new())),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}