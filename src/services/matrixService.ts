import { invoke } from "@tauri-apps/api/core";
import { RoomInfo, Message, LoginResponse, MessagesResponse } from "../types";

export const matrixService = {
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

  async sync(): Promise<string> {
    return await invoke<string>("matrix_sync");
  },

  async getRooms(): Promise<RoomInfo[]> {
    return await invoke<RoomInfo[]>("get_rooms");
  },

  async getMessages(roomId: string, limit: number = 100, fromToken?: string): Promise<MessagesResponse> {
    return await invoke<MessagesResponse>("get_messages", { 
      roomId, 
      limit,
      fromToken: fromToken || null
    });
  },

  async sendMessage(roomId: string, message: string): Promise<string> {
    return await invoke<string>("send_message", { roomId, message });
  },
};
