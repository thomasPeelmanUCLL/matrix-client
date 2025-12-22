# Frontend Documentation - React/TypeScript Components

## Table of Contents
1. [Overview](#overview)
2. [Application Entry Points](#application-entry-points)
3. [Main Application Component](#main-application-component)
4. [Components](#components)
5. [Services](#services)
6. [Types](#types)
7. [Styling](#styling)

## Overview

The frontend is built with React 19 and TypeScript, providing a type-safe, component-based user interface for the Matrix client. The architecture follows a clear separation of concerns with components handling UI, services managing backend communication, and types ensuring type safety.

## Application Entry Points

### main.tsx
**Location**: `src/main.tsx`

**Purpose**: Application bootstrap and React initialization

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

**What it does**:
1. Imports React and ReactDOM libraries
2. Imports the main App component
3. Creates a React root attached to the "root" div in index.html
4. Renders the App component inside React.StrictMode for development checks

**React.StrictMode**: Enables additional development checks like:
- Detecting unexpected side effects
- Identifying unsafe lifecycle methods
- Warning about deprecated APIs

---

## Main Application Component

### App.tsx
**Location**: `src/App.tsx`

**Purpose**: Main application orchestrator and state manager

#### Component Structure

```typescript
function App() {
  // Authentication State
  const [loggedIn, setLoggedIn] = useState(false);
  const [currentUser, setCurrentUser] = useState("");
  
  // Room State
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<RoomInfo | null>(null);
  
  // Message State
  const [messages, setMessages] = useState<Message[]>([]);
  const [nextToken, setNextToken] = useState<string | undefined>(undefined);
  const [hasMoreMessages, setHasMoreMessages] = useState(true);
  const [isLoadingMessages, setIsLoadingMessages] = useState(false);
  
  // Verification State
  const [showVerification, setShowVerification] = useState(false);
  const [isVerified, setIsVerified] = useState<boolean | null>(null);
  const [isCheckingVerification, setIsCheckingVerification] = useState(false);
  
  // UI State
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
}
```

#### State Variables Explained

**Authentication**:
- `loggedIn`: Boolean indicating if user is authenticated
- `currentUser`: Matrix user ID (e.g., "@user:matrix.org")

**Rooms**:
- `rooms`: Array of all joined rooms
- `selectedRoom`: Currently selected room for viewing

**Messages**:
- `messages`: Array of messages for selected room
- `nextToken`: Pagination token for loading older messages
- `hasMoreMessages`: Whether more messages are available
- `isLoadingMessages`: Loading indicator for message operations

**Verification**:
- `showVerification`: Controls visibility of verification dialog
- `isVerified`: Device verification status (null=unknown, true=verified, false=not verified)
- `isCheckingVerification`: Loading state for verification check

**UI Feedback**:
- `error`: Error messages to display to user
- `status`: Status messages (success, info, loading states)

#### Key Functions

##### checkExistingSession()
```typescript
async function checkExistingSession() {
  try {
    const userId = await matrixService.checkSession();
    if (userId) {
      setCurrentUser(userId);
      setLoggedIn(true);
      await loadRooms();
    }
  } catch (e) {
    console.log("No existing session");
  }
}
```
**Purpose**: Check if user has an existing session on app startup
**Flow**: 
1. Call backend to check for saved session
2. If session exists, restore user ID and load rooms
3. If no session, user sees login screen

##### loadRooms()
```typescript
async function loadRooms() {
  try {
    setStatus("Syncing with server...");
    await matrixService.sync();
    
    setStatus("Loading rooms...");
    const roomList = await matrixService.getRooms();
    setRooms(roomList);
    setStatus("");
    
    if (roomList.length === 0) {
      setStatus("No rooms found. Try joining a room on matrix.org");
    }
  } catch (error) {
    setError(`Failed to load rooms: ${error}`);
  }
}
```
**Purpose**: Sync with server and load user's rooms
**Flow**:
1. Perform Matrix sync to get latest data
2. Fetch room list from backend
3. Update state with rooms
4. Show helpful message if no rooms found

##### loadInitialMessages(roomId)
```typescript
async function loadInitialMessages(roomId: string) {
  try {
    setIsLoadingMessages(true);
    setError("");
    setMessages([]);
    setNextToken(undefined);
    
    const response = await matrixService.getMessages(roomId, 50);
    
    setMessages(response.messages);
    setNextToken(response.next_token);
    setHasMoreMessages(response.has_more);
  } catch (error) {
    setError(`Error loading messages: ${error}`);
  } finally {
    setIsLoadingMessages(false);
  }
}
```
**Purpose**: Load initial batch of messages when room is selected
**Flow**:
1. Clear existing messages and reset pagination
2. Fetch 50 most recent messages
3. Update state with messages and pagination token
4. Set loading state appropriately

##### loadMoreMessages()
```typescript
async function loadMoreMessages() {
  if (!selectedRoom || isLoadingMessages || !hasMoreMessages || !nextToken)
    return;
    
  try {
    setIsLoadingMessages(true);
    const response = await matrixService.getMessages(
      selectedRoom.room_id,
      50,
      nextToken
    );
    
    // Deduplicate by timestamp
    const existingTimestamps = new Set(messages.map((m) => m.timestamp));
    const newMessages = response.messages.filter(
      (m) => !existingTimestamps.has(m.timestamp)
    );
    
    setMessages((prev) => [...newMessages, ...prev]);
    setNextToken(response.next_token);
    setHasMoreMessages(response.has_more);
  } catch (error) {
    setError(`Error loading more messages: ${error}`);
  } finally {
    setIsLoadingMessages(false);
  }
}
```
**Purpose**: Load older messages (backward pagination)
**Flow**:
1. Check if loading is possible (not already loading, has more messages, has token)
2. Fetch next batch using pagination token
3. Deduplicate messages by timestamp to avoid duplicates
4. Prepend new messages to existing array
5. Update pagination token for next load

##### handleLoginSuccess(userId)
```typescript
async function handleLoginSuccess(userId: string) {
  setCurrentUser(userId);
  setLoggedIn(true);
  
  await new Promise((resolve) => setTimeout(resolve, 500));
  await loadRooms();
  
  try {
    const verificationStatus = await matrixService.checkVerificationStatus();
    if (verificationStatus.needs_verification) {
      setShowVerification(true);
    }
  } catch (e) {
    console.error("Could not check verification:", e);
    setShowVerification(true);
    await refreshVerificationStatus();
  }
}
```
**Purpose**: Handle successful login and initial setup
**Flow**:
1. Update authentication state
2. Brief delay for state to settle
3. Load user's rooms
4. Check device verification status
5. Show verification dialog if needed

##### handleSendMessage(message)
```typescript
async function handleSendMessage(message: string) {
  if (!selectedRoom) return;
  
  try {
    await matrixService.sendMessage(selectedRoom.room_id, message);
    setStatus("Message sent!");
    setTimeout(() => setStatus(""), 2000);
    await loadInitialMessages(selectedRoom.room_id);
  } catch (error) {
    setError(`Error: ${error}`);
  }
}
```
**Purpose**: Send a message to the current room
**Flow**:
1. Validate room is selected
2. Send message via backend
3. Show success status
4. Reload messages to show sent message

##### handleVerificationComplete()
```typescript
async function handleVerificationComplete() {
  setShowVerification(false);
  setStatus("✅ Device verified! Syncing encryption keys...");
  
  try {
    await matrixService.sync();
    setStatus("✅ Keys synced! Reloading messages...");
    
    await new Promise((resolve) => setTimeout(resolve, 2000));
    
    if (selectedRoom) {
      await loadInitialMessages(selectedRoom.room_id);
    }
    
    setStatus("✅ Device verified and messages decrypted!");
    setTimeout(() => setStatus(""), 3000);
  } catch (error) {
    setError(`Error syncing after verification: ${error}`);
  }
}
```
**Purpose**: Handle completion of device verification
**Flow**:
1. Close verification dialog
2. Perform full sync to download encryption keys
3. Wait for keys to be processed
4. Reload current room messages (now decrypted)
5. Show success status

#### Effects (useEffect)

```typescript
// Check for existing session on mount
useEffect(() => {
  checkExistingSession();
}, []);

// Load messages when room changes
useEffect(() => {
  if (selectedRoom) {
    loadInitialMessages(selectedRoom.room_id);
  }
}, [selectedRoom]);
```

**First Effect**: Runs once on component mount, checks for existing session
**Second Effect**: Runs when selectedRoom changes, loads messages for new room

#### Render Logic

```typescript
if (!loggedIn) {
  return <Login onLoginSuccess={handleLoginSuccess} />;
}

return (
  <>
    <div className="app-layout">
      <Sidebar {...props} />
      <div className="main-content">
        {selectedRoom ? (
          <ChatView {...props} />
        ) : (
          <div className="no-room-selected">
            {/* Encouraging message */}
          </div>
        )}
      </div>
    </div>
    
    {showVerification && (
      <VerificationDialog {...props} />
    )}
  </>
);
```

**Logic**:
1. If not logged in → Show Login component
2. If logged in → Show main app layout with Sidebar and ChatView
3. If no room selected → Show placeholder message
4. If verification needed → Show modal dialog overlay

---

## Components

### Login.tsx
**Location**: `src/components/Login.tsx`

**Purpose**: User authentication interface

#### Props
```typescript
interface LoginProps {
  onLoginSuccess: (userId: string) => void;
}
```
- `onLoginSuccess`: Callback function called after successful login

#### State
```typescript
const [homeserver, setHomeserver] = useState(loginInfo.homeserver);
const [username, setUsername] = useState(loginInfo.username);
const [password, setPassword] = useState(loginInfo.password);
const [isLoading, setIsLoading] = useState(false);
const [error, setError] = useState("");
const [status, setStatus] = useState("");
```

#### Key Function: handleLogin
```typescript
async function handleLogin(e: React.FormEvent) {
  e.preventDefault();
  setIsLoading(true);
  setError("");
  setStatus("Connecting to homeserver...");
  
  try {
    const response = await matrixService.login(homeserver, username, password);
    
    if (response.success) {
      setPassword("");
      onLoginSuccess(response.user_id);
    }
  } catch (error) {
    setError(String(error));
  } finally {
    setIsLoading(false);
  }
}
```

**What it does**:
1. Prevents form default submission
2. Sets loading state and status message
3. Calls backend login function
4. On success: clears password and triggers success callback
5. On error: displays error message
6. Always clears loading state

#### UI Structure
- Form with three inputs: homeserver URL, username, password
- Submit button with loading state
- Error/status message display
- Helpful footer with supportive messaging

---

### Sidebar.tsx
**Location**: `src/components/Sidebar.tsx`

**Purpose**: Navigation sidebar with room list and user info

#### Props
```typescript
interface SidebarProps {
  currentUser: string;
  rooms: RoomInfo[];
  selectedRoom: RoomInfo | null;
  onRoomSelect: (room: RoomInfo) => void;
  onLogout: () => void;
  isVerified: boolean | null;
  onRetryVerification: () => void;
}
```

#### Structure
```typescript
return (
  <div className="sidebar">
    {/* Header with title and logout button */}
    <div className="sidebar-header">
      <h2>Rooms</h2>
      <button onClick={onLogout}>🚪</button>
    </div>
    
    {/* User information badge */}
    <div className="user-info">
      <span>👤</span>
      <span>{currentUser}</span>
    </div>
    
    {/* Room list component */}
    <RoomList
      rooms={rooms}
      selectedRoom={selectedRoom}
      onRoomSelect={onRoomSelect}
    />
    
    {/* Verification status panel */}
    <div className="verification-panel">
      <div className="verification-status">
        <span>Device security:</span>
        <strong>
          {isVerified === null && "Checking..."}
          {isVerified === true && "Verified ✅"}
          {isVerified === false && "Not verified ⚠️"}
        </strong>
      </div>
      <button onClick={onRetryVerification}>
        {isVerified === true ? "Re-check verification" : "Try verification"}
      </button>
    </div>
  </div>
);
```

**What it does**:
- Shows current user information
- Displays list of rooms
- Shows device verification status
- Provides logout and verification buttons

---

### RoomList.tsx
**Location**: `src/components/RoomList.tsx`

**Purpose**: Display list of available rooms

#### Props
```typescript
interface RoomListProps {
  rooms: RoomInfo[];
  selectedRoom: RoomInfo | null;
  onRoomSelect: (room: RoomInfo) => void;
}
```

#### Structure
```typescript
return (
  <div className="rooms-container">
    {rooms.length === 0 ? (
      <div className="no-rooms">No rooms found</div>
    ) : (
      rooms.map((room) => (
        <div
          key={room.room_id}
          className={`room-item ${
            selectedRoom?.room_id === room.room_id ? "selected" : ""
          }`}
          onClick={() => onRoomSelect(room)}
        >
          <strong>{room.name || room.room_id}</strong>
          {room.topic && <p className="room-topic">{room.topic}</p>}
        </div>
      ))
    )}
  </div>
);
```

**What it does**:
- Maps over rooms array to render list items
- Highlights selected room with CSS class
- Shows room name (or ID if no name)
- Displays room topic if available
- Handles click to select room

---

### ChatView.tsx
**Location**: `src/components/ChatView.tsx`

**Purpose**: Message display and input interface

#### Props
```typescript
interface ChatViewProps {
  room: RoomInfo;
  messages: Message[];
  onSendMessage: (message: string) => void;
  onRefresh: () => void;
  onLoadMore: () => void;
  isLoading: boolean;
  hasMore: boolean;
}
```

#### State
```typescript
const [message, setMessage] = useState("");
```
- `message`: Current text in input field

#### Key Function: handleSend
```typescript
function handleSend() {
  if (message.trim()) {
    onSendMessage(message.trim());
    setMessage("");
  }
}
```
**What it does**:
1. Validates message is not empty
2. Calls parent's send function
3. Clears input field

#### Structure
```typescript
return (
  <div className="chat-view">
    {/* Header with room name and action buttons */}
    <div className="room-header">
      <h2>{room.name || room.room_id}</h2>
      <div>
        <button onClick={onLoadMore} disabled={isLoading || !hasMore}>
          {isLoading ? "⏳ Loading..." : hasMore ? "📜 Load More" : "✓ All loaded"}
        </button>
        <button onClick={onRefresh} disabled={isLoading}>
          🔄 Refresh
        </button>
      </div>
    </div>
    
    {/* Messages area */}
    <div className="messages-area">
      {/* Loading state or message list */}
      {hasMore && (
        <button onClick={onLoadMore}>⬆️ Load older messages</button>
      )}
      
      {messages.map((msg, idx) => (
        <div key={idx} className="message">
          <div className="message-header">
            <span className="sender">{msg.sender}</span>
            <span className="timestamp">
              {new Date(msg.timestamp).toLocaleString()}
            </span>
          </div>
          <div className="message-body">{msg.body}</div>
        </div>
      ))}
    </div>
    
    {/* Message input */}
    <div className="message-input">
      <input
        type="text"
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        onKeyPress={(e) => e.key === "Enter" && handleSend()}
        disabled={isLoading}
      />
      <button onClick={handleSend} disabled={isLoading}>Send</button>
    </div>
  </div>
);
```

**What it does**:
- Displays room name in header
- Shows "Load More" button at top if more messages available
- Renders message list with sender, timestamp, and body
- Provides text input with Enter key support
- Includes send button

---

### VerificationDialog.tsx
**Location**: `src/components/VerificationDialog.tsx`

**Purpose**: Device verification wizard for end-to-end encryption

#### Props
```typescript
interface VerificationDialogProps {
  onClose: () => void;
  onVerified: () => void;
}
```

#### State
```typescript
const [step, setStep] = useState<"request" | "emoji" | "waiting" | "recovery">("request");
const [emoji, setEmoji] = useState<[string, string][]>([]);
const [error, setError] = useState("");
const [status, setStatus] = useState("");
const [recoveryKey, setRecoveryKey] = useState("");
```

**Step States**:
- `request`: Initial screen, choose verification method
- `waiting`: Waiting for other device to accept
- `emoji`: Comparing emoji between devices
- `recovery`: Recovery key input

#### Key Functions

##### startVerification()
```typescript
async function startVerification() {
  try {
    setStatus("Requesting verification...");
    await matrixService.requestVerification();
    setStep("waiting");
    setStatus("Check your other device for verification request");
    pollForEmoji();
  } catch (e) {
    setError(String(e));
  }
}
```
**What it does**:
1. Requests verification from backend
2. Changes to waiting step
3. Starts polling for emoji

##### pollForEmoji()
```typescript
async function pollForEmoji() {
  const maxAttempts = 60;
  let attempts = 0;
  
  const interval = setInterval(async () => {
    attempts++;
    
    try {
      const emojiList = await matrixService.getVerificationEmoji();
      if (emojiList && emojiList.length > 0) {
        setEmoji(emojiList);
        setStep("emoji");
        clearInterval(interval);
      }
    } catch (e) {
      // Still waiting
    }
    
    if (attempts >= maxAttempts) {
      clearInterval(interval);
      setError("Verification timed out");
    }
  }, 1000);
}
```
**What it does**:
1. Polls backend every second for emoji
2. Maximum 60 attempts (60 seconds)
3. When emoji received, displays them
4. Times out with error if no response

##### submitRecoveryKey()
```typescript
async function submitRecoveryKey() {
  if (!recoveryKey.trim()) {
    setError("Please enter your recovery key");
    return;
  }
  
  try {
    setStatus("Verifying recovery key...");
    await matrixService.requestRecoveryKeyVerification(recoveryKey.trim());
    setStatus("Recovery key verified! Syncing encryption keys...");
    await new Promise(resolve => setTimeout(resolve, 3000));
    onVerified();
  } catch (e) {
    setError(String(e));
  }
}
```
**What it does**:
1. Validates recovery key input
2. Sends to backend for verification
3. Waits for key import to complete
4. Triggers verified callback

##### confirmMatch()
```typescript
async function confirmMatch() {
  try {
    setStatus("Confirming...");
    await matrixService.confirmVerification();
    setStatus("Verified! Loading keys...");
    setTimeout(() => {
      onVerified();
    }, 2000);
  } catch (e) {
    setError(String(e));
  }
}
```
**What it does**:
1. Confirms emoji match
2. Waits for verification to complete
3. Triggers verified callback

#### UI Flow

**Step 1 - Request**: 
- Explanation of verification process
- Instructions to have Element ready
- Two buttons: "Start Verification" or "Use Recovery Key"

**Step 2 - Waiting**:
- Instructions for accepting on other device
- Loading spinner
- Cancel button

**Step 3 - Emoji**:
- Grid of 7 emoji with names
- "They Match" button (green)
- "They Don't Match" button (red)

**Step 4 - Recovery** (alternate path):
- Explanation of recovery key
- Input field for key
- "Verify with Key" button
- "Back" button

---

## Services

### matrixService.ts
**Location**: `src/services/matrixService.ts`

**Purpose**: Abstraction layer for Tauri backend commands

This service wraps all Tauri `invoke` calls, providing a clean API for components to use.

```typescript
export const matrixService = {
  // Authentication
  async login(homeserver: string, username: string, password: string): Promise<LoginResponse> {
    return await invoke<LoginResponse>("matrix_login", {
      homeserver: homeserver.trim(),
      username: username.trim(),
      password,
    });
  },
  
  async checkSession(): Promise<string | null> {
    return await invoke<string | null>("check_session");
  },
  
  async logout(): Promise<string> {
    return await invoke<string>("logout");
  },
  
  // Synchronization
  async sync(): Promise<string> {
    return await invoke<string>("matrix_sync");
  },
  
  // Rooms
  async getRooms(): Promise<RoomInfo[]> {
    return await invoke<RoomInfo[]>("get_rooms");
  },
  
  // Messages
  async getMessages(
    roomId: string,
    limit: number = 100,
    fromToken?: string
  ): Promise<MessagesResponse> {
    return await invoke<MessagesResponse>("get_messages", {
      roomId,
      limit,
      fromToken: fromToken || null,
    });
  },
  
  async sendMessage(roomId: string, message: string): Promise<string> {
    return await invoke<string>("send_message", { roomId, message });
  },
  
  // Verification
  async checkVerificationStatus(): Promise<VerificationStatus> {
    return await invoke<VerificationStatus>("check_verification_status");
  },
  
  async requestVerification(): Promise<string> {
    return await invoke<string>("request_verification");
  },
  
  async requestRecoveryKeyVerification(recoveryKey: string): Promise<string> {
    return await invoke<string>("verify_with_recovery_key", { recoveryKey });
  },
  
  async getVerificationEmoji(): Promise<[string, string][]> {
    return await invoke<[string, string][]>("get_verification_emoji");
  },
  
  async confirmVerification(): Promise<string> {
    return await invoke<string>("confirm_verification");
  },
  
  async cancelVerification(): Promise<string> {
    return await invoke<string>("cancel_verification");
  },
};
```

**Why this pattern?**:
- Centralizes backend communication
- Provides type safety with TypeScript
- Easy to mock for testing
- Single place to update if backend API changes

---

## Types

### index.ts
**Location**: `src/types/index.ts`

**Purpose**: TypeScript type definitions shared across frontend

```typescript
// Room information
export interface RoomInfo {
  room_id: string;
  name?: string;
  topic?: string;
}

// Message data
export interface Message {
  sender: string;
  body: string;
  timestamp: number;
}

// Login response from backend
export interface LoginResponse {
  success: boolean;
  user_id: string;
  device_id: string;
  message: string;
}

// Message pagination response
export interface MessagesResponse {
  messages: Message[];
  has_more: boolean;
  next_token?: string;
}

// Device verification status
export interface VerificationStatus {
  needs_verification: boolean;
  is_verified: boolean;
}
```

**Type Benefits**:
- Compile-time type checking
- IDE autocomplete
- Documentation through types
- Refactoring safety

---

## Styling

### App.css
**Location**: `src/App.css`

**Purpose**: Application-wide styles

**Key Style Patterns**:
- Dark theme with modern color palette
- Flexbox for layouts
- CSS Grid where appropriate
- Hover states for interactive elements
- Loading states and animations
- Responsive design considerations

**Color Scheme**:
- Background: Dark grays (#2c2f33, #23272a)
- Text: Light gray (#dcddde)
- Accents: Blues and greens for actions
- Error: Red tones
- Success: Green tones

**Layout Structure**:
- `.app-layout`: Main container with sidebar and content area
- `.sidebar`: Fixed width navigation
- `.main-content`: Flexible content area
- `.chat-view`: Message display area

---

## Data Flow Example

Let's trace a complete user action:

### Sending a Message

1. **User types in input** (ChatView.tsx)
   ```
   User types → input onChange → setMessage(value)
   ```

2. **User presses Enter or clicks Send**
   ```
   handleSend() → onSendMessage(message) → App.handleSendMessage()
   ```

3. **App calls service**
   ```
   App.handleSendMessage() → matrixService.sendMessage()
   ```

4. **Service invokes Tauri command**
   ```
   matrixService.sendMessage() → invoke("send_message", {...})
   ```

5. **Rust backend processes**
   ```
   Tauri → send_message() command → Matrix SDK → Homeserver
   ```

6. **Response returns**
   ```
   Homeserver → Matrix SDK → Rust → Tauri → Promise resolves
   ```

7. **UI updates**
   ```
   Success → setStatus("Message sent!") → loadInitialMessages()
   New messages loaded → setMessages() → React re-renders ChatView
   ```

This complete cycle typically takes 200-500ms depending on network latency.

---

## Best Practices Used

1. **Type Safety**: All props and state are typed
2. **Error Handling**: Try-catch blocks on all async operations
3. **Loading States**: User feedback during operations
4. **Separation of Concerns**: Components, services, types in separate files
5. **Reusable Components**: RoomList, message components
6. **Controlled Components**: Form inputs controlled by React state
7. **Effect Management**: Proper dependency arrays in useEffect
8. **Clean Code**: Clear function names, comments where needed
