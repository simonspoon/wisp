use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, oneshot, Mutex};
use wisp_core::{NodeStore, UndoStack};
use wisp_protocol::RpcNotification;

/// Pending screenshot requests — maps request_id to a oneshot sender.
pub type ScreenshotBridge = Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>;

/// Callback that emits a screenshot request to the frontend.
/// Set by the Tauri app; the server calls it when a CLI requests a screenshot.
pub type ScreenshotEmitter = Arc<dyn Fn(String) + Send + Sync>;

/// Shared server state.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<NodeStore>>,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub tx: broadcast::Sender<String>,
    pub screenshot_bridge: ScreenshotBridge,
    pub screenshot_emitter: Arc<Mutex<Option<ScreenshotEmitter>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            store: Arc::new(Mutex::new(NodeStore::new())),
            undo_stack: Arc::new(Mutex::new(UndoStack::default())),
            tx,
            screenshot_bridge: Arc::new(Mutex::new(HashMap::new())),
            screenshot_emitter: Arc::new(Mutex::new(None)),
        }
    }

    /// Broadcast a notification to all connected clients.
    pub fn broadcast(&self, notification: RpcNotification) {
        let msg = serde_json::to_string(&notification).unwrap_or_default();
        let _ = self.tx.send(msg);
    }

    /// Set the screenshot emitter (called by Tauri setup with AppHandle).
    pub async fn set_screenshot_emitter(&self, emitter: ScreenshotEmitter) {
        *self.screenshot_emitter.lock().await = Some(emitter);
    }

    /// Request a screenshot from the frontend. Returns base64 PNG data.
    pub async fn request_screenshot(&self, request_id: &str) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        self.screenshot_bridge
            .lock()
            .await
            .insert(request_id.to_string(), tx);

        // Emit the request to the frontend
        let emitter = self.screenshot_emitter.lock().await;
        match &*emitter {
            Some(emit_fn) => emit_fn(request_id.to_string()),
            None => return Err("Screenshot emitter not configured (app not running?)".to_string()),
        }
        drop(emitter);

        // Wait for the frontend to deliver the screenshot (timeout 10s)
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(_)) => Err("Screenshot channel closed".to_string()),
            Err(_) => {
                self.screenshot_bridge.lock().await.remove(request_id);
                Err("Screenshot timed out (10s)".to_string())
            }
        }
    }

    /// Deliver a screenshot result from the frontend.
    pub async fn deliver_screenshot(
        &self,
        request_id: &str,
        png_base64: String,
    ) -> Result<(), String> {
        let sender = self
            .screenshot_bridge
            .lock()
            .await
            .remove(request_id)
            .ok_or_else(|| format!("No pending screenshot request: {request_id}"))?;
        sender
            .send(png_base64)
            .map_err(|_| "Failed to deliver screenshot".to_string())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
