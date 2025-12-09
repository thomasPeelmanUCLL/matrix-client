use matrix_sdk::config::SyncSettings;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::state::MatrixState;

#[derive(Serialize, Deserialize)]
pub struct VerificationStatus {
    pub needs_verification: bool,
    pub is_verified: bool,
}

#[tauri::command]
pub async fn check_verification_status(
    state: State<'_, MatrixState>,
) -> Result<VerificationStatus, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let encryption = client.encryption();

    let status = encryption.cross_signing_status().await
        .ok_or("Cross-signing not available")?;

    let is_verified = status.is_complete();

    Ok(VerificationStatus {
        needs_verification: !is_verified,
        is_verified,
    })
}

#[tauri::command]
pub async fn request_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();

    println!("Requesting verification for user: {}", user_id);

    client.sync_once(SyncSettings::default()).await
        .map_err(|e| format!("Sync failed: {}", e))?;

    let devices = encryption
        .get_user_devices(user_id)
        .await
        .map_err(|e| format!("Failed to get devices: {}", e))?;

    println!("Found {} devices", devices.devices().count());

    let our_device_id = client.device_id().unwrap();
    let other_devices: Vec<_> = devices.devices()
        .filter(|d| d.device_id() != our_device_id)
        .collect();

    if other_devices.is_empty() {
        return Err("No other devices found. Make sure you're logged in on Element.".to_string());
    }

    println!("Found {} other devices", other_devices.len());

    for device in other_devices {
        println!("Requesting verification from device: {} ({})",
            device.device_id(),
            device.display_name().unwrap_or("Unknown"),
        );

        match device.request_verification().await {
            Ok(verification) => {
                let flow_id = verification.flow_id().to_string();
                println!("Verification requested successfully! Flow ID: {}", flow_id);

                *state.verification_flow_id.write().await = Some(flow_id.clone());

                return Ok(format!(
                    "Verification request sent! Check Element on device: {}",
                    device.display_name().unwrap_or("Unknown device"),
                ));
            }
            Err(e) => {
                println!("Failed to request from device {}: {}", device.device_id(), e);
                continue;
            }
        }
    }

    Err("Could not send verification request to any device".to_string())
}

#[tauri::command]
pub async fn get_verification_emoji(
    state: State<'_, MatrixState>,
) -> Result<Vec<(String, String)>, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;

    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();

    println!("Getting emoji for flow: {}", flow_id);

    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    println!("Verification state: is_ready={}, is_done={}, is_cancelled={}",
        verification.is_ready(),
        verification.is_done(),
        verification.is_cancelled(),
    );

    if verification.is_cancelled() {
        return Err("Verification was cancelled".to_string());
    }

    if !verification.is_ready() {
        return Err("Waiting for other device to accept...".to_string());
    }

    println!("Starting SAS verification...");
    let sas = verification.start_sas()
        .await
        .map_err(|e| format!("Failed to start SAS: {}", e))?
        .ok_or("SAS not available - other device may not support emoji")?;

    println!("SAS started, accepting...");
    sas.accept().await
        .map_err(|e| format!("Failed to accept SAS: {}", e))?;

    sleep(Duration::from_millis(1000)).await;

    if let Some(emoji) = sas.emoji() {
        let emoji_list: Vec<(String, String)> = emoji
            .iter()
            .map(|e| (e.symbol.to_string(), e.description.to_string()))
            .collect();
        println!("Got {} emoji", emoji_list.len());
        return Ok(emoji_list);
    }

    Err("Emoji not ready yet, keep polling...".to_string())
}

#[tauri::command]
pub async fn confirm_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;

    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();

    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    let sas = verification.start_sas()
        .await
        .map_err(|e| format!("Failed to get SAS: {}", e))?
        .ok_or("SAS not available")?;

    println!("Confirming verification...");
    sas.confirm()
        .await
        .map_err(|e| format!("Failed to confirm: {}", e))?;

    println!("Confirmed! Waiting for completion...");

    for _ in 0..20 {
        sleep(Duration::from_millis(500)).await;

        let verification_check = encryption
            .get_verification_request(user_id, flow_id)
            .await;

        if let Some(v) = verification_check {
            if v.is_done() {
                println!("Verification complete!");

                client.sync_once(SyncSettings::default()).await
                    .map_err(|e| format!("Sync after verification failed: {}", e))?;

                break;
            }
        }
    }

    drop(flow_id_guard);
    *state.verification_flow_id.write().await = None;

    Ok("Verification confirmed and complete!".to_string())
}

#[tauri::command]
pub async fn cancel_verification(
    state: State<'_, MatrixState>,
) -> Result<String, String> {
    let client = state.client.read().await;
    let client = client.as_ref().ok_or("Not logged in")?;

    let flow_id_guard = state.verification_flow_id.read().await;
    let flow_id = flow_id_guard.as_ref().ok_or("No active verification")?;

    let user_id = client.user_id().ok_or("No user ID")?;
    let encryption = client.encryption();

    let verification = encryption
        .get_verification_request(user_id, flow_id)
        .await
        .ok_or("Verification not found")?;

    verification
        .cancel()
        .await
        .map_err(|e| format!("Failed to cancel: {}", e))?;

    drop(flow_id_guard);
    *state.verification_flow_id.write().await = None;

    Ok("Verification cancelled".to_string())
}
