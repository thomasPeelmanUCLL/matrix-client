use tauri::State;
use matrix_sdk::config::SyncSettings;

use crate::state::MatrixState;

#[tauri::command]
pub async fn matrix_sync(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_lock = state.client.read().await;
    let client = client_lock.as_ref().ok_or("Not logged in")?;

    println!("Starting sync...");

    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    println!("Sync completed");

    Ok("Synced successfully".to_string())
}
