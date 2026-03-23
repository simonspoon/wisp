use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;

use wisp_core::{render_tree, ComponentLibrary};
use wisp_protocol::*;

use crate::state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast notifications
    let mut rx = state.tx.subscribe();

    // Spawn task to forward broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let response = process_message(&text, &state_clone).await;
                    if let Some(resp) = response {
                        // We need to send the response back, but sender is moved.
                        // Instead, broadcast it tagged with the connection.
                        // For simplicity in v0.1, use the broadcast channel for responses too.
                        let _ = state_clone.tx.send(resp);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn process_message(text: &str, state: &AppState) -> Option<String> {
    let req: RpcRequest = match serde_json::from_str(text) {
        Ok(r) => r,
        Err(e) => {
            let resp = RpcResponse::error(Value::Null, PARSE_ERROR, format!("Parse error: {e}"));
            return Some(serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"Serialization error: {e}"}},"id":null}}"#)
            }));
        }
    };

    let resp = match req.method.as_str() {
        "node.create" => handle_node_create(&req, state).await,
        "node.edit" => handle_node_edit(&req, state).await,
        "node.delete" => handle_node_delete(&req, state).await,
        "node.move" => handle_node_move(&req, state).await,
        "tree.get" => handle_tree_get(&req, state).await,
        "node.show" => handle_node_show(&req, state).await,
        "node.query" => handle_node_query(&req, state).await,
        "root.get" => handle_root_get(&req, state).await,
        "doc.save" => handle_doc_save(&req, state).await,
        "doc.load" => handle_doc_load(&req, state).await,
        "doc.undo" => handle_doc_undo(&req, state).await,
        "doc.redo" => handle_doc_redo(&req, state).await,
        "component.list" => handle_component_list(&req, state).await,
        "component.use" => handle_component_use(&req, state).await,
        "doc.screenshot" => handle_doc_screenshot(&req, state).await,
        _ => RpcResponse::error(
            req.id.clone(),
            METHOD_NOT_FOUND,
            format!("Unknown method: {}", req.method),
        ),
    };

    Some(serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"Serialization error: {e}"}},"id":null}}"#)
            }))
}

async fn handle_node_create(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeCreateParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let mut store = state.store.lock().await;
    // Treat nil UUID as "use root"
    let parent_id = if params.parent_id.is_nil() {
        store.root_id()
    } else {
        params.parent_id
    };
    // Snapshot for undo before mutation
    state.undo_stack.lock().await.push(&store);
    match store.add(&params.name, params.node_type, parent_id) {
        Ok(id) => {
            // Apply optional properties
            if let Ok(node) = store.get_mut(id) {
                if let Some(layout) = params.layout {
                    node.layout = layout;
                }
                if let Some(style) = params.style {
                    node.style = style;
                }
                if let Some(typography) = params.typography {
                    node.typography = typography;
                }
            }

            let change = StateChange::NodeCreated { id, parent_id };
            state.broadcast(RpcNotification::state_change(change));

            let result = NodeCreateResult { id };
            RpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap_or(Value::Null),
            )
        }
        Err(e) => RpcResponse::error(req.id.clone(), OPERATION_FAILED, e.to_string()),
    }
}

async fn handle_node_edit(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeEditParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let mut store = state.store.lock().await;
    // Snapshot for undo before mutation
    state.undo_stack.lock().await.push(&store);
    match store.get_mut(params.id) {
        Ok(node) => {
            if let Some(name) = params.name {
                node.name = name;
            }
            if let Some(ref layout) = params.layout {
                node.layout.merge(layout);
            }
            if let Some(ref style) = params.style {
                node.style.merge(style);
            }
            if let Some(ref typography) = params.typography {
                node.typography.merge(typography);
            }

            state.broadcast(RpcNotification::state_change(StateChange::NodeEdited {
                id: params.id,
            }));

            RpcResponse::success(req.id.clone(), serde_json::json!({"ok": true}))
        }
        Err(e) => RpcResponse::error(req.id.clone(), NODE_NOT_FOUND, e.to_string()),
    }
}

async fn handle_node_delete(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeDeleteParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let mut store = state.store.lock().await;
    // Snapshot for undo before mutation
    state.undo_stack.lock().await.push(&store);
    match store.delete(params.id) {
        Ok(()) => {
            state.broadcast(RpcNotification::state_change(StateChange::NodeDeleted {
                id: params.id,
            }));
            RpcResponse::success(req.id.clone(), serde_json::json!({"ok": true}))
        }
        Err(e) => RpcResponse::error(req.id.clone(), OPERATION_FAILED, e.to_string()),
    }
}

async fn handle_node_move(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeMoveParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let mut store = state.store.lock().await;
    // Snapshot for undo before mutation
    state.undo_stack.lock().await.push(&store);
    match store.move_node(params.id, params.new_parent_id) {
        Ok(()) => {
            state.broadcast(RpcNotification::state_change(StateChange::NodeMoved {
                id: params.id,
                new_parent_id: params.new_parent_id,
            }));
            RpcResponse::success(req.id.clone(), serde_json::json!({"ok": true}))
        }
        Err(e) => RpcResponse::error(req.id.clone(), OPERATION_FAILED, e.to_string()),
    }
}

async fn handle_tree_get(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let store = state.store.lock().await;
    let tree = render_tree(&store);
    let result = TreeGetResult { tree };
    RpcResponse::success(
        req.id.clone(),
        serde_json::to_value(result).unwrap_or(Value::Null),
    )
}

async fn handle_node_show(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeShowParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let store = state.store.lock().await;
    match store.get(params.id) {
        Ok(node) => {
            let result = NodeShowResult { node: node.clone() };
            RpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap_or(Value::Null),
            )
        }
        Err(e) => RpcResponse::error(req.id.clone(), NODE_NOT_FOUND, e.to_string()),
    }
}

async fn handle_node_query(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: NodeQueryParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let store = state.store.lock().await;
    let query = params.name.to_lowercase();
    let nodes: Vec<_> = store
        .nodes()
        .filter(|n| n.name.to_lowercase().contains(&query))
        .cloned()
        .collect();

    let result = NodeQueryResult { nodes };
    RpcResponse::success(
        req.id.clone(),
        serde_json::to_value(result).unwrap_or(Value::Null),
    )
}

async fn handle_root_get(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let store = state.store.lock().await;
    let root_id = store.root_id();
    RpcResponse::success(req.id.clone(), serde_json::json!({"root_id": root_id}))
}

async fn handle_doc_save(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: DocSaveParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let store = state.store.lock().await;
    let json = match serde_json::to_string_pretty(&*store) {
        Ok(j) => j,
        Err(e) => {
            return RpcResponse::error(
                req.id.clone(),
                INTERNAL_ERROR,
                format!("Serialize error: {e}"),
            );
        }
    };

    if let Err(e) = std::fs::write(&params.path, &json) {
        return RpcResponse::error(
            req.id.clone(),
            OPERATION_FAILED,
            format!("Write error: {e}"),
        );
    }

    RpcResponse::success(
        req.id.clone(),
        serde_json::json!({"path": params.path, "bytes": json.len()}),
    )
}

async fn handle_doc_undo(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let mut store = state.store.lock().await;
    let mut undo_stack = state.undo_stack.lock().await;
    match undo_stack.undo(&store) {
        Some(prev) => {
            *store = prev;
            RpcResponse::success(
                req.id.clone(),
                serde_json::json!({"ok": true, "undo_available": undo_stack.undo_count(), "redo_available": undo_stack.redo_count()}),
            )
        }
        None => RpcResponse::error(req.id.clone(), OPERATION_FAILED, "Nothing to undo"),
    }
}

async fn handle_doc_redo(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let mut store = state.store.lock().await;
    let mut undo_stack = state.undo_stack.lock().await;
    match undo_stack.redo(&store) {
        Some(next) => {
            *store = next;
            RpcResponse::success(
                req.id.clone(),
                serde_json::json!({"ok": true, "undo_available": undo_stack.undo_count(), "redo_available": undo_stack.redo_count()}),
            )
        }
        None => RpcResponse::error(req.id.clone(), OPERATION_FAILED, "Nothing to redo"),
    }
}

async fn handle_doc_screenshot(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let request_id = format!("ss-{}", uuid::Uuid::new_v4());
    match state.request_screenshot(&request_id).await {
        Ok(png_base64) => RpcResponse::success(
            req.id.clone(),
            serde_json::json!({"png_base64": png_base64}),
        ),
        Err(e) => RpcResponse::error(req.id.clone(), OPERATION_FAILED, e),
    }
}

