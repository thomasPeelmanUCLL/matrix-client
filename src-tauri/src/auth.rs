use matrix_sdk::{config::SyncSettings, Client};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

use crate::state::MatrixState;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: String,
    pub device_id: String,
    pub message: String,
}

#[tauri::command]
pub async fn matrix_login(
    state: State<'_, MatrixState>,
    homeserver: String,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    if homeserver.trim().is_empty() || username.trim().is_empty() || password.is_empty() {
        return Err("All fields are required".to_string());
    }

    if !homeserver.starts_with("http://") && !homeserver.starts_with("https://") {
        return Err("Homeserver URL must start with http:// or https://".to_string());
    }

    let session_dir = state.data_dir.join(sanitize_user_id(&username));

    if session_dir.exists() {
        println!("Found existing session data, clearing...");
        fs::remove_dir_all(&session_dir)
            .map_err(|e| format!("Failed to clear old session: {}", e))?;
    }

    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    println!("Using session directory: {:?}", session_dir);

    let client = Client::builder()
        .homeserver_url(homeserver.trim())
        .sqlite_store(&session_dir, None)
        .build()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let response = client
        .matrix_auth()
        .login_username(username.trim(), &password)
        .initial_device_display_name("Matrix Client (Rust)")
        .await
        .map_err(|e| format!("Login failed: {}", e))?;

    let user_id = response.user_id.to_string();
    let device_id = response.device_id.to_string();

    println!("Logged in as {} on device {}", user_id, device_id);

    println!("Performing initial sync...");
    client
        .sync_once(SyncSettings::default())
        .await
        .map_err(|e| format!("Initial sync failed: {}", e))?;

    println!("Login and sync completed successfully");

    *state.client.write().await = Some(client);
    *state.user_id.write().await = Some(user_id.clone());

    Ok(LoginResponse {
        success: true,
        user_id,
        device_id,
        message: "Login successful - encryption enabled".to_string(),
    })
}

fn sanitize_user_id(user_id: &str) -> String {
    user_id
        .replace("@", "")
        .replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_")
}

#[tauri::command]
pub async fn check_session(state: State<'_, MatrixState>) -> Result<Option<String>, String> {
    let user_id = state.user_id.read().await;
    Ok(user_id.clone())
}

#[tauri::command]
pub async fn logout(state: State<'_, MatrixState>) -> Result<String, String> {
    let client_read = state.client.read().await;

    if let Some(client) = client_read.as_ref() {
        client.logout().await.map_err(|e| e.to_string())?;
    }
    drop(client_read);

    *state.client.write().await = None;
    *state.user_id.write().await = None;
    *state.verification_flow_id.write().await = None;

    let user_id_guard = state.user_id.read().await;
    if let Some(user_id) = user_id_guard.as_ref() {
        let session_dir = state.data_dir.join(sanitize_user_id(user_id));
        if session_dir.exists() {
            fs::remove_dir_all(&session_dir)
                .map_err(|e| format!("Failed to clear session: {}", e))?;
        }
    }

    Ok("Logged out successfully".to_string())
}

#[tauri::command]
pub async fn verify_with_recovery_key(
    state: State<'_, MatrixState>,
    recovery_key: String,
) -> Result<String, String> {
    if recovery_key.trim().is_empty() {
        return Err("Recovery key is required".to_string());
    }

    let client_guard = state.client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or("Client is not logged in")?;

    let encryption = client.encryption();
    let recovery = encryption.recovery();

    println!("Attempting to recover using recovery key...");

    recovery
        .recover(&recovery_key)
        .await
        .map_err(|e| format!("Failed to verify with recovery key: {}", e))?;

    println!("Recovery completed successfully.");

    Ok("Recovery key verification completed".to_string())
}