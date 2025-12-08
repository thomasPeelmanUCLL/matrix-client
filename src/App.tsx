import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface RoomInfo {
  room_id: string;
  name?: string;
  topic?: string;
}

interface Message {
  sender: string;
  body: string;
  timestamp: number;
}

interface LoginResponse {
  success: boolean;
  user_id: string;
  message: string;
}

function App() {
  const [homeserver, setHomeserver] = useState("https://matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);
  const [currentUser, setCurrentUser] = useState("");
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<RoomInfo | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [message, setMessage] = useState("");
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  // Check for existing session on mount
  useEffect(() => {
    checkExistingSession();
  }, []);

  async function checkExistingSession() {
    try {
      const userId = await invoke<string | null>("check_session");
      if (userId) {
        setCurrentUser(userId);
        setLoggedIn(true);
        await loadRooms();
      }
    } catch (e) {
      console.log("No existing session");
    }
  }

  async function loadRooms() {
    try {
      await invoke("matrix_sync");
      const roomList = await invoke<RoomInfo[]>("get_rooms");
      setRooms(roomList);
    } catch (error) {
      setError(`Failed to load rooms: ${error}`);
    }
  }

  async function handleLogin(e: React.FormEvent) {
    e.preventDefault();
    setIsLoading(true);
    setError("");
    setStatus("Connecting to homeserver...");

    try {
      const response = await invoke<LoginResponse>("matrix_login", {
        homeserver: homeserver.trim(),
        username: username.trim(),
        password: password,
      });

      if (response.success) {
        setCurrentUser(response.user_id);
        setLoggedIn(true);
        setStatus("Loading rooms...");
        await loadRooms();
        setStatus("");
        setPassword(""); // Clear password from memory
      }
    } catch (error) {
      setError(String(error));
      setStatus("");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleLogout() {
    try {
      await invoke("logout");
      setLoggedIn(false);
      setCurrentUser("");
      setRooms([]);
      setSelectedRoom(null);
      setPassword("");
      setError("");
      setStatus("Logged out successfully");
    } catch (error) {
      setError(`Logout failed: ${error}`);
    }
  }

  async function loadMessages(roomId: string) {
    try {
      const msgs = await invoke<Message[]>("get_messages", {
        roomId,
        limit: 50,
      });
      setMessages(msgs);
    } catch (error) {
      setError(`Error loading messages: ${error}`);
    }
  }

  useEffect(() => {
    if (selectedRoom) {
      loadMessages(selectedRoom.room_id);
    }
  }, [selectedRoom]);

  async function handleSendMessage() {
    if (!selectedRoom || !message.trim()) return;

    try {
      await invoke("send_message", {
        roomId: selectedRoom.room_id,
        message: message.trim(),
      });
      setMessage("");
      setStatus("Message sent!");
      setTimeout(() => setStatus(""), 2000);
    } catch (error) {
      setError(`Error: ${error}`);
    }
  }

  if (!loggedIn) {
    return (
      <div className="login-container">
        <div className="login-box">
          <div className="login-header">
            <h1>Matrix Client</h1>
            <p>Sign in to your Matrix account</p>
          </div>

          <form onSubmit={handleLogin} className="login-form">
            <div className="form-group">
              <label htmlFor="homeserver">Homeserver</label>
              <input
                id="homeserver"
                type="text"
                placeholder="https://matrix.org"
                value={homeserver}
                onChange={(e) => setHomeserver(e.target.value)}
                disabled={isLoading}
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="username">Username</label>
              <input
                id="username"
                type="text"
                placeholder="@username:matrix.org"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={isLoading}
                required
                autoComplete="username"
              />
            </div>

            <div className="form-group">
              <label htmlFor="password">Password</label>
              <input
                id="password"
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={isLoading}
                required
                autoComplete="current-password"
              />
            </div>

            {error && <div className="error-message">{error}</div>}
            {status && !error && <div className="info-message">{status}</div>}

            <button type="submit" className="login-button" disabled={isLoading}>
              {isLoading ? "Signing in..." : "Sign In"}
            </button>
          </form>

          <div className="login-footer">
            <p>Don't have an account? Register on your homeserver</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-layout">
      <div className="sidebar">
        <div className="sidebar-header">
          <h2>Rooms</h2>
          <button className="logout-btn" onClick={handleLogout} title="Logout">
            🚪
          </button>
        </div>
        <div className="user-info">
          <span className="user-badge">👤</span>
          <span className="user-id">{currentUser}</span>
        </div>
        <div className="rooms-container">
          {rooms.map((room) => (
            <div
              key={room.room_id}
              className={`room-item ${selectedRoom?.room_id === room.room_id ? "selected" : ""}`}
              onClick={() => setSelectedRoom(room)}
            >
              <strong>{room.name || room.room_id}</strong>
              {room.topic && <p className="room-topic">{room.topic}</p>}
            </div>
          ))}
        </div>
      </div>

      <div className="main-content">
        {selectedRoom ? (
          <>
            <div className="room-header">
              <h2>{selectedRoom.name || selectedRoom.room_id}</h2>
              <button onClick={() => loadMessages(selectedRoom.room_id)}>
                🔄 Refresh
              </button>
            </div>
            <div className="messages-area">
              {messages.length === 0 ? (
                <p style={{ color: "#8e9297" }}>No messages yet</p>
              ) : (
                messages.map((msg, idx) => (
                  <div key={idx} className="message">
                    <div className="message-header">
                      <span className="sender">{msg.sender}</span>
                      <span className="timestamp">
                        {new Date(msg.timestamp * 1000).toLocaleTimeString()}
                      </span>
                    </div>
                    <div className="message-body">{msg.body}</div>
                  </div>
                ))
              )}
            </div>
            <div className="message-input">
              <input
                type="text"
                placeholder="Type a message..."
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && handleSendMessage()}
              />
              <button onClick={handleSendMessage}>Send</button>
            </div>
          </>
        ) : (
          <div className="no-room-selected">
            <p>Select a room to start chatting</p>
          </div>
        )}
        {error && <p className="status error">{error}</p>}
        {status && <p className="status">{status}</p>}
      </div>
    </div>
  );
}

export default App;
