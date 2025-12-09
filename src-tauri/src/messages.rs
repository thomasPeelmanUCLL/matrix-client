use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::OwnedRoomId;
use tauri::State;

use crate::state::MatrixState;

#[tauri::command]
pub async fn send_message(
    state: State<'_, MatrixState>,
    room_id: String,
    message: String,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let room_id: OwnedRoomId = room_id
        .parse()
        .map_err(|e| format!("Invalid room ID: {}", e))?;

    let room = client
        .get_room(&room_id)
        .ok_or("Room not found")?;

    let content = RoomMessageEventContent::text_plain(message.trim());

    let response = room
        .send(content)
        .await
        .map_err(|e| format!("Failed to send: {}", e))?;

    Ok(response.event_id.to_string())
}
