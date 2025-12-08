import { RoomInfo } from "../types";
import { RoomList } from "./RoomList";

interface SidebarProps {
  currentUser: string;
  rooms: RoomInfo[];
  selectedRoom: RoomInfo | null;
  onRoomSelect: (room: RoomInfo) => void;
  onLogout: () => void;
}

export function Sidebar({
  currentUser,
  rooms,
  selectedRoom,
  onRoomSelect,
  onLogout,
}: SidebarProps) {
  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2>Rooms</h2>
        <button className="logout-btn" onClick={onLogout} title="Logout">
          🚪
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
    </div>
  );
  
}
