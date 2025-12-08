import { RoomInfo } from "../types";

interface RoomListProps {
  rooms: RoomInfo[];
  selectedRoom: RoomInfo | null;
  onRoomSelect: (room: RoomInfo) => void;
}

export function RoomList({ rooms, selectedRoom, onRoomSelect }: RoomListProps) {
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
}
