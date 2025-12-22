import { useState } from "react";
import { RoomInfo, Message } from "../types";
import { matrixService } from "../services/matrixService";

interface ChatViewProps {
  room: RoomInfo;
  messages: Message[];
  onSendMessage: (message: string) => void;
  onRefresh: () => void;
  onLoadMore: () => void;
  isLoading: boolean;
  hasMore: boolean;
}

export function ChatView({ room, messages, onSendMessage, onRefresh, onLoadMore, isLoading, hasMore }: ChatViewProps) {
  const [message, setMessage] = useState("");

  function handleSend() {
    if (message.trim()) {
      onSendMessage(message.trim());
      setMessage("");
    }
  }

  return (
    <div className="chat-view">
      <div className="room-header">
  <h2>{room.name || room.room_id}</h2>
  <div>
    <button onClick={onLoadMore} disabled={isLoading || !hasMore} style={{ marginRight: "10px" }}>
      {isLoading ? "⏳ Loading..." : hasMore ? "📜 Load More" : "✓ All loaded"}
    </button>
    <button onClick={onRefresh} disabled={isLoading} style={{ marginRight: "10px" }}>
      🔄 Refresh
    </button>
    <button 
      onClick={async () => {
        try {
          await matrixService.requestRoomKeys(room.room_id);
          alert("Key request sent! Wait a moment, then refresh.");
        } catch (e) {
          alert(`Error: ${e}`);
        }
      }} 
      disabled={isLoading}
      style={{ background: "#5865f2" }}
    >
      🔑 Request Keys
    </button>
  </div>
</div>

      <div className="messages-area">
        {isLoading && messages.length === 0 ? (
          <div className="loading-container">
            <div className="spinner"></div>
            <p>Loading messages...</p>
          </div>
        ) : messages.length === 0 ? (
          <p style={{ color: "#8e9297" }}>No messages yet (click refresh)</p>
        ) : (
          <>
            {hasMore && (
              <button 
                onClick={onLoadMore}
                disabled={isLoading}
                className="load-more-top"
                style={{
                  width: "100%",
                  padding: "10px",
                  marginBottom: "10px",
                  background: isLoading ? "#2c2f33" : "#40444b",
                  border: "none",
                  borderRadius: "4px",
                  color: isLoading ? "#72767d" : "white",
                  cursor: isLoading ? "not-allowed" : "pointer"
                }}
              >
                {isLoading ? "⏳ Loading older messages..." : "⬆️ Load older messages"}
              </button>
            )}
            {!hasMore && (
              <div style={{ 
                textAlign: "center", 
                padding: "10px", 
                color: "#72767d",
                fontSize: "14px",
                marginBottom: "10px"
              }}>
                ✓ No more messages to load
              </div>
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
          </>
        )}
      </div>

      <div className="message-input">
        <input
          type="text"
          placeholder="Type a message..."
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyPress={(e) => e.key === "Enter" && handleSend()}
          disabled={isLoading}
        />
        <button onClick={handleSend} disabled={isLoading}>Send</button>
      </div>
    </div>
  );
}
