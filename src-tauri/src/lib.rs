<<<<<<< Updated upstream
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
use tauri::{State, Manager}; // Add Manager here
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

=======
use tauri::{ Manager};
mod state;
mod auth;
mod sync_mod;
mod rooms;
mod messages;
mod verification;
>>>>>>> Stashed changes

// Add to the state struct
pub struct MatrixState {
    pub client: Arc<RwLock<Option<Client>>>,
    pub user_id: Arc<RwLock<Option<String>>>,
    pub pagination_tokens: Arc<RwLock<HashMap<String, String>>>,
    pub data_dir: PathBuf,
    pub verification_flow_id: Arc<RwLock<Option<String>>>, // Track active verification
}


#[derive(Serialize, Deserialize)]
pub struct VerificationStatus {
    needs_verification: bool,
    is_verified: bool,
}

#[tauri::command]
async fn check_verification_status(
    state: State<'_, MatrixState>,
) -> Result<VerificationStatus, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let encryption = client.encryption();
    
    // Check if we're verified - returns Option
    let status = encryption.cross_signing_status().await
        .ok_or("Cross-signing not available")?;
    
    let is_verified = status.is_complete();

    Ok(VerificationStatus {
        needs_verification: !is_verified,
        is_verified,
    })
}

#[tauri::command]
async fn request_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();

    println!("Requesting verification for user: {}", user_id);

    // First, do a sync to ensure we have latest device info
    client.sync_once(SyncSettings::default()).await
        .map_err(|e| format!("Sync failed: {}", e))?;

    // Get all our devices
    let devices = encryption
        .get_user_devices(user_id)
        .await
        .map_err(|e| format!("Failed to get devices: {}", e))?;

    println!("Found {} devices", devices.devices().count());

    // Find other devices
    let our_device_id = client.device_id().unwrap();
    let other_devices: Vec<_> = devices.devices()
        .filter(|d| d.device_id() != our_device_id)
        .collect();

    if other_devices.is_empty() {
        return Err("No other devices found. Make sure you're logged in on Element.".to_string());
    }

    println!("Found {} other devices", other_devices.len());

    // Request verification from the first other device
    for device in other_devices {
        println!("Requesting verification from device: {} ({})", 
            device.device_id(), 
            device.display_name().unwrap_or("Unknown")
        );
        
        match device.request_verification().await {
            Ok(verification) => {
                let flow_id = verification.flow_id().to_string();
                println!("Verification requested successfully! Flow ID: {}", flow_id);
                
                // Store the flow ID
                *state.verification_flow_id.write().await = Some(flow_id.clone());
                
                return Ok(format!(
                    "Verification request sent! Check Element on device: {}",
                    device.display_name().unwrap_or("Unknown device")
                ));
            }
            Err(e) => {
                println!("Failed to request from device {}: {}", device.device_id(), e);
                continue;
            }
        }
    }

    Err("Could not send verification request to any device".to_string())
}


#[tauri::command]
async fn get_verification_emoji(
    state: State<'_, MatrixState>,
) -> Result<Vec<(String, String)>, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;
    
    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();
    
    println!("Getting emoji for flow: {}", flow_id);
    
    // Get the verification request
    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    println!("Verification state: is_ready={}, is_done={}, is_cancelled={}", 
        verification.is_ready(), 
        verification.is_done(), 
        verification.is_cancelled()
    );

    // Check if cancelled
    if verification.is_cancelled() {
        return Err("Verification was cancelled".to_string());
    }

    // Wait for it to be ready (other side accepted)
    if !verification.is_ready() {
        return Err("Waiting for other device to accept...".to_string());
    }

    // Start SAS verification
    println!("Starting SAS verification...");
    let sas = verification.start_sas()
        .await
        .map_err(|e| format!("Failed to start SAS: {}", e))?
        .ok_or("SAS not available - other device may not support emoji")?;

    println!("SAS started, accepting...");
    sas.accept().await
        .map_err(|e| format!("Failed to accept SAS: {}", e))?;

    // Wait a bit for emoji to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Get emoji
    if let Some(emoji) = sas.emoji() {
        let emoji_list: Vec<(String, String)> = emoji
            .iter()
            .map(|e| (e.symbol.to_string(), e.description.to_string()))
            .collect();
        println!("Got {} emoji", emoji_list.len());
        return Ok(emoji_list);
    }

    Err("Emoji not ready yet, keep polling...".to_string())
}

#[tauri::command]
async fn confirm_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;
    
    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();
    
    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    // Get SAS by starting it again (it will return the existing one)
    let sas = verification.start_sas()
        .await
        .map_err(|e| format!("Failed to get SAS: {}", e))?
        .ok_or("SAS not available")?;

    println!("Confirming verification...");
    sas.confirm()
        .await
        .map_err(|e| format!("Failed to confirm: {}", e))?;
    
    println!("Confirmed! Waiting for completion...");
    
    // Wait for verification to complete
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let verification_check = encryption
            .get_verification_request(user_id, flow_id)
            .await;
        
        if let Some(v) = verification_check {
            if v.is_done() {
                println!("Verification complete!");
                
                // Do a sync to get the keys
                client.sync_once(SyncSettings::default()).await
                    .map_err(|e| format!("Sync after verification failed: {}", e))?;
                
                break;
            }
        }
    }
    
    // Clear the flow ID
    drop(flow_id_guard);
    *state.verification_flow_id.write().await = None;
    
    Ok("Verification confirmed and complete!".to_string())
}


