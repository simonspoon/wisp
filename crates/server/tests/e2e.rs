use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use wisp_protocol::*;
use wisp_server::AppState;

const TEST_PORT: u16 = 19847;

async fn start_server() -> AppState {
    let state = AppState::new();
    let server_state = state.clone();
    tokio::spawn(async move {
        wisp_server::serve(server_state, TEST_PORT).await.unwrap();
    });
    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    state
}

async fn send_request(method: &str, params: Value) -> RpcResponse {
    let url = format!("ws://127.0.0.1:{TEST_PORT}/ws");
    let (ws, _) = connect_async(&url).await.expect("Failed to connect");
    let (mut write, mut read) = ws.split();

    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: serde_json::json!(1),
    };

    write
        .send(Message::Text(serde_json::to_string(&req).unwrap().into()))
        .await
        .unwrap();

    // Read until we get our response (skip notifications)
    loop {
        if let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                if resp.id == serde_json::json!(1) {
                    write.close().await.ok();
                    return resp;
                }
            }
        }
    }
}

#[tokio::test]
async fn e2e_full_lifecycle() {
    let _state = start_server().await;

    // 1. Get root ID
    let resp = send_request("root.get", serde_json::json!({})).await;
    assert!(resp.error.is_none(), "root.get failed: {:?}", resp.error);
    let root_id = resp.result.unwrap()["root_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!root_id.is_empty());

    // 2. Create a node under root
    let resp = send_request(
        "node.create",
        serde_json::json!({
            "name": "Header",
            "node_type": "frame",
            "parent_id": root_id,
            "layout": {"x": 0, "y": 0, "width": 1920, "height": 80}
        }),
    )
    .await;
    assert!(resp.error.is_none(), "node.create failed: {:?}", resp.error);
    let header_id = resp.result.unwrap()["id"].as_str().unwrap().to_string();

    // 3. Create a child text node
    let resp = send_request(
        "node.create",
        serde_json::json!({
            "name": "Title",
            "node_type": "text",
            "parent_id": header_id,
            "typography": {"content": "Wisp v0.1"}
        }),
    )
    .await;
    assert!(
        resp.error.is_none(),
        "node.create text failed: {:?}",
        resp.error
    );
    let title_id = resp.result.unwrap()["id"].as_str().unwrap().to_string();

    // 4. Edit the title node
    let resp = send_request(
        "node.edit",
        serde_json::json!({
            "id": title_id,
            "name": "Main Title",
            "style": {"fill": "#1a1a2e"}
        }),
    )
    .await;
    assert!(resp.error.is_none(), "node.edit failed: {:?}", resp.error);

    // 5. Get the tree
    let resp = send_request("tree.get", serde_json::json!({})).await;
    assert!(resp.error.is_none(), "tree.get failed: {:?}", resp.error);
    let tree = resp.result.unwrap()["tree"].as_str().unwrap().to_string();
    assert!(
        tree.contains("Document"),
        "Tree should contain Document root"
    );
    assert!(tree.contains("Header"), "Tree should contain Header");
    assert!(
        tree.contains("Main Title"),
        "Tree should contain edited name 'Main Title'"
    );
    assert!(tree.contains("fill=#1a1a2e"), "Tree should show fill color");

    // 6. Show a specific node
    let resp = send_request("node.show", serde_json::json!({"id": title_id})).await;
    assert!(resp.error.is_none(), "node.show failed: {:?}", resp.error);
    let node = &resp.result.unwrap()["node"];
    assert_eq!(node["name"].as_str().unwrap(), "Main Title");
    assert_eq!(node["typography"]["content"].as_str().unwrap(), "Wisp v0.1");

    // 7. Query nodes by name
    let resp = send_request("node.query", serde_json::json!({"name": "title"})).await;
    assert!(resp.error.is_none(), "node.query failed: {:?}", resp.error);
    let nodes = resp.result.unwrap()["nodes"].as_array().unwrap().clone();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["name"].as_str().unwrap(), "Main Title");

    // 8. Delete a node
    let resp = send_request("node.delete", serde_json::json!({"id": title_id})).await;
    assert!(resp.error.is_none(), "node.delete failed: {:?}", resp.error);

    // 9. Verify tree after delete
    let resp = send_request("tree.get", serde_json::json!({})).await;
    let tree = resp.result.unwrap()["tree"].as_str().unwrap().to_string();
    assert!(
        !tree.contains("Main Title"),
        "Deleted node should not appear in tree"
    );
    assert!(tree.contains("Header"), "Parent should still exist");

    // 10. Test error handling: delete root should fail
    let resp = send_request("node.delete", serde_json::json!({"id": root_id})).await;
    assert!(resp.error.is_some(), "Deleting root should fail");

    // 11. Test error handling: unknown method
    let resp = send_request("foo.bar", serde_json::json!({})).await;
    assert!(resp.error.is_some(), "Unknown method should fail");
    assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);

    println!("All E2E tests passed!");
}

#[tokio::test]
async fn e2e_nil_uuid_uses_root() {
    // Start on a different port to avoid collision
    let state = AppState::new();
    let server_state = state.clone();
    tokio::spawn(async move {
        wisp_server::serve(server_state, TEST_PORT + 1)
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let url = format!("ws://127.0.0.1:{}/ws", TEST_PORT + 1);
    let (ws, _) = connect_async(&url).await.expect("Failed to connect");
    let (mut write, mut read) = ws.split();

    // Create node with nil UUID parent — should use root
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "node.create".to_string(),
        params: serde_json::json!({
            "name": "TestNode",
            "node_type": "frame",
            "parent_id": "00000000-0000-0000-0000-000000000000"
        }),
        id: serde_json::json!(1),
    };

    write
        .send(Message::Text(serde_json::to_string(&req).unwrap().into()))
        .await
        .unwrap();

    loop {
        if let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                if resp.id == serde_json::json!(1) {
                    assert!(
                        resp.error.is_none(),
                        "Creating node with nil parent should succeed: {:?}",
                        resp.error
                    );
                    break;
                }
            }
        }
    }

    // Verify it's under root via tree
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tree.get".to_string(),
        params: serde_json::json!({}),
        id: serde_json::json!(2),
    };

    write
        .send(Message::Text(serde_json::to_string(&req).unwrap().into()))
        .await
        .unwrap();

    loop {
        if let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(resp) = serde_json::from_str::<RpcResponse>(&text) {
                if resp.id == serde_json::json!(2) {
                    let tree = resp.result.unwrap()["tree"].as_str().unwrap().to_string();
                    assert!(
                        tree.contains("TestNode"),
                        "Node created with nil parent should appear in tree"
                    );
                    break;
                }
            }
        }
    }

    write.close().await.ok();
}