async fn handle_component_list(req: &RpcRequest, _state: &AppState) -> RpcResponse {
    let lib = ComponentLibrary::new();
    let components: Vec<ComponentInfo> = lib
        .list()
        .iter()
        .map(|t| ComponentInfo {
            name: t.name.clone(),
            description: t.description.clone(),
        })
        .collect();

    let result = ComponentListResult { components };
    RpcResponse::success(
        req.id.clone(),
        serde_json::to_value(result).unwrap_or(Value::Null),
    )
}

async fn handle_component_use(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: ComponentUseParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let lib = ComponentLibrary::new();
    let template = match lib.get(&params.name) {
        Some(t) => t,
        None => {
            return RpcResponse::error(
                req.id.clone(),
                NODE_NOT_FOUND,
                format!("Component '{}' not found", params.name),
            );
        }
    };

    let mut store = state.store.lock().await;
    let parent_id = if params.parent_id.is_nil() {
        store.root_id()
    } else {
        params.parent_id
    };

    // Snapshot for undo before mutation
    state.undo_stack.lock().await.push(&store);

    match (template.instantiate)(&mut store, parent_id) {
        Ok(ids) => {
            // Apply x, y to root component node
            if let Some(&root_id) = ids.first() {
                if let Ok(node) = store.get_mut(root_id) {
                    if let Some(x) = params.x {
                        node.layout.x = x;
                    }
                    if let Some(y) = params.y {
                        node.layout.y = y;
                    }
                }
                // Apply label and value to child text nodes
                if let Some(label) = &params.label {
                    for &id in &ids[1..] {
                        if let Ok(node) = store.get_mut(id) {
                            if node.name == "Label" {
                                node.typography.content = Some(label.clone());
                                break;
                            }
                        }
                    }
                }
                if let Some(value) = &params.value {
                    for &id in &ids[1..] {
                        if let Ok(node) = store.get_mut(id) {
                            if node.name == "Value" {
                                node.typography.content = Some(value.clone());
                                break;
                            }
                        }
                    }
                }

                let change = StateChange::NodeCreated {
                    id: root_id,
                    parent_id,
                };
                state.broadcast(RpcNotification::state_change(change));
            }

            let result = ComponentUseResult { ids: ids.clone() };
            RpcResponse::success(
                req.id.clone(),
                serde_json::to_value(result).unwrap_or(Value::Null),
            )
        }
        Err(e) => RpcResponse::error(req.id.clone(), OPERATION_FAILED, e.to_string()),
    }
}

