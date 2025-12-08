import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface RoomInfo {
  room_id: string;
  name?: string;
  topic?: string;
}

function App() {
  const [homeserver, setHomeserver] = useState("https://matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<RoomInfo | null>(null);
  const [message, setMessage] = useState("");
  const [status, setStatus] = useState("");

  async function handleLogin() {
    try {
      setStatus("Logging in...");
      const result = await invoke<string>("matrix_login", {
        homeserver,
        username,
        password,
      });
      setStatus(result);
      setLoggedIn(true);
      
      await invoke("matrix_sync");
      const roomList = await invoke<RoomInfo[]>("get_rooms");
      setRooms(roomList);
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  async function handleSendMessage() {
    if (!selectedRoom || !message.trim()) return;
    
    try {
      await invoke("send_message", {
        roomId: selectedRoom.room_id,
        message: message.trim(),
      });
      setMessage("");
      setStatus("Message sent!");
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  if (!loggedIn) {
    return (
      <div className="container">
        <h1>Matrix Client</h1>
        <div className="login-form">
          <input
            type="text"
            placeholder="Homeserver (e.g., https://matrix.org)"
            value={homeserver}
            onChange={(e) => setHomeserver(e.target.value)}
          />
          <input
            type="text"
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <button onClick={handleLogin}>Login</button>
          <p className="status">{status}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="app-layout">
      <div className="sidebar">
        <h2>Rooms</h2>
        {rooms.map((room) => (
          <div
            key={room.room_id}
            className={`room-item ${selectedRoom?.room_id === room.room_id ? 'selected' : ''}`}
            onClick={() => setSelectedRoom(room)}
          >
            <strong>{room.name || room.room_id}</strong>
            {room.topic && <p className="room-topic">{room.topic}</p>}
          </div>
        ))}
      </div>
      
      <div className="main-content">
        {selectedRoom ? (
          <>
            <div className="room-header">
              <h2>{selectedRoom.name || selectedRoom.room_id}</h2>
            </div>
            <div className="messages-area">
              <p>Messages will appear here (coming next!)</p>
            </div>
            <div className="message-input">
              <input
                type="text"
                placeholder="Type a message..."
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
              />
              <button onClick={handleSendMessage}>Send</button>
            </div>
          </>
        ) : (
          <div className="no-room-selected">
            <p>Select a room to start chatting</p>
          </div>
        )}
        <p className="status">{status}</p>
      </div>
    </div>
  );
}

export default App;
