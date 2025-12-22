# Backend Documentation - Rust/Tauri

## Table of Contents
1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [State Management](#state-management)
4. [Authentication Module](#authentication-module)
5. [Rooms Module](#rooms-module)
6. [Messages Module](#messages-module)
7. [Sync Module](#sync-module)
8. [Verification Module](#verification-module)
9. [Matrix SDK Integration](#matrix-sdk-integration)
10. [Error Handling](#error-handling)

## Overview

The backend is built with Rust and Tauri, providing native performance and security. It uses the official Matrix Rust SDK for protocol implementation and SQLite for local data persistence.

**Key Technologies**:
- **Tauri**: Desktop application framework
- **matrix-sdk**: Official Rust implementation of Matrix protocol
- **tokio**: Async runtime for concurrent operations
- **serde**: Serialization/deserialization for data transfer
- **SQLite**: Embedded database for session and encryption data

## Module Structure

### File Organization

```
src-tauri/src/
├── main.rs          # Application entry point
├── lib.rs           # Library root, Tauri setup
├── state.rs         # Application state management
├── auth.rs          # Authentication and session management
├── rooms.rs         # Room operations
├── messages.rs      # Message sending
├── sync_mod.rs      # Matrix synchronization
└── verification.rs  # Device verification and encryption
```

### Module Dependencies

```
lib.rs
├── Uses: state, auth, sync_mod, rooms, messages, verification
├── Exports: All module functions as Tauri commands
└── Initializes: Tauri app, state, command handlers

state.rs
└── Provides: MatrixState struct (shared state)

auth.rs
├── Depends on: state, matrix_sdk
└── Provides: login, logout, session check, recovery key

rooms.rs
├── Depends on: state, matrix_sdk
└── Provides: get_rooms, room information

messages.rs
├── Depends on: state, matrix_sdk
└── Provides: get_messages, send_message

sync_mod.rs
├── Depends on: state, matrix_sdk
└── Provides: matrix_sync

verification.rs
├── Depends on: state, matrix_sdk
└── Provides: verification commands
```

---

## State Management

### state.rs
**Location**: `src-tauri/src/state.rs`

**Purpose**: Define and manage application-wide state

```rust
use matrix_sdk::Client;
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

pub struct MatrixState {
    pub client: Arc<RwLock<Option<Client>>>,
    pub user_id: Arc<RwLock<Option<String>>>,
    pub pagination_tokens: Arc<RwLock<HashMap<String, String>>>,
    pub data_dir: PathBuf,
    pub verification_flow_id: Arc<RwLock<Option<String>>>,
}

impl MatrixState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            user_id: Arc::new(RwLock::new(None)),
            pagination_tokens: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
            verification_flow_id: Arc::new(RwLock::new(None)),
        }
    }
}
```

#### Field Explanations

**`client: Arc<RwLock<Option<Client>>>`**
- **Type**: Shared, thread-safe, optional Matrix client
- **Arc**: Atomic Reference Counted pointer - allows multiple ownership
- **RwLock**: Read-Write lock - multiple readers OR one writer
- **Option**: May or may not contain a client (None when logged out)
- **Why**: Client needs to be shared across all command handlers and accessed concurrently

**`user_id: Arc<RwLock<Option<String>>>`**
- **Purpose**: Store current logged-in user's Matrix ID
- **Example**: "@user:matrix.org"
- **Why Optional**: None when not logged in

**`pagination_tokens: Arc<RwLock<HashMap<String, String>>>`**
- **Purpose**: Store pagination tokens for each room
- **Key**: Room ID
- **Value**: Next pagination token
- **Note**: Currently not heavily used (tokens passed through frontend)

**`data_dir: PathBuf`**
- **Purpose**: Path to application data directory
- **Contains**: SQLite databases, session files, encryption keys
- **Example**: `C:\Users\user\AppData\Local\matrix-client\`

**`verification_flow_id: Arc<RwLock<Option<String>>>`**
- **Purpose**: Track active device verification session
- **Usage**: Link verification requests to their emoji/confirmation

#### Concurrency Pattern

```rust
// Reading (many readers allowed simultaneously)
let client_guard = state.client.read().await;
let client = client_guard.as_ref().ok_or("Not logged in")?;

// Writing (exclusive access)
*state.client.write().await = Some(new_client);
```

**Why RwLock?**
- Multiple commands can read client simultaneously
- Only one command can modify state at a time
- Prevents data races and corruption

**Why Arc?**
- Tauri commands run in separate async tasks
- Each needs reference to state
- Arc provides safe shared ownership

---

## Authentication Module

### auth.rs
**Location**: `src-tauri/src/auth.rs`

**Purpose**: Handle user authentication and session management

#### Command: matrix_login

```rust
#[tauri::command]
pub async fn matrix_login(
    state: State<'_, MatrixState>,
    homeserver: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String>
```

**Parameters**:
- `state`: Injected application state (managed by Tauri)
- `homeserver`: Matrix homeserver URL (e.g., "https://matrix.org")
- `username`: User's Matrix ID (e.g., "@user:matrix.org")
- `password`: User's password

**Return Type**: `Result<LoginResponse, String>`
- **Ok**: LoginResponse with success, user_id, device_id, message
- **Err**: Error message string

**Flow**:

1. **Validate Input**
   ```rust
   if homeserver.trim().is_empty() || username.trim().is_empty() || password.is_empty() {
       return Err("All fields are required".to_string());
   }
   
   if !homeserver.starts_with("http://") && !homeserver.starts_with("https://") {
       return Err("Homeserver URL must start with http:// or https://".to_string());
   }
   ```

2. **Prepare Session Directory**
   ```rust
   let session_dir = state.data_dir.join(sanitize_user_id(&username));
   
   // Clear existing session if present
   if session_dir.exists() {
       fs::remove_dir_all(&session_dir)?;
   }
   
   fs::create_dir_all(&session_dir)?;
   ```
   - Creates unique directory per user
   - Clears old data on re-login
   - Ensures clean state

3. **Build Matrix Client**
   ```rust
   let client = Client::builder()
       .homeserver_url(homeserver.trim())
       .sqlite_store(&session_dir, None)
       .build()
       .await?;
   ```
   - Connects to homeserver
   - Configures SQLite storage for:
     - Session tokens
     - Encryption keys
     - Room data
     - Device information

4. **Perform Login**
   ```rust
   let response = client
       .matrix_auth()
       .login_username(username.trim(), &password)
       .initial_device_display_name("Matrix Client (Rust)")
       .await?;
   ```
   - Authenticates with homeserver
   - Sets device display name
   - Returns session information

5. **Initial Sync**
   ```rust
   client.sync_once(SyncSettings::default()).await?;
   ```
   - Performs initial sync with server
   - Downloads:
     - Room list
     - Recent messages
     - Encryption keys (if applicable)
     - User presence

6. **Update State**
   ```rust
   *state.client.write().await = Some(client);
   *state.user_id.write().await = Some(user_id.clone());
   ```
   - Stores client for future use
   - Saves user ID

#### Helper Function: sanitize_user_id

```rust
fn sanitize_user_id(user_id: &str) -> String {
    user_id
        .replace("@", "")
        .replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_")
}
```

**Purpose**: Convert Matrix user ID to safe filesystem path
**Example**: `@user:matrix.org` → `user_matrix.org`

#### Command: check_session

```rust
#[tauri::command]
pub async fn check_session(state: State<'_, MatrixState>) -> Result<Option<String>, String> {
    let user_id = state.user_id.read().await;
    Ok(user_id.clone())
}
```

**Purpose**: Check if user is currently logged in
**Returns**: 
- `Some(user_id)` if logged in
- `None` if not logged in

**Usage**: Frontend calls on app startup to restore session

#### Command: logout

```rust
#[tauri::command]
pub async fn logout(state: State<'_, MatrixState>) -> Result<String, String>
```

**Flow**:

1. **Logout from Server**
   ```rust
   let client_read = state.client.read().await;
   if let Some(client) = client_read.as_ref() {
       client.logout().await?;
   }
   drop(client_read);
   ```
   - Invalidates session on homeserver
   - Properly drops read lock

2. **Clear State**
   ```rust
   *state.client.write().await = None;
   *state.user_id.write().await = None;
   *state.verification_flow_id.write().await = None;
   ```

3. **Delete Local Data**
   ```rust
   let user_id_guard = state.user_id.read().await;
   if let Some(user_id) = user_id_guard.as_ref() {
       let session_dir = state.data_dir.join(sanitize_user_id(user_id));
       if session_dir.exists() {
           fs::remove_dir_all(&session_dir)?;
       }
   }
   ```
   - Removes session database
   - Clears encryption keys
   - Ensures data privacy

#### Command: verify_with_recovery_key

```rust
#[tauri::command]
pub async fn verify_with_recovery_key(
    state: State<'_, MatrixState>,
    recovery_key: String,
) -> Result<String, String>
```

**Purpose**: Verify device using recovery key instead of another device

**Flow**:

1. **Validate Input**
   ```rust
   if recovery_key.trim().is_empty() {
       return Err("Recovery key is required".to_string());
   }
   ```

2. **Get Encryption API**
   ```rust
   let client_guard = state.client.read().await;
   let client = client_guard.as_ref().ok_or("Client is not logged in")?;
   let encryption = client.encryption();
   let recovery = encryption.recovery();
   ```

3. **Perform Recovery**
   ```rust
   recovery.recover(&recovery_key).await?;
   ```
   - Imports cross-signing keys
   - Marks device as verified
   - Enables message decryption

**Recovery Key Format**: Base58 encoded string with spaces
- Example: `EsTc 1234 5678 9abc defg...`

---

## Rooms Module

### rooms.rs
**Location**: `src-tauri/src/rooms.rs`

**Purpose**: Handle room operations and information retrieval

#### Data Structures

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    pub room_id: String,
    pub name: Option<String>,
    pub topic: Option<String>,
}
```

**Fields**:
- `room_id`: Unique room identifier (e.g., "!abc123:matrix.org")
- `name`: Human-readable room name (None if not set)
- `topic`: Room description/topic (None if not set)

**Derive Macros**:
- `Serialize/Deserialize`: JSON conversion for Tauri
- `Clone`: Allow copying room info
- `Debug`: Enable debug printing

#### Command: get_rooms

```rust
#[tauri::command]
pub async fn get_rooms(state: State<'_, MatrixState>) -> Result<Vec<RoomInfo>, String>
```

**Purpose**: Retrieve list of all joined rooms

**Flow**:

1. **Get Client**
   ```rust
   let client_lock = state.client.read().await;
   let client = client_lock.as_ref().ok_or("Not logged in")?;
   ```

2. **Iterate Rooms**
   ```rust
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
   ```

3. **Return List**
   ```rust
   println!("Found {} rooms", rooms_info.len());
   Ok(rooms_info)
   ```

**How Display Name Works**:
- Tries to get room's set name
- Falls back to room ID if no name
- Matrix spec defines name hierarchy:
  1. m.room.name event
  2. m.room.canonical_alias event
  3. Calculated from members (DMs)
  4. Room ID

---

## Messages Module

### messages.rs
**Location**: `src-tauri/src/messages.rs`

**Purpose**: Handle sending messages (receiving is in rooms.rs)

#### Command: send_message

```rust
#[tauri::command]
pub async fn send_message(
    state: State<'_, MatrixState>,
    room_id: String,
    message: String,
) -> Result<String, String>
```

**Purpose**: Send text message to a room

**Flow**:

1. **Get Client and Room**
   ```rust
   let client = state.client.read().await;
   let client = client.as_ref().ok_or("Not logged in")?;
   
   let room_id: OwnedRoomId = room_id.parse()?;
   let room = client.get_room(&room_id).ok_or("Room not found")?;
   ```

2. **Create Message Content**
   ```rust
   let content = RoomMessageEventContent::text_plain(message.trim());
   ```
   - Creates plain text message
   - Matrix supports rich content (HTML, files, images, etc.)
   - This implementation uses simple text

3. **Send Message**
   ```rust
   let response = room.send(content).await?;
   ```
   - Encrypts if room is encrypted
   - Sends to homeserver
   - Homeserver distributes to room members

4. **Return Event ID**
   ```rust
   Ok(response.event_id.to_string())
   ```
   - Event ID uniquely identifies this message
   - Format: `$abc123xyz:matrix.org`
   - Can be used for editing, reactions, replies

---

## Messages Module (Receiving)

### rooms.rs - get_messages
**Location**: `src-tauri/src/rooms.rs`

```rust
#[tauri::command]
pub async fn get_messages(
    state: State<'_, MatrixState>,
    room_id: String,
    _limit: u32,
    from_token: Option<String>,
) -> Result<MessagesResponse, String>
```

**Purpose**: Retrieve message history for a room with pagination

#### Data Structures

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub body: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
    pub next_token: Option<String>,
}
```

#### Flow

1. **Setup**
   ```rust
   let room_id_parsed: OwnedRoomId = room_id.parse()?;
   let room = client.get_room(&room_id_parsed).ok_or("Room not found")?;
   ```

2. **Configure Pagination**
   ```rust
   let options = if let Some(token) = from_token {
       MessagesOptions::backward().from(Some(token.as_str()))
   } else {
       MessagesOptions::backward()
   };
   ```
   - **backward()**: Load older messages
   - **from()**: Start from pagination token
   - **No token**: Load most recent messages

3. **Fetch from Server**
   ```rust
   let messages_response = room.messages(options).await?;
   ```
   - Retrieves batch of timeline events
   - Includes encrypted and plain messages
   - Returns pagination token for next batch

4. **Process Events** (Complex Part)

The Matrix SDK provides events in different states:

```rust
for timeline_event in messages_response.chunk.iter() {
    match &timeline_event.kind {
        TimelineEventKind::Decrypted(decrypted) => {
            // Successfully decrypted message
        }
        TimelineEventKind::PlainText { event } => {
            // Unencrypted message
        }
        TimelineEventKind::UnableToDecrypt { .. } => {
            // Encrypted but keys not available
        }
    }
}
```

##### Processing Decrypted Messages

```rust
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
                
                let timestamp = timeline_event.timestamp
                    .map(|ts| ts.get().into())
                    .unwrap_or(0);
                
                result.push(Message { sender, body, timestamp });
            }
        }
    }
}
```

**Steps**:
1. Deserialize decrypted event
2. Check if it's a room message
3. Extract sender from encryption info
4. Handle different message types (text, notice, emote)
5. Get timestamp
6. Create Message struct

##### Processing Plain Text Messages

```rust
TimelineEventKind::PlainText { event } => {
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
                    let timestamp = timeline_event.timestamp
                        .map(|ts| ts.get().into())
                        .unwrap_or(0);
                    
                    result.push(Message { sender, body, timestamp });
                }
            }
        }
    }
}
```

Similar to decrypted, but simpler:
- No encryption info
- Different event types (Sync vs Timeline)

##### Handling Unable to Decrypt

```rust
TimelineEventKind::UnableToDecrypt { .. } => {
    let timestamp = timeline_event.timestamp
        .map(|ts| ts.get().into())
        .unwrap_or(0);
    
    result.push(Message {
        sender: "[Encrypted]".to_string(),
        body: "🔒 Waiting for encryption keys...".to_string(),
        timestamp,
    });
}
```

**Why Unable to Decrypt?**
- Device not verified
- Keys not yet received from other devices
- Sender's device was deleted
- Session rotation

5. **Prepare Response**
   ```rust
   result.reverse();  // Oldest first
   
   let next_token = messages_response.end.clone();
   let has_more = next_token.is_some() && messages_response.chunk.len() > 0;
   
   Ok(MessagesResponse {
       messages: result,
       has_more,
       next_token,
   })
   ```

---

## Sync Module

### sync_mod.rs
**Location**: `src-tauri/src/sync_mod.rs`

**Purpose**: Synchronize client state with homeserver

```rust
#[tauri::command]
pub async fn matrix_sync(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;
    
    println!("Starting sync...");
    
    client
        .sync_once(SyncSettings::default())
        .await?;
    
    println!("Sync completed");
    
    Ok("Synced successfully".to_string())
}
```

**What Sync Does**:
1. Fetch new events since last sync
2. Update room state (members, names, topics)
3. Download new encryption keys
4. Update device lists
5. Receive new messages
6. Update presence information

**Sync Types**:
- **sync_once()**: Single sync operation (used here)
- **sync()**: Continuous sync loop (for always-online clients)

**SyncSettings Options**:
- **Timeout**: How long to wait for new events
- **Filter**: What events to receive
- **Full state**: Whether to request full room state

---

## Verification Module

### verification.rs
**Location**: `src-tauri/src/verification.rs`

**Purpose**: Handle device verification for end-to-end encryption

#### Command: check_verification_status

```rust
#[tauri::command]
pub async fn check_verification_status(
    state: State<'_, MatrixState>,
) -> Result<VerificationStatus, String>
```

**Purpose**: Check if device is verified

**Flow**:
```rust
let encryption = client.encryption();
let status = encryption.cross_signing_status().await?;
let is_verified = status.is_complete();

Ok(VerificationStatus {
    needs_verification: !is_verified,
    is_verified,
})
```

**Cross-Signing Status**:
- Checks if device has been verified
- Verifies cross-signing keys are available
- Confirms trust relationship established

#### Command: request_verification

```rust
#[tauri::command]
pub async fn request_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Purpose**: Request verification from another logged-in device

**Flow**:

1. **Sync First**
   ```rust
   client.sync_once(SyncSettings::default()).await?;
   ```
   - Ensures device list is up-to-date

2. **Get User's Devices**
   ```rust
   let user_id = client.user_id().ok_or("No user ID")?;
   let devices = encryption.get_user_devices(user_id).await?;
   ```

3. **Filter Other Devices**
   ```rust
   let our_device_id = client.device_id().unwrap();
   let other_devices: Vec<_> = devices.devices()
       .filter(|d| d.device_id() != our_device_id)
       .collect();
   ```
   - Excludes current device
   - Can't verify with yourself

4. **Request from Each Device**
   ```rust
   for device in other_devices {
       match device.request_verification().await {
           Ok(verification) => {
               let flow_id = verification.flow_id().to_string();
               *state.verification_flow_id.write().await = Some(flow_id.clone());
               return Ok(format!("Verification request sent! ..."));
           }
           Err(e) => {
               println!("Failed to request from device: {}", e);
               continue;
           }
       }
   }
   ```

**Verification Flow ID**: Unique identifier for this verification session

#### Command: get_verification_emoji

```rust
#[tauri::command]
pub async fn get_verification_emoji(
    state: State<'_, MatrixState>,
) -> Result<Vec<(String, String)>, String>
```

**Purpose**: Get emoji for comparison with other device

**Flow**:

1. **Get Active Verification**
   ```rust
   let flow_id = state.verification_flow_id.read().await;
   let flow_id = flow_id.as_ref().ok_or("No active verification")?;
   
   let verification = encryption
       .get_verification_request(user_id, flow_id)
       .await
       .ok_or("Verification not found")?;
   ```

2. **Check Status**
   ```rust
   if verification.is_cancelled() {
       return Err("Verification was cancelled".to_string());
   }
   
   if !verification.is_ready() {
       return Err("Waiting for other device to accept...".to_string());
   }
   ```

3. **Start SAS (Short Authentication String)**
   ```rust
   let sas = verification.start_sas().await?
       .ok_or("SAS not available - other device may not support emoji")?;
   
   sas.accept().await?;
   ```

**SAS**: Interactive verification using short codes (emoji or numbers)

4. **Wait for Emoji**
   ```rust
   sleep(Duration::from_millis(1000)).await;
   
   if let Some(emoji) = sas.emoji() {
       let emoji_list: Vec<(String, String)> = emoji
           .iter()
           .map(|e| (e.symbol.to_string(), e.description.to_string()))
           .collect();
       return Ok(emoji_list);
   }
   ```

**Emoji Format**: 7 emoji with descriptions
- Example: [("🐶", "dog"), ("🌊", "wave"), ...]

#### Command: confirm_verification

```rust
#[tauri::command]
pub async fn confirm_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Purpose**: Confirm emoji match and complete verification

**Flow**:

1. **Get SAS**
   ```rust
   let verification = encryption
       .get_verification_request(user_id, flow_id)
       .await?;
   
   let sas = verification.start_sas().await?
       .ok_or("SAS not available")?;
   ```

2. **Confirm**
   ```rust
   sas.confirm().await?;
   ```
   - Tells other device emoji matched
   - Establishes trust

3. **Wait for Completion**
   ```rust
   for _ in 0..20 {
       sleep(Duration::from_millis(500)).await;
       
       let verification_check = encryption
           .get_verification_request(user_id, flow_id)
           .await;
       
       if let Some(v) = verification_check {
           if v.is_done() {
               println!("Verification complete!");
               break;
           }
       }
   }
   ```
   - Polls for completion (up to 10 seconds)
   - Both devices must confirm

4. **Final Sync**
   ```rust
   client.sync_once(SyncSettings::default()).await?;
   ```
   - Downloads cross-signing keys
   - Updates verification status

#### Command: cancel_verification

```rust
#[tauri::command]
pub async fn cancel_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Purpose**: Cancel ongoing verification

**Flow**:
```rust
let verification = encryption
    .get_verification_request(user_id, flow_id)
    .await?;

verification.cancel().await?;

*state.verification_flow_id.write().await = None;
```

---

## Matrix SDK Integration

### Key SDK Concepts

#### Client
- Main entry point to Matrix SDK
- Manages connection to homeserver
- Stores session data
- Provides access to rooms, encryption, etc.

#### Room
- Represents a Matrix room
- Methods for sending messages, reading history
- Access to room state (name, topic, members)

#### Encryption
- End-to-end encryption management
- Device verification
- Key sharing and backup

#### Timeline Events
- Messages and state changes in rooms
- Can be encrypted or plaintext
- Different types: messages, membership, name changes, etc.

### SQLite Storage

The Matrix SDK stores:
1. **Session Data**: Login tokens, device ID
2. **Room State**: Members, names, power levels
3. **Encryption Keys**: Olm and Megolm keys
4. **Cross-Signing Keys**: For device verification
5. **Message History**: Recent messages for offline access

**Location**: `{data_dir}/{sanitized_user_id}/`

---

## Error Handling

### Pattern: Result<T, String>

All Tauri commands return `Result<Success, Error>`:

```rust
#[tauri::command]
pub async fn example(state: State<'_, MatrixState>) -> Result<String, String> {
    // Success case
    Ok("Success message".to_string())
    
    // Error case
    Err("Error message".to_string())
}
```

### Error Propagation with ?

```rust
let client = client_lock.as_ref().ok_or("Not logged in")?;
```

**What happens**:
1. If `Some(client)`: extracts client, continues
2. If `None`: returns `Err("Not logged in")` immediately

### map_err for Custom Messages

```rust
client.logout()
    .await
    .map_err(|e| format!("Logout failed: {}", e))?;
```

**What happens**:
- SDK error converted to custom message
- Includes original error for debugging

### Common Error Patterns

```rust
// Validation
if input.is_empty() {
    return Err("Input required".to_string());
}

// Option to Result
let value = option.ok_or("Value not found")?;

// Result with context
operation().await
    .map_err(|e| format!("Operation failed: {}", e))?;
```

---

## Async/Await in Rust

### Why Async?

- Network operations take time
- Don't block UI thread
- Handle multiple operations concurrently

### Pattern

```rust
#[tauri::command]
pub async fn my_command(state: State<'_, MatrixState>) -> Result<String, String> {
    // .await pauses execution until future completes
    let result = async_operation().await?;
    Ok(result)
}
```

### tokio Runtime

Tauri uses tokio for async:
- Efficient task scheduling
- Async file I/O
- Async networking
- Sleep and timers

---

## Security Considerations

### Password Handling
- Never logged
- Never stored
- Passed directly to SDK
- Cleared from memory after use

### Session Storage
- Encrypted SQLite database
- Platform-specific data directory
- User-specific isolation
- Deleted on logout

### Encryption Keys
- Managed by Matrix SDK
- Never exposed to application
- Protected by device verification
- Backed up with recovery key

### Input Validation
- All user input validated
- Homeserver URL format checked
- Room IDs parsed safely
- No SQL injection (SQLite is abstraction)

---

## Best Practices Used

1. **Error Handling**: All operations return Result
2. **Logging**: println! for debugging (should use proper logging in production)
3. **State Management**: Centralized in MatrixState
4. **Async/Await**: Non-blocking operations
5. **Type Safety**: Rust's type system prevents many bugs
6. **Memory Safety**: No manual memory management
7. **Thread Safety**: RwLock for concurrent access
8. **Clean Architecture**: Separate modules for concerns

---

## Performance Considerations

### Concurrent Reads
- RwLock allows multiple simultaneous reads
- Only writes block

### Database Performance
- SQLite is fast for local storage
- Indexed by SDK for quick queries
- Connection pooling handled by SDK

### Message Batching
- Fetch 50 messages at a time
- Pagination prevents loading entire history
- Balance between UX and performance

### Sync Efficiency
- Only fetches new events
- Uses pagination tokens
- Filters reduce data transfer