async fn handle_doc_load(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let params: DocLoadParams = match serde_json::from_value(req.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return RpcResponse::error(req.id.clone(), INVALID_PARAMS, e.to_string());
        }
    };

    let contents = match std::fs::read_to_string(&params.path) {
        Ok(c) => c,
        Err(e) => {
            return RpcResponse::error(
                req.id.clone(),
                OPERATION_FAILED,
                format!("Read error: {e}"),
            );
        }
    };

    let loaded: wisp_core::NodeStore = match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(e) => {
            return RpcResponse::error(
                req.id.clone(),
                OPERATION_FAILED,
                format!("Parse error: {e}"),
            );
        }
    };

    let mut store = state.store.lock().await;
    *store = loaded;

    RpcResponse::success(
        req.id.clone(),
        serde_json::json!({"path": params.path, "ok": true}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wisp_core::NodeType;

    fn make_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    async fn test_node_create() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.create".to_string(),
            params: serde_json::json!({
                "name": "Header",
                "node_type": "frame",
                "parent_id": root_id,
            }),
            id: serde_json::json!(1),
        };

        let resp = handle_node_create(&req, &state).await;
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());

        let result: NodeCreateResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        let store = state.store.lock().await;
        let node = store.get(result.id).unwrap();
        assert_eq!(node.name, "Header");
    }

    #[tokio::test]
    async fn test_node_edit() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();
        let id = state
            .store
            .lock()
            .await
            .add("Box", NodeType::Rectangle, root_id)
            .unwrap();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.edit".to_string(),
            params: serde_json::json!({
                "id": id,
                "name": "Red Box",
                "style": {"fill": "#ff0000"},
            }),
            id: serde_json::json!(2),
        };

        let resp = handle_node_edit(&req, &state).await;
        assert!(resp.error.is_none());

        let store = state.store.lock().await;
        let node = store.get(id).unwrap();
        assert_eq!(node.name, "Red Box");
        assert_eq!(node.style.fill.as_deref(), Some("#ff0000"));
    }

    #[tokio::test]
    async fn test_node_delete() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();
        let id = state
            .store
            .lock()
            .await
            .add("Temp", NodeType::Rectangle, root_id)
            .unwrap();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.delete".to_string(),
            params: serde_json::json!({"id": id}),
            id: serde_json::json!(3),
        };

        let resp = handle_node_delete(&req, &state).await;
        assert!(resp.error.is_none());
        assert_eq!(state.store.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_tree_get() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();
        state
            .store
            .lock()
            .await
            .add("Header", NodeType::Frame, root_id)
            .unwrap();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tree.get".to_string(),
            params: serde_json::json!({}),
            id: serde_json::json!(4),
        };

        let resp = handle_tree_get(&req, &state).await;
        let result: TreeGetResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        assert!(result.tree.contains("Document"));
        assert!(result.tree.contains("Header"));
    }

    #[tokio::test]
    async fn test_node_query() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();
        state
            .store
            .lock()
            .await
            .add("Header", NodeType::Frame, root_id)
            .unwrap();
        state
            .store
            .lock()
            .await
            .add("Footer", NodeType::Frame, root_id)
            .unwrap();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.query".to_string(),
            params: serde_json::json!({"name": "head"}),
            id: serde_json::json!(5),
        };

        let resp = handle_node_query(&req, &state).await;
        let result: NodeQueryResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].name, "Header");
    }

    #[tokio::test]
    async fn test_partial_edit_preserves_unset_fields() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();

        // Create a node with full layout and style
        let id = state
            .store
            .lock()
            .await
            .add("Box", NodeType::Rectangle, root_id)
            .unwrap();
        {
            let mut store = state.store.lock().await;
            let node = store.get_mut(id).unwrap();
            node.layout.x = 100.0;
            node.layout.y = 200.0;
            node.layout.width = 300.0;
            node.layout.height = 400.0;
            node.style.fill = Some("#ff0000".to_string());
            node.style.opacity = Some(0.8);
        }

        // Edit only fill — layout and other style fields must survive
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.edit".to_string(),
            params: serde_json::json!({
                "id": id,
                "style": {"fill": "#00ff00"},
            }),
            id: serde_json::json!(10),
        };
        let resp = handle_node_edit(&req, &state).await;
        assert!(resp.error.is_none());

        let store = state.store.lock().await;
        let node = store.get(id).unwrap();
        assert_eq!(node.style.fill.as_deref(), Some("#00ff00"));
        assert_eq!(node.style.opacity, Some(0.8)); // preserved
        assert_eq!(node.layout.x, 100.0); // untouched
        assert_eq!(node.layout.y, 200.0);
        assert_eq!(node.layout.width, 300.0);
        assert_eq!(node.layout.height, 400.0);
    }

    #[tokio::test]
    async fn test_partial_layout_edit() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();

        let id = state
            .store
            .lock()
            .await
            .add("Box", NodeType::Rectangle, root_id)
            .unwrap();
        {
            let mut store = state.store.lock().await;
            let node = store.get_mut(id).unwrap();
            node.layout.x = 100.0;
            node.layout.y = 200.0;
            node.layout.width = 300.0;
            node.layout.height = 400.0;
        }

        // Edit only x — y/width/height must survive
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.edit".to_string(),
            params: serde_json::json!({
                "id": id,
                "layout": {"x": 50.0},
            }),
            id: serde_json::json!(11),
        };
        let resp = handle_node_edit(&req, &state).await;
        assert!(resp.error.is_none());

        let store = state.store.lock().await;
        let node = store.get(id).unwrap();
        assert_eq!(node.layout.x, 50.0);
        assert_eq!(node.layout.y, 200.0); // preserved
        assert_eq!(node.layout.width, 300.0); // preserved
        assert_eq!(node.layout.height, 400.0); // preserved
    }

    #[tokio::test]
    async fn test_component_list() {
        let state = make_state();
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "component.list".to_string(),
            params: serde_json::json!({}),
            id: serde_json::json!(40),
        };
        let resp = handle_component_list(&req, &state).await;
        assert!(resp.error.is_none());
        let result: ComponentListResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(result.components.len(), 4);
        assert!(result.components.iter().any(|c| c.name == "stat-card"));
    }

    #[tokio::test]
    async fn test_component_use_stat_card() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();

        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "component.use".to_string(),
            params: serde_json::json!({
                "name": "stat-card",
                "parent_id": root_id,
            }),
            id: serde_json::json!(41),
        };
        let resp = handle_component_use(&req, &state).await;
        assert!(resp.error.is_none());
        let result: ComponentUseResult = serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(result.ids.len(), 4); // card + label + value + change
        assert_eq!(state.store.lock().await.len(), 5); // root + 4 component nodes
    }

    #[tokio::test]
    async fn test_component_use_unknown() {
        let state = make_state();
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "component.use".to_string(),
            params: serde_json::json!({
                "name": "nonexistent",
                "parent_id": "00000000-0000-0000-0000-000000000000",
            }),
            id: serde_json::json!(42),
        };
        let resp = handle_component_use(&req, &state).await;
        assert!(resp.error.is_some());
    }

    #[tokio::test]
    async fn test_undo_redo() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();

        // Create a node
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.create".to_string(),
            params: serde_json::json!({
                "name": "UndoTest",
                "node_type": "frame",
                "parent_id": root_id,
            }),
            id: serde_json::json!(30),
        };
        let resp = handle_node_create(&req, &state).await;
        assert!(resp.error.is_none());
        assert_eq!(state.store.lock().await.len(), 2);

        // Undo — should remove the node
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "doc.undo".to_string(),
            params: serde_json::json!({}),
            id: serde_json::json!(31),
        };
        let resp = handle_doc_undo(&req, &state).await;
        assert!(resp.error.is_none());
        assert_eq!(state.store.lock().await.len(), 1);

        // Redo — should restore the node
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "doc.redo".to_string(),
            params: serde_json::json!({}),
            id: serde_json::json!(32),
        };
        let resp = handle_doc_redo(&req, &state).await;
        assert!(resp.error.is_none());
        assert_eq!(state.store.lock().await.len(), 2);
    }

    #[tokio::test]
    async fn test_doc_save_and_load() {
        let state = make_state();
        let root_id = state.store.lock().await.root_id();
        state
            .store
            .lock()
            .await
            .add("SaveTest", NodeType::Frame, root_id)
            .unwrap();
        assert_eq!(state.store.lock().await.len(), 2);

        let tmp = std::env::temp_dir().join("wisp-test-save.json");
        let path = tmp.to_string_lossy().to_string();

        // Save
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "doc.save".to_string(),
            params: serde_json::json!({"path": &path}),
            id: serde_json::json!(20),
        };
        let resp = handle_doc_save(&req, &state).await;
        assert!(resp.error.is_none());

        // Verify file exists
        assert!(tmp.exists());

        // Clear the store
        let state2 = make_state();
        assert_eq!(state2.store.lock().await.len(), 1);

        // Load
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "doc.load".to_string(),
            params: serde_json::json!({"path": &path}),
            id: serde_json::json!(21),
        };
        let resp = handle_doc_load(&req, &state2).await;
        assert!(resp.error.is_none());
        assert_eq!(state2.store.lock().await.len(), 2);

        // Cleanup
        std::fs::remove_file(&tmp).ok();
    }

    #[tokio::test]
    async fn test_process_invalid_json() {
        let state = make_state();
        let resp = process_message("not json", &state).await.unwrap();
        let parsed: RpcResponse = serde_json::from_str(&resp).unwrap();
        assert!(parsed.error.is_some());
        assert_eq!(parsed.error.unwrap().code, PARSE_ERROR);
    }

    #[tokio::test]
    async fn test_process_unknown_method() {
        let state = make_state();
        let resp = process_message(
            r#"{"jsonrpc":"2.0","method":"foo.bar","params":{},"id":1}"#,
            &state,
        )
        .await
        .unwrap();
        let parsed: RpcResponse = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed.error.unwrap().code, METHOD_NOT_FOUND);
    }
}
