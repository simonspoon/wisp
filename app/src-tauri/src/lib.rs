use std::sync::Arc;
use tauri::{Emitter, State};
use wisp_core::{render_tree, Node, NodeType};
use wisp_protocol::*;
use wisp_server::AppState;

const WS_PORT: u16 = 9847;

// --- Tauri IPC Commands ---

#[tauri::command]
async fn get_tree(state: State<'_, AppState>) -> Result<String, String> {
    let store = state.store.lock().await;
    Ok(render_tree(&store))
}

#[tauri::command]
async fn get_nodes(state: State<'_, AppState>) -> Result<Vec<Node>, String> {
    let store = state.store.lock().await;
    Ok(store.nodes().cloned().collect())
}

#[tauri::command]
async fn get_root_id(state: State<'_, AppState>) -> Result<String, String> {
    let store = state.store.lock().await;
    Ok(store.root_id().to_string())
}

#[tauri::command]
async fn create_node(
    state: State<'_, AppState>,
    name: String,
    node_type: String,
    parent_id: String,
) -> Result<String, String> {
    let parent_uuid = parent_id
        .parse()
        .map_err(|e| format!("Invalid UUID: {e}"))?;
    let nt: NodeType = serde_json::from_value(serde_json::json!(node_type))
        .map_err(|e| format!("Invalid node type: {e}"))?;

    let mut store = state.store.lock().await;
    let id = store
        .add(&name, nt, parent_uuid)
        .map_err(|e| e.to_string())?;

    let change = StateChange::NodeCreated {
        id,
        parent_id: parent_uuid,
    };
    state.broadcast(RpcNotification::state_change(change));

    Ok(id.to_string())
}

#[tauri::command]
async fn save_document(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let store = state.store.lock().await;
    let json = serde_json::to_string_pretty(&*store).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())?;
    Ok(format!("Saved to {path} ({} bytes)", json.len()))
}

#[tauri::command]
async fn load_document(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let loaded: wisp_core::NodeStore =
        serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    let mut store = state.store.lock().await;
    *store = loaded;
    Ok(format!("Loaded from {path}"))
}

#[tauri::command]
async fn deliver_screenshot(
    state: State<'_, AppState>,
    request_id: String,
    png_base64: String,
) -> Result<(), String> {
    state.deliver_screenshot(&request_id, png_base64).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();
    let server_state = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_tree,
            get_nodes,
            get_root_id,
            create_node,
            save_document,
            load_document,
            deliver_screenshot,
        ])
        .setup(move |app| {
            // Set up screenshot emitter using AppHandle
            let handle = app.handle().clone();
            let emitter_state = server_state.clone();
            tauri::async_runtime::spawn(async move {
                emitter_state
                    .set_screenshot_emitter(Arc::new(move |request_id: String| {
                        let _ = handle.emit("screenshot-request", request_id);
                    }))
                    .await;
            });

            // Start the WebSocket server on Tauri's async runtime
            tauri::async_runtime::spawn(async move {
                if let Err(e) = wisp_server::serve(server_state, WS_PORT).await {
                    eprintln!("WebSocket server error: {e}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
