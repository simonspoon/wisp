use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;

use wisp_core::render_tree;
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
            RpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap_or(Value::Null))
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
    match store.get_mut(params.id) {
        Ok(node) => {
            if let Some(name) = params.name {
                node.name = name;
            }
            if let Some(layout) = params.layout {
                node.layout = layout;
            }
            if let Some(style) = params.style {
                node.style = style;
            }
            if let Some(typography) = params.typography {
                node.typography = typography;
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
    RpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap_or(Value::Null))
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
            RpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap_or(Value::Null))
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
    RpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap_or(Value::Null))
}

async fn handle_root_get(req: &RpcRequest, state: &AppState) -> RpcResponse {
    let store = state.store.lock().await;
    let root_id = store.root_id();
    RpcResponse::success(req.id.clone(), serde_json::json!({"root_id": root_id}))
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
