export interface RoomInfo {
  room_id: string;
  name?: string;
  topic?: string;
}

export interface Message {
  sender: string;
  body: string;
  timestamp: number;
}

export interface LoginResponse {
  success: boolean;
  user_id: string;
  device_id: string;
  message: string;
}

export interface MessagesResponse {
  messages: Message[];
  has_more: boolean;
  next_token?: string; // Add this
}

