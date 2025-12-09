use matrix_sdk::{
    config::SyncSettings,
    Client,
    room::MessagesOptions,
    ruma::{
        OwnedRoomId,
        events::room::message::RoomMessageEventContent,
    },
};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager};
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

mod state;
mod auth;
mod sync_mod;
mod rooms;
mod messages;
mod verification;

pub use state::*;
pub use auth::*;
pub use sync_mod::*;
pub use rooms::*;
pub use messages::*;
pub use verification::*;