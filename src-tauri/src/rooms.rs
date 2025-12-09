use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::room::MessagesOptions;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::MatrixState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    pub room_id: String,
    pub name: Option<String>,
    pub topic: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub body: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
    pub next_token: Option<String>,
}

#[tauri::command]
pub async fn get_rooms(state: State<'_, MatrixState>) -> Result<Vec<RoomInfo>, String> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    println!("Getting rooms for client...");

    let mut rooms_info = Vec::new();

    for room in client.rooms() {
        let name = room
            .display_name()
            .await
            .ok()
            .map(|dn| dn.to_string())
            .or_else(|| Some(room.room_id().to_string()));

        let topic = room.topic();

        rooms_info.push(RoomInfo {
            room_id: room.room_id().to_string(),
            name,
            topic,
        });
    }

    println!("Found {} rooms", rooms_info.len());

    Ok(rooms_info)
}

#[tauri::command]
pub async fn get_messages(
    state: State<'_, MatrixState>,
    room_id: String,
    _limit: u32,
    from_token: Option<String>,
) -> Result<MessagesResponse, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    println!("Getting messages for room: {}", room_id);
    println!("From token: {:?}", from_token);

    let room_id_parsed: OwnedRoomId = room_id
        .parse()
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let room = client
        .get_room(&room_id_parsed)
        .ok_or("Room not found")?;

    let options = if let Some(token) = from_token {
        MessagesOptions::backward().from(Some(token.as_str()))
    } else {
        MessagesOptions::backward()
    };

    let messages_response = room
        .messages(options)
        .await
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    println!("Received {} events from server", messages_response.chunk.len());

    let mut result = Vec::new();

    for (idx, timeline_event) in messages_response.chunk.iter().enumerate() {
        use matrix_sdk::deserialized_responses::TimelineEventKind;
        use matrix_sdk::ruma::events::{AnyTimelineEvent, AnySyncTimelineEvent, AnyMessageLikeEvent, AnySyncMessageLikeEvent};
        use matrix_sdk::ruma::events::room::message::{MessageType, RoomMessageEvent, SyncRoomMessageEvent};

        match &timeline_event.kind {
            TimelineEventKind::Decrypted(decrypted) => {
                println!("Event {}: Decrypted successfully!", idx);
                if let Ok(any_event) = decrypted.event.deserialize() {
                    if let AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(msg)) = any_event {
                        if let RoomMessageEvent::Original(original) = msg {
                            let sender = decrypted.encryption_info.sender.to_string();
                            let body = match &original.content.msgtype {
                                MessageType::Text(t) => t.body.clone(),
                                MessageType::Notice(n) => n.body.clone(),
                                MessageType::Emote(e) => format!("* {}", e.body),
                                _ => continue,
                            };

                            let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                            println!("  -> Decrypted message: {}", body);
                            result.push(Message { sender, body, timestamp });
                        }
                    }
                }
            }
            TimelineEventKind::PlainText { event } => {
                println!("Event {}: PlainText", idx);
                if let Ok(any_event) = event.deserialize() {
                    if let AnySyncTimelineEvent::MessageLike(msg) = any_event {
                        if let AnySyncMessageLikeEvent::RoomMessage(room_msg) = msg {
                            if let SyncRoomMessageEvent::Original(original) = room_msg {
                                let sender = original.sender.to_string();
                                let body = match &original.content.msgtype {
                                    MessageType::Text(t) => t.body.clone(),
                                    MessageType::Notice(n) => n.body.clone(),
                                    MessageType::Emote(e) => format!("* {}", e.body),
                                    _ => continue,
                                };

                                let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);
                                result.push(Message { sender, body, timestamp });
                            }
                        }
                    }
                }
            }
            TimelineEventKind::UnableToDecrypt { .. } => {
                println!("Event {}: UnableToDecrypt - waiting for keys", idx);

                let timestamp = timeline_event.timestamp.map(|ts| ts.get().into()).unwrap_or(0);

                result.push(Message {
                    sender: "[Encrypted]".to_string(),
                    body: "🔒 Waiting for encryption keys...".to_string(),
                    timestamp,
                });
            }
        }
    }

    result.reverse();

    println!("Parsed {} messages out of {} events", result.len(), messages_response.chunk.len());

    let next_token = messages_response.end.clone();
    let has_more = next_token.is_some() && messages_response.chunk.len() > 0;

    Ok(MessagesResponse {
        messages: result,
        has_more,
        next_token,
    })
}
