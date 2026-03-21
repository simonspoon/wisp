use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use wisp_core::{Layout, NodeType, PartialLayout, Style, Typography};

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Value,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Value,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (no id field).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

// --- Method-specific param/result types ---

/// Params for node.create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCreateParams {
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<Typography>,
}

/// Result for node.create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCreateResult {
    pub id: Uuid,
}

/// Params for node.edit — uses partial types so only explicitly-set fields are overwritten.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEditParams {
    pub id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<PartialLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<Typography>,
}

/// Params for node.delete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDeleteParams {
    pub id: Uuid,
}

/// Params for node.move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMoveParams {
    pub id: Uuid,
    pub new_parent_id: Uuid,
}

/// Params for tree.get (no params needed, returns full tree)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeGetResult {
    pub tree: String,
}

/// Params for node.show
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeShowParams {
    pub id: Uuid,
}

/// Result for node.show — full node JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeShowResult {
    pub node: wisp_core::Node,
}

/// Params for node.query — search nodes by name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeQueryParams {
    pub name: String,
}

/// Result for node.query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeQueryResult {
    pub nodes: Vec<wisp_core::Node>,
}

/// Params for component.use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUseParams {
    pub name: String,
    #[serde(default)]
    pub parent_id: Uuid,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub label: Option<String>,
    pub value: Option<String>,
}

/// Result for component.use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUseResult {
    pub ids: Vec<Uuid>,
}

/// A single component template info (for listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub description: String,
}

/// Result for component.list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentListResult {
    pub components: Vec<ComponentInfo>,
}

/// Params for doc.save
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSaveParams {
    pub path: String,
}

/// Params for doc.load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocLoadParams {
    pub path: String,
}

// --- Notification types ---

/// Notification sent when state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StateChange {
    #[serde(rename = "node.created")]
    NodeCreated { id: Uuid, parent_id: Uuid },
    #[serde(rename = "node.edited")]
    NodeEdited { id: Uuid },
    #[serde(rename = "node.deleted")]
    NodeDeleted { id: Uuid },
    #[serde(rename = "node.moved")]
    NodeMoved { id: Uuid, new_parent_id: Uuid },
}

// --- Helpers ---

impl RpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

impl RpcNotification {
    pub fn state_change(change: StateChange) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "state.changed".to_string(),
            params: serde_json::to_value(change).unwrap_or_default(),
        }
    }
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

// App-specific error codes
pub const NODE_NOT_FOUND: i32 = -32000;
pub const OPERATION_FAILED: i32 = -32001;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_rpc_request() {
        let req = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "node.create".to_string(),
            params: serde_json::json!({
                "name": "Header",
                "node_type": "frame",
                "parent_id": Uuid::nil()
            }),
            id: serde_json::json!(1),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("node.create"));
        assert!(json.contains("2.0"));
    }

    #[test]
    fn deserialize_rpc_request() {
        let json = r#"{"jsonrpc":"2.0","method":"tree.get","params":{},"id":42}"#;
        let req: RpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "tree.get");
        assert_eq!(req.id, serde_json::json!(42));
    }

    #[test]
    fn success_response() {
        let resp =
            RpcResponse::success(serde_json::json!(1), serde_json::json!({"id": Uuid::nil()}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn error_response() {
        let resp = RpcResponse::error(serde_json::json!(1), NODE_NOT_FOUND, "node not found");
        assert!(resp.result.is_none());
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32000);
    }

    #[test]
    fn node_create_params_roundtrip() {
        let params = NodeCreateParams {
            name: "Header".to_string(),
            node_type: NodeType::Frame,
            parent_id: Uuid::nil(),
            layout: None,
            style: None,
            typography: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        let decoded: NodeCreateParams = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.name, "Header");
    }

    #[test]
    fn state_change_notification() {
        let change = StateChange::NodeCreated {
            id: Uuid::nil(),
            parent_id: Uuid::nil(),
        };
        let notif = RpcNotification::state_change(change);
        assert_eq!(notif.method, "state.changed");
        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("node.created"));
    }

    #[test]
    fn node_edit_params_partial() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000000","name":"New Name"}"#;
        let params: NodeEditParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name.as_deref(), Some("New Name"));
        assert!(params.layout.is_none());
        assert!(params.style.is_none());
    }
}
