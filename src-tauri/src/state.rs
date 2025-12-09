use matrix_sdk::Client;
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

pub struct MatrixState {
    pub client: Arc<RwLock<Option<Client>>,
    pub user_id: Arc<RwLock<Option<String>>,
    pub pagination_tokens: Arc<RwLock<HashMap<String, String>>>,
    pub data_dir: PathBuf,
    pub verification_flow_id: Arc<RwLock<Option<String>>>,
}

impl MatrixState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            user_id: Arc::new(RwLock::new(None)),
            pagination_tokens: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
            verification_flow_id: Arc::new(RwLock::new(None)),
        }
    }
}
