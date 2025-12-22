import { RoomInfo } from "../types";
import { RoomList } from "./RoomList";

interface SidebarProps {
  currentUser: string;
  rooms: RoomInfo[];
  selectedRoom: RoomInfo | null;
  onRoomSelect: (room: RoomInfo) => void;
  onLogout: () => void;
  isVerified: boolean | null; // null = unknown/loading
  onRetryVerification: () => void; // callback to trigger verification
}

export function Sidebar({
  currentUser,
  rooms,
  selectedRoom,
  onRoomSelect,
  onLogout,
  isVerified,
  onRetryVerification,
}: SidebarProps) {
  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2>Rooms</h2>
        <button className="logout-btn" onClick={onLogout} title="Logout">
          Logout
        </button>
      </div>

      <div className="user-info">
        <span className="user-badge">👤</span>
        <span className="user-id">{currentUser}</span>
      </div>

      <RoomList
        rooms={rooms}
        selectedRoom={selectedRoom}
        onRoomSelect={onRoomSelect}
      />
      <div className="verification-panel">
        <div className="verification-status">
          <span>Device security:</span>
          <strong>
            {isVerified === null && "Checking..."}
            {isVerified === true && "Verified ✅"}
            {isVerified === false && "Not verified ⚠️"}
          </strong>
        </div>
        <button className="verification-button" onClick={onRetryVerification}>
          {isVerified === true ? "Re-check verification" : "Try verification"}
        </button>
      </div>
    </div>
  );
}
