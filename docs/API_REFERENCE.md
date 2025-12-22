# API Reference - Complete Command & Interface Documentation

## Table of Contents
1. [Overview](#overview)
2. [Tauri Commands](#tauri-commands)
3. [TypeScript Interfaces](#typescript-interfaces)
4. [Frontend Service API](#frontend-service-api)
5. [Error Codes](#error-codes)
6. [Usage Examples](#usage-examples)

## Overview

This document provides a complete reference for all API commands and interfaces in the Matrix Client application. Commands are invoked from the frontend (TypeScript) and handled by the backend (Rust).

**Communication Pattern**:
```
Frontend (TypeScript) 
    ↓ invoke("command_name", args)
Tauri Bridge
    ↓
Backend (Rust Command)
    ↓ Result<T, String>
Tauri Bridge
    ↓ Promise<T> or throws Error
Frontend (TypeScript)
```

---

## Tauri Commands

All commands are async and return a Promise on the frontend side.

### Authentication Commands

#### matrix_login
**Backend**: `src-tauri/src/auth.rs`

**Purpose**: Authenticate user with Matrix homeserver

**Signature**:
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
| Name | Type | Required | Description | Example |
|------|------|----------|-------------|---------|
| homeserver | String | Yes | Matrix homeserver URL | "https://matrix.org" |
| username | String | Yes | Matrix user ID | "@user:matrix.org" |
| password | String | Yes | User's password | "secretpassword" |

**Returns**: `LoginResponse`
```typescript
{
  success: boolean,
  user_id: string,
  device_id: string,
  message: string
}
```

**Success Example**:
```json
{
  "success": true,
  "user_id": "@user:matrix.org",
  "device_id": "ABCDEFGHIJ",
  "message": "Login successful - encryption enabled"
}
```

**Error Cases**:
- "All fields are required" - Missing input
- "Homeserver URL must start with http:// or https://" - Invalid URL format
- "Failed to connect: [reason]" - Connection failed
- "Login failed: [reason]" - Invalid credentials or server error
- "Initial sync failed: [reason]" - Post-login sync failed

**Side Effects**:
- Creates SQLite database in app data directory
- Stores session data
- Initializes encryption
- Updates application state

---

#### check_session
**Backend**: `src-tauri/src/auth.rs`

**Purpose**: Check if user has active session

**Signature**:
```rust
#[tauri::command]
pub async fn check_session(state: State<'_, MatrixState>) -> Result<Option<String>, String>
```

**Parameters**: None

**Returns**: `Option<String>`
- `Some(user_id)` - User is logged in
- `None` - No active session

**Success Example**:
```json
"@user:matrix.org"
```

**Error Cases**: Never errors (returns None instead)

**Usage**: Called on app startup to restore session

---

#### logout
**Backend**: `src-tauri/src/auth.rs`

**Purpose**: Log out user and clear session data

**Signature**:
```rust
#[tauri::command]
pub async fn logout(state: State<'_, MatrixState>) -> Result<String, String>
```

**Parameters**: None

**Returns**: `String` - Success message

**Success Example**:
```json
"Logged out successfully"
```

**Error Cases**:
- "Failed to clear session: [reason]" - Local data cleanup failed

**Side Effects**:
- Invalidates server session
- Clears application state
- Deletes local SQLite database
- Removes encryption keys

---

#### verify_with_recovery_key
**Backend**: `src-tauri/src/auth.rs`

**Purpose**: Verify device using recovery key

**Signature**:
```rust
#[tauri::command]
pub async fn verify_with_recovery_key(
    state: State<'_, MatrixState>,
    recovery_key: String,
) -> Result<String, String>
```

**Parameters**:
| Name | Type | Required | Description | Example |
|------|------|----------|-------------|---------|
| recovery_key | String | Yes | Security key from Element | "EsTc 1234 5678 9abc..." |

**Returns**: `String` - Success message

**Success Example**:
```json
"Recovery key verification completed"
```

**Error Cases**:
- "Recovery key is required" - Empty input
- "Client is not logged in" - No active session
- "Failed to verify with recovery key: [reason]" - Invalid key or verification failed

**Side Effects**:
- Imports cross-signing keys
- Marks device as verified
- Enables encrypted message decryption

---

### Synchronization Commands

#### matrix_sync
**Backend**: `src-tauri/src/sync_mod.rs`

**Purpose**: Synchronize with homeserver to get updates

**Signature**:
```rust
#[tauri::command]
pub async fn matrix_sync(state: State<'_, MatrixState>) -> Result<String, String>
```

**Parameters**: None

**Returns**: `String` - Success message

**Success Example**:
```json
"Synced successfully"
```

**Error Cases**:
- "Not logged in" - No active session
- "Sync failed: [reason]" - Server communication error

**What It Does**:
- Fetches new events
- Updates room state
- Downloads encryption keys
- Updates device lists
- Receives new messages

**Performance**: Usually takes 100-500ms

---

### Room Commands

#### get_rooms
**Backend**: `src-tauri/src/rooms.rs`

**Purpose**: Get list of joined rooms

**Signature**:
```rust
#[tauri::command]
pub async fn get_rooms(state: State<'_, MatrixState>) -> Result<Vec<RoomInfo>, String>
```

**Parameters**: None

**Returns**: `Array<RoomInfo>`
```typescript
{
  room_id: string,
  name?: string,
  topic?: string
}[]
```

**Success Example**:
```json
[
  {
    "room_id": "!abc123:matrix.org",
    "name": "General Chat",
    "topic": "Welcome to the room!"
  },
  {
    "room_id": "!xyz789:matrix.org",
    "name": null,
    "topic": null
  }
]
```

**Error Cases**:
- "Not logged in" - No active session

**Note**: 
- `name` is `null` if room has no set name
- Falls back to room ID for display in that case
- `topic` is `null` if not set

---

#### get_messages
**Backend**: `src-tauri/src/rooms.rs`

**Purpose**: Get message history for a room with pagination

**Signature**:
```rust
#[tauri::command]
pub async fn get_messages(
    state: State<'_, MatrixState>,
    room_id: String,
    limit: u32,
    from_token: Option<String>,
) -> Result<MessagesResponse, String>
```

**Parameters**:
| Name | Type | Required | Description | Example |
|------|------|----------|-------------|---------|
| room_id | String | Yes | Room identifier | "!abc123:matrix.org" |
| limit | u32 | Yes | Max messages to fetch | 50 |
| from_token | Option<String> | No | Pagination token | "t1234-5678..." |

**Returns**: `MessagesResponse`
```typescript
{
  messages: Message[],
  has_more: boolean,
  next_token?: string
}
```

**Message Structure**:
```typescript
{
  sender: string,
  body: string,
  timestamp: number
}
```

**Success Example**:
```json
{
  "messages": [
    {
      "sender": "@user:matrix.org",
      "body": "Hello, world!",
      "timestamp": 1703260800000
    },
    {
      "sender": "@alice:matrix.org",
      "body": "Hi there!",
      "timestamp": 1703260860000
    }
  ],
  "has_more": true,
  "next_token": "t1234-5678_M..."
}
```

**Special Messages**:
```json
{
  "sender": "[Encrypted]",
  "body": "🔒 Waiting for encryption keys...",
  "timestamp": 1703260920000
}
```
Indicates encrypted message that can't be decrypted yet.

**Error Cases**:
- "Not logged in" - No active session
- "Invalid room ID: [reason]" - Malformed room ID
- "Room not found" - Not member of this room
- "Failed to fetch messages: [reason]" - Server error

**Pagination**:
- First call: omit `from_token` to get most recent messages
- Subsequent calls: use `next_token` from previous response
- `has_more` indicates if more messages are available

---

### Message Commands

#### send_message
**Backend**: `src-tauri/src/messages.rs`

**Purpose**: Send text message to a room

**Signature**:
```rust
#[tauri::command]
pub async fn send_message(
    state: State<'_, MatrixState>,
    room_id: String,
    message: String,
) -> Result<String, String>
```

**Parameters**:
| Name | Type | Required | Description | Example |
|------|------|----------|-------------|---------|
| room_id | String | Yes | Target room ID | "!abc123:matrix.org" |
| message | String | Yes | Message text | "Hello, world!" |

**Returns**: `String` - Event ID of sent message

**Success Example**:
```json
"$abc123xyz789:matrix.org"
```

**Error Cases**:
- "Not logged in" - No active session
- "Invalid room ID: [reason]" - Malformed room ID
- "Room not found" - Not member of this room
- "Failed to send: [reason]" - Server error or no permission

**Side Effects**:
- Message encrypted if room is encrypted
- Sent to all room members
- Appears in message history

**Performance**: Usually 100-300ms

---

### Verification Commands

#### check_verification_status
**Backend**: `src-tauri/src/verification.rs`

**Purpose**: Check if device is verified

**Signature**:
```rust
#[tauri::command]
pub async fn check_verification_status(
    state: State<'_, MatrixState>,
) -> Result<VerificationStatus, String>
```

**Parameters**: None

**Returns**: `VerificationStatus`
```typescript
{
  needs_verification: boolean,
  is_verified: boolean
}
```

**Success Example**:
```json
{
  "needs_verification": false,
  "is_verified": true
}
```

**Error Cases**:
- "Not logged in" - No active session
- "Cross-signing not available" - Account not set up for encryption

**States**:
- `{ needs_verification: true, is_verified: false }` - Device needs verification
- `{ needs_verification: false, is_verified: true }` - Device is verified

---

#### request_verification
**Backend**: `src-tauri/src/verification.rs`

**Purpose**: Request verification from another device

**Signature**:
```rust
#[tauri::command]
pub async fn request_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Parameters**: None

**Returns**: `String` - Success message with device name

**Success Example**:
```json
"Verification request sent! Check Element on device: Element Web"
```

**Error Cases**:
- "Not logged in" - No active session
- "No user ID" - Invalid state
- "Failed to get devices: [reason]" - Can't fetch device list
- "No other devices found. Make sure you're logged in on Element." - No devices to verify with
- "Could not send verification request to any device" - All requests failed

**Side Effects**:
- Sends verification request to other device(s)
- Stores flow ID for tracking
- Other device receives notification

**Prerequisites**: 
- Must be logged in on another device (Element, etc.)
- Other device must be online and responsive

---

#### get_verification_emoji
**Backend**: `src-tauri/src/verification.rs`

**Purpose**: Get emoji for verification comparison

**Signature**:
```rust
#[tauri::command]
pub async fn get_verification_emoji(
    state: State<'_, MatrixState>,
) -> Result<Vec<(String, String)>, String>
```

**Parameters**: None

**Returns**: `Array<[string, string]>` - Array of [emoji, name] tuples

**Success Example**:
```json
[
  ["🐶", "dog"],
  ["🌊", "wave"],
  ["🍕", "pizza"],
  ["🎸", "guitar"],
  ["🌈", "rainbow"],
  ["🔥", "fire"],
  ["⭐", "star"]
]
```

**Error Cases**:
- "No active verification" - No request in progress
- "Verification not found" - Flow ID invalid
- "Verification was cancelled" - User or other device cancelled
- "Waiting for other device to accept..." - Not ready yet (keep polling)
- "SAS not available - other device may not support emoji" - Incompatible verification method
- "Emoji not ready yet, keep polling..." - Still generating

**Usage Pattern**:
1. Call `request_verification()`
2. Poll this command every 1 second
3. When emoji returned, display to user
4. User compares with other device
5. Call `confirm_verification()` if match

---

#### confirm_verification
**Backend**: `src-tauri/src/verification.rs`

**Purpose**: Confirm emoji match and complete verification

**Signature**:
```rust
#[tauri::command]
pub async fn confirm_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Parameters**: None

**Returns**: `String` - Success message

**Success Example**:
```json
"Verification confirmed and complete!"
```

**Error Cases**:
- "No active verification" - No request in progress
- "Verification not found" - Flow ID invalid
- "SAS not available" - Verification state invalid
- "Failed to confirm: [reason]" - Confirmation failed

**Side Effects**:
- Establishes trust between devices
- Enables cross-signing
- Allows key sharing
- Enables encrypted message decryption

**Performance**: May take 2-5 seconds to complete

---

#### cancel_verification
**Backend**: `src-tauri/src/verification.rs`

**Purpose**: Cancel ongoing verification

**Signature**:
```rust
#[tauri::command]
pub async fn cancel_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String>
```

**Parameters**: None

**Returns**: `String` - Success message

**Success Example**:
```json
"Verification cancelled"
```

**Error Cases**:
- "No active verification" - No request in progress
- "Verification not found" - Flow ID invalid
- "Failed to cancel: [reason]" - Cancellation failed

**Side Effects**:
- Notifies other device
- Clears verification flow ID
- User can start new verification

---

## TypeScript Interfaces

### Types Location
**File**: `src/types/index.ts`

### RoomInfo
```typescript
export interface RoomInfo {
  room_id: string;
  name?: string;
  topic?: string;
}
```

**Description**: Information about a Matrix room

**Fields**:
- `room_id`: Unique room identifier (always present)
- `name`: Human-readable room name (optional)
- `topic`: Room description (optional)

**Usage**: Displayed in room list, passed to message functions

---

### Message
```typescript
export interface Message {
  sender: string;
  body: string;
  timestamp: number;
}
```

**Description**: A single message in a room

**Fields**:
- `sender`: Matrix user ID of sender (@user:server)
- `body`: Message text content
- `timestamp`: Unix timestamp in milliseconds

**Usage**: Displayed in message list

---

### LoginResponse
```typescript
export interface LoginResponse {
  success: boolean;
  user_id: string;
  device_id: string;
  message: string;
}
```

**Description**: Response from login command

**Fields**:
- `success`: Whether login succeeded
- `user_id`: Logged-in user's Matrix ID
- `device_id`: Unique device identifier
- `message`: Human-readable status message

**Usage**: Returned by `matrix_login` command

---

### MessagesResponse
```typescript
export interface MessagesResponse {
  messages: Message[];
  has_more: boolean;
  next_token?: string;
}
```

**Description**: Paginated message history response

**Fields**:
- `messages`: Array of messages
- `has_more`: Whether more messages are available
- `next_token`: Token for fetching next page (if has_more is true)

**Usage**: Returned by `get_messages` command

---

### VerificationStatus
```typescript
export interface VerificationStatus {
  needs_verification: boolean;
  is_verified: boolean;
}
```

**Description**: Device verification state

**Fields**:
- `needs_verification`: Whether device should be verified
- `is_verified`: Whether device is currently verified

**Usage**: Returned by `check_verification_status` command

---

## Frontend Service API

### matrixService
**Location**: `src/services/matrixService.ts`

**Description**: Service wrapper for Tauri commands

All methods are async and return Promises.

### Authentication Methods

#### login(homeserver, username, password)
```typescript
async login(
  homeserver: string,
  username: string,
  password: string
): Promise<LoginResponse>
```

**Example**:
```typescript
const result = await matrixService.login(
  "https://matrix.org",
  "@user:matrix.org",
  "password123"
);
console.log(result.user_id); // "@user:matrix.org"
```

---

#### checkSession()
```typescript
async checkSession(): Promise<string | null>
```

**Example**:
```typescript
const userId = await matrixService.checkSession();
if (userId) {
  console.log("Logged in as", userId);
} else {
  console.log("Not logged in");
}
```

---

#### logout()
```typescript
async logout(): Promise<string>
```

**Example**:
```typescript
await matrixService.logout();
console.log("Logged out");
```

---

### Sync Methods

#### sync()
```typescript
async sync(): Promise<string>
```

**Example**:
```typescript
await matrixService.sync();
console.log("Synced with server");
```

---

### Room Methods

#### getRooms()
```typescript
async getRooms(): Promise<RoomInfo[]>
```

**Example**:
```typescript
const rooms = await matrixService.getRooms();
console.log(`Found ${rooms.length} rooms`);
```

---

### Message Methods

#### getMessages(roomId, limit, fromToken?)
```typescript
async getMessages(
  roomId: string,
  limit: number = 100,
  fromToken?: string
): Promise<MessagesResponse>
```

**Example - Initial Load**:
```typescript
const response = await matrixService.getMessages(
  "!abc123:matrix.org",
  50
);
console.log(`Loaded ${response.messages.length} messages`);
```

**Example - Load More**:
```typescript
const response = await matrixService.getMessages(
  "!abc123:matrix.org",
  50,
  previousResponse.next_token
);
```

---

#### sendMessage(roomId, message)
```typescript
async sendMessage(
  roomId: string,
  message: string
): Promise<string>
```

**Example**:
```typescript
const eventId = await matrixService.sendMessage(
  "!abc123:matrix.org",
  "Hello, world!"
);
console.log("Message sent:", eventId);
```

---

### Verification Methods

#### checkVerificationStatus()
```typescript
async checkVerificationStatus(): Promise<VerificationStatus>
```

**Example**:
```typescript
const status = await matrixService.checkVerificationStatus();
if (status.needs_verification) {
  console.log("Device needs verification");
}
```

---

#### requestVerification()
```typescript
async requestVerification(): Promise<string>
```

**Example**:
```typescript
const message = await matrixService.requestVerification();
console.log(message); // "Verification request sent! Check Element on device: ..."
```

---

#### requestRecoveryKeyVerification(recoveryKey)
```typescript
async requestRecoveryKeyVerification(
  recoveryKey: string
): Promise<string>
```

**Example**:
```typescript
await matrixService.requestRecoveryKeyVerification(
  "EsTc 1234 5678 9abc..."
);
console.log("Verified with recovery key");
```

---

#### getVerificationEmoji()
```typescript
async getVerificationEmoji(): Promise<[string, string][]>
```

**Example**:
```typescript
const emoji = await matrixService.getVerificationEmoji();
console.log(emoji); // [["🐶", "dog"], ["🌊", "wave"], ...]
```

---

#### confirmVerification()
```typescript
async confirmVerification(): Promise<string>
```

**Example**:
```typescript
await matrixService.confirmVerification();
console.log("Verification confirmed!");
```

---

#### cancelVerification()
```typescript
async cancelVerification(): Promise<string>
```

**Example**:
```typescript
await matrixService.cancelVerification();
console.log("Verification cancelled");
```

---

## Error Codes

### Common Error Messages

| Error Message | Cause | Solution |
|---------------|-------|----------|
| "Not logged in" | No active session | Call `matrix_login()` first |
| "All fields are required" | Missing login parameters | Provide homeserver, username, password |
| "Invalid room ID: [reason]" | Malformed room ID | Check room ID format |
| "Room not found" | Not a member of room | Join room first |
| "No active verification" | No verification in progress | Call `request_verification()` first |
| "Waiting for other device to accept..." | Other device hasn't responded | User needs to accept on other device |
| "No other devices found" | Not logged in elsewhere | Log in on Element or another client |
| "Verification was cancelled" | User or other device cancelled | Start new verification |
| "Failed to connect: [reason]" | Network or server issue | Check homeserver URL and network |
| "Login failed: [reason]" | Invalid credentials | Check username and password |

---

## Usage Examples

### Complete Login Flow

```typescript
// Check for existing session
const existingUserId = await matrixService.checkSession();
if (existingUserId) {
  console.log("Already logged in as", existingUserId);
  return;
}

// Login
try {
  const response = await matrixService.login(
    "https://matrix.org",
    "@user:matrix.org",
    "password"
  );
  
  console.log("Logged in as", response.user_id);
  console.log("Device ID:", response.device_id);
  
  // Sync to get latest data
  await matrixService.sync();
  
  // Load rooms
  const rooms = await matrixService.getRooms();
  console.log("Rooms:", rooms);
  
} catch (error) {
  console.error("Login failed:", error);
}
```

---

### Loading Messages with Pagination

```typescript
let allMessages: Message[] = [];
let nextToken: string | undefined = undefined;
const roomId = "!abc123:matrix.org";

// Load first batch
let response = await matrixService.getMessages(roomId, 50);
allMessages = response.messages;
nextToken = response.next_token;

console.log(`Loaded ${allMessages.length} messages`);

// Load more if available
while (response.has_more && nextToken) {
  response = await matrixService.getMessages(roomId, 50, nextToken);
  allMessages = [...response.messages, ...allMessages]; // Prepend older messages
  nextToken = response.next_token;
  
  console.log(`Total messages: ${allMessages.length}`);
  
  // Optional: add delay to avoid rate limits
  await new Promise(resolve => setTimeout(resolve, 100));
}

console.log(`Finished loading ${allMessages.length} total messages`);
```

---

### Device Verification Flow

```typescript
// Check if verification needed
const status = await matrixService.checkVerificationStatus();
if (!status.needs_verification) {
  console.log("Device already verified");
  return;
}

// Request verification from another device
try {
  const message = await matrixService.requestVerification();
  console.log(message);
  
  // Poll for emoji (other device must accept first)
  let emoji: [string, string][] | null = null;
  let attempts = 0;
  const maxAttempts = 60;
  
  while (!emoji && attempts < maxAttempts) {
    try {
      emoji = await matrixService.getVerificationEmoji();
      console.log("Emoji received:", emoji);
    } catch (error) {
      console.log("Waiting for other device...");
      await new Promise(resolve => setTimeout(resolve, 1000));
      attempts++;
    }
  }
  
  if (!emoji) {
    throw new Error("Verification timed out");
  }
  
  // Display emoji to user for comparison
  // ... (UI code)
  
  // If user confirms match
  await matrixService.confirmVerification();
  console.log("Verification complete!");
  
  // Sync to download keys
  await matrixService.sync();
  
} catch (error) {
  console.error("Verification failed:", error);
}
```

---

### Using Recovery Key

```typescript
const recoveryKey = prompt("Enter your recovery key:");

if (recoveryKey) {
  try {
    await matrixService.requestRecoveryKeyVerification(recoveryKey);
    console.log("Verified with recovery key");
    
    // Sync to download keys
    await matrixService.sync();
    
    // Wait for keys to be imported
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Reload messages to see decrypted content
    const messages = await matrixService.getMessages(currentRoomId, 50);
    console.log("Messages now decrypted:", messages);
    
  } catch (error) {
    console.error("Recovery key verification failed:", error);
  }
}
```

---

### Sending Messages

```typescript
const roomId = "!abc123:matrix.org";
const messageText = "Hello, world!";

try {
  const eventId = await matrixService.sendMessage(roomId, messageText);
  console.log("Message sent with event ID:", eventId);
  
  // Reload messages to see new message
  const response = await matrixService.getMessages(roomId, 50);
  console.log("Updated messages:", response.messages);
  
} catch (error) {
  console.error("Failed to send message:", error);
}
```

---

### Handling Errors

```typescript
async function safeLogin(homeserver: string, username: string, password: string) {
  try {
    const response = await matrixService.login(homeserver, username, password);
    return { success: true, data: response };
  } catch (error) {
    // Error is a string from Rust backend
    const errorMessage = String(error);
    
    if (errorMessage.includes("Invalid credentials")) {
      return { success: false, error: "Username or password incorrect" };
    } else if (errorMessage.includes("Failed to connect")) {
      return { success: false, error: "Could not connect to homeserver" };
    } else {
      return { success: false, error: errorMessage };
    }
  }
}

// Usage
const result = await safeLogin("https://matrix.org", "@user:matrix.org", "pass");
if (result.success) {
  console.log("Logged in:", result.data.user_id);
} else {
  console.error("Login failed:", result.error);
}
```

---

## Performance Considerations

### Command Timing

Typical response times (on stable connection):

| Command | Time | Notes |
|---------|------|-------|
| check_session | <10ms | Local read only |
| matrix_login | 1-3s | Initial sync included |
| logout | 500ms-1s | Server roundtrip |
| matrix_sync | 100-500ms | Depends on updates |
| get_rooms | 50-200ms | Cached after sync |
| get_messages | 200-800ms | Depends on batch size |
| send_message | 100-300ms | Plus encryption time |
| request_verification | 500ms-1s | Device lookup |
| get_verification_emoji | 100ms-5s | Waits for other device |
| confirm_verification | 2-5s | Waits for completion |

### Optimization Tips

1. **Batch Operations**: Load multiple rooms' messages in parallel
2. **Cache Results**: Store room list and messages in state
3. **Incremental Loading**: Use pagination for message history
4. **Debounce Sync**: Don't sync too frequently (every 30s is reasonable)
5. **Loading States**: Show UI feedback during slow operations

### Rate Limiting

Matrix homeservers typically rate limit:
- Message sending: ~10-20 per second
- Sync: No limit on frequency, but not useful more than once per second
- Login attempts: ~5 per minute

**Best Practice**: Add delays between operations if doing bulk actions.

---

## Debugging Tips

### Enable Logging

Rust backend logs to console:
```rust
println!("Debug info: {}", variable);
```

Frontend logs:
```typescript
console.log("Debug info:", variable);
```

### Common Issues

**"Not logged in" errors**:
- Check if `checkSession()` returns a user ID
- Verify login completed successfully
- Check if session expired (rare)

**Messages not decrypting**:
- Verify device is verified
- Sync after verification
- Wait for key sharing (can take 10-30 seconds)
- Try closing and reopening room

**Verification timeout**:
- Ensure other device is online and visible
- Check other device for notification
- Try refreshing other device
- Fallback to recovery key

**Room not found**:
- Sync before getting rooms
- Check room ID format
- Verify user is member of room
