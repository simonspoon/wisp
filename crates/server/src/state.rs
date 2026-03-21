use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use wisp_core::{NodeStore, UndoStack};
use wisp_protocol::RpcNotification;

/// Shared server state.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<NodeStore>>,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            store: Arc::new(Mutex::new(NodeStore::new())),
            undo_stack: Arc::new(Mutex::new(UndoStack::default())),
            tx,
        }
    }

    /// Broadcast a notification to all connected clients.
    pub fn broadcast(&self, notification: RpcNotification) {
        let msg = serde_json::to_string(&notification).unwrap_or_default();
        // Ignore error if no receivers
        let _ = self.tx.send(msg);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
