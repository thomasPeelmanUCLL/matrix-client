import { useState, useEffect } from "react";
import { Login } from "./components/Login";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { VerificationDialog } from "./components/VerificationDialog";
import { matrixService } from "./services/matrixService";
import { RoomInfo, Message } from "./types";
import "./App.css";
import { VerificationStatus } from "./types"; // if not already

function App() {
  const [loggedIn, setLoggedIn] = useState(false);
  const [currentUser, setCurrentUser] = useState("");
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<RoomInfo | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [nextToken, setNextToken] = useState<string | undefined>(undefined);
  const [hasMoreMessages, setHasMoreMessages] = useState(true);
  const [isLoadingMessages, setIsLoadingMessages] = useState(false);
  const [showVerification, setShowVerification] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");

  const [isVerified, setIsVerified] = useState<boolean | null>(null);
  const [isCheckingVerification, setIsCheckingVerification] = useState(false);

  useEffect(() => {
    checkExistingSession();
  }, []);

  useEffect(() => {
    if (selectedRoom) {
      loadInitialMessages(selectedRoom.room_id);
    }
  }, [selectedRoom]);

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

  async function refreshVerificationStatus() {
    try {
      setIsCheckingVerification(true);
      const v = await matrixService.checkVerificationStatus();
      setIsVerified(!v.needs_verification);
    } catch (e) {
      console.error("Verification status error:", e);
      setIsVerified(null);
    } finally {
      setIsCheckingVerification(false);
    }
  }

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
      setStatus("");
    }
  }

  async function loadInitialMessages(roomId: string) {
    try {
      setIsLoadingMessages(true);
      setError("");
      setMessages([]);
      setNextToken(undefined);

      const response = await matrixService.getMessages(roomId, 50);
      console.log("Initial load:", response);

      setMessages(response.messages);
      setNextToken(response.next_token);
      setHasMoreMessages(response.has_more);
    } catch (error) {
      setError(`Error loading messages: ${error}`);
    } finally {
      setIsLoadingMessages(false);
    }
  }

  async function loadMoreMessages() {
    if (!selectedRoom || isLoadingMessages || !hasMoreMessages || !nextToken)
      return;

    try {
      setIsLoadingMessages(true);
      setError("");

      console.log("Loading more with token:", nextToken);

      const response = await matrixService.getMessages(
        selectedRoom.room_id,
        50,
        nextToken
      );

      console.log("Loaded more:", response);

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

  async function handleLoginSuccess(userId: string) {
    setCurrentUser(userId);
    setLoggedIn(true);

    // Small delay for state to settle
    await new Promise((resolve) => setTimeout(resolve, 500));
    await loadRooms();

    // Check if verification is needed
    console.log("Checking verification status...");
    try {
      const verificationStatus = await matrixService.checkVerificationStatus();
      console.log("Verification status:", verificationStatus);

      if (verificationStatus.needs_verification) {
        console.log("Showing verification dialog");
        setShowVerification(true);
      }
    } catch (e) {
      console.error("Could not check verification:", e);
      // Still show the dialog as a fallback
      setShowVerification(true);

      await refreshVerificationStatus();
    }
  }
  async function handleRetryVerification() {
    // For debugging: always re-check first
    await refreshVerificationStatus();

    // If still unverified, show your existing VerificationDialog
    if (isVerified === false) {
      setShowVerification(true);
    }
  }

  async function handleLogout() {
    try {
      await matrixService.logout();
      setLoggedIn(false);
      setCurrentUser("");
      setRooms([]);
      setSelectedRoom(null);
      setMessages([]);
      setNextToken(undefined);
      setHasMoreMessages(true);
      setShowVerification(false);
      setStatus("Logged out successfully");
    } catch (error) {
      setError(`Logout failed: ${error}`);
    }
  }

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

  function handleVerificationComplete() {
    setShowVerification(false);
    setStatus("✅ Device verified! Messages should now decrypt.");

    // Reload current room messages
    if (selectedRoom) {
      setTimeout(() => {
        loadInitialMessages(selectedRoom.room_id);
      }, 1000);
    }
  }

  if (!loggedIn) {
    return <Login onLoginSuccess={handleLoginSuccess} />;
  }

  return (
    <>
      <div className="app-layout">
        <Sidebar
          currentUser={currentUser}
          rooms={rooms}
          selectedRoom={selectedRoom}
          onRoomSelect={(room) => {
            setSelectedRoom(room);
            setNextToken(undefined);
            setHasMoreMessages(true);
          }}
          onLogout={handleLogout}
          isVerified={isCheckingVerification ? null : isVerified}
          onRetryVerification={handleRetryVerification}
        />

        <div className="main-content">
          {selectedRoom ? (
            <ChatView
              room={selectedRoom}
              messages={messages}
              onSendMessage={handleSendMessage}
              onRefresh={() => loadInitialMessages(selectedRoom.room_id)}
              onLoadMore={loadMoreMessages}
              isLoading={isLoadingMessages}
              hasMore={hasMoreMessages}
            />
          ) : (
            <div className="no-room-selected">
              <div className="no-room-selected-content">
                <p className="main-message">Select a room to start chatting</p>
                <p className="encouragement">
                  <span role="img" aria-label="speech balloon">
                    💬
                  </span>{" "}
                  Every conversation starts with a single message. You belong
                  here, and your voice matters.
                </p>
                <p className="reminder">
                  Remember: Everyone feels uncertain sometimes. Reaching out is
                  brave.
                </p>
              </div>
            </div>
          )}
          {error && <p className="status error">{error}</p>}
          {status && <p className="status">{status}</p>}
        </div>
      </div>

      {showVerification && (
        <VerificationDialog
          onClose={() => setShowVerification(false)}
          onVerified={handleVerificationComplete}
        />
      )}
    </>
  );
}

export default App;