#[tauri::command]
async fn cancel_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;
    
    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();
    
    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    verification
        .cancel()
        .await
        .map_err(|e| format!("Failed to cancel: {}", e))?;
    
    // Clear the flow ID
    drop(flow_id_guard);
    *state.verification_flow_id.write().await = None;

    Ok("Verification cancelled".to_string())
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

    // Create data directory for this session
    let session_dir = state.data_dir.join(sanitize_user_id(&username));
    
    // If session dir exists, try to restore or clear it
    if session_dir.exists() {
        println!("Found existing session data, clearing...");
        std::fs::remove_dir_all(&session_dir)
            .map_err(|e| format!("Failed to clear old session: {}", e))?;
    }
    
    std::fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    println!("Using session directory: {:?}", session_dir);

    // Build client with encryption enabled
    let client = Client::builder()
        .homeserver_url(homeserver.trim())
        .sqlite_store(&session_dir, None)
        .build()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Login
    let response = client
        .matrix_auth()
        .login_username(username.trim(), &password)
        .initial_device_display_name("Matrix Client (Rust)")
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    let user_id = response.user_id.to_string();
    let device_id = response.device_id.to_string();

    println!("Logged in as {} on device {}", user_id, device_id);

    // Initial sync to get encryption keys
    println!("Performing initial sync...");
    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| format!("Initial sync failed: {}", e))?;

    println!("Login and sync completed successfully");

    *state.client.write().await = Some(client);
    *state.user_id.write().await = Some(user_id.clone());

    Ok(LoginResponse {
        success: true,
        user_id,
        device_id,
        message: "Login successful - encryption enabled".to_string(),
    })
}


// Helper to sanitize user ID for file system
fn sanitize_user_id(user_id: &str) -> String {
    user_id
        .replace("@", "")
        .replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_")
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

    // Clear state
    *state.client.write().await = None;
    *state.user_id.write().await = None;
    *state.verification_flow_id.write().await = None;

    // Clear session directory
    let user_id_guard = state.user_id.read().await;
    if let Some(user_id) = user_id_guard.as_ref() {
        let session_dir = state.data_dir.join(sanitize_user_id(user_id));
        if session_dir.exists() {
            std::fs::remove_dir_all(&session_dir)
                .map_err(|e| format!("Failed to clear session: {}", e))?;
        }
    }

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
    _limit: u32, // Add underscore prefix
    from_token: Option<String>,
) -> Result<MessagesResponse, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    println!("Getting messages for room: {}", room_id);
    println!("From token: {:?}", from_token);

    let room_id_parsed: OwnedRoomId = room_id
        .parse()
        .map_err(|e| format!("Invalid room ID: {}", e))?;
    
    let room = client
        .get_room(&room_id_parsed)
        .ok_or("Room not found")?;

    let options = if let Some(token) = from_token {
        MessagesOptions::backward().from(Some(token.as_str()))
    } else {
        MessagesOptions::backward()
    };

    let messages_response = room
        .messages(options)
        .await
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    println!("Received {} events from server", messages_response.chunk.len());

    let mut result = Vec::new();

    for (idx, timeline_event) in messages_response.chunk.iter().enumerate() {
        use matrix_sdk::deserialized_responses::TimelineEventKind;
        use matrix_sdk::ruma::events::{AnyTimelineEvent, AnySyncTimelineEvent, AnyMessageLikeEvent, AnySyncMessageLikeEvent};
        use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEvent, SyncRoomMessageEvent};
        
        match &timeline_event.kind {
            TimelineEventKind::Decrypted(decrypted) => {
                println!("Event {}: Decrypted successfully!", idx);
                if let Ok(any_event) = decrypted.event.deserialize() {
                    if let AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(msg)) = any_event {
                        if let RoomMessageEvent::Original(original) = msg {
                            let sender = decrypted.encryption_info.sender.to_string();
                            let body = match &original.content.msgtype {
                                MessageType::Text(t) => t.body.clone(),
                                MessageType::Notice(n) => n.body.clone(),
                                MessageType::Emote(e) => format!("* {}", e.body),
                                _ => continue,
                            };

                            let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                            println!("  -> Decrypted message: {}", body);
                            result.push(Message { sender, body, timestamp });
                        }
                    }
                }
            }
            TimelineEventKind::PlainText { event } => {
                println!("Event {}: PlainText", idx);
                if let Ok(any_event) = event.deserialize() {
                    if let AnySyncTimelineEvent::MessageLike(msg) = any_event {
                        if let AnySyncMessageLikeEvent::RoomMessage(room_msg) = msg {
                            if let SyncRoomMessageEvent::Original(original) = room_msg {
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
                }
            }
            TimelineEventKind::UnableToDecrypt { .. } => {
                println!("Event {}: UnableToDecrypt - waiting for keys", idx);
                
                let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                
                result.push(Message {
                    sender: "[Encrypted]".to_string(),
                    body: "🔒 Waiting for encryption keys...".to_string(),
                    timestamp,
                });
            }
        }
    }

    result.reverse();
    
    println!("Parsed {} messages out of {} events", result.len(), messages_response.chunk.len());
    
    let next_token = messages_response.end.clone();
    let has_more = next_token.is_some() && messages_response.chunk.len() > 0;
    
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
        .setup(|app| {
            // Get app data directory
            let data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;

            println!("Using data directory: {:?}", data_dir);

            app.manage(MatrixState {
                client: Arc::new(RwLock::new(None)),
                user_id: Arc::new(RwLock::new(None)),
                pagination_tokens: Arc::new(RwLock::new(HashMap::new())),
                data_dir,
                verification_flow_id: Arc::new(RwLock::new(None)),
            });


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
