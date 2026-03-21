# JSON-RPC Protocol

Wisp uses JSON-RPC 2.0 over WebSocket for all communication between the CLI and
the server. This document covers every RPC method, notification type, and error
code.

**Source**: `crates/protocol/src/lib.rs` (types), `crates/server/src/handler.rs`
(dispatch and handling).

## Connection

- **Transport**: WebSocket
- **Endpoint**: `ws://127.0.0.1:9847/ws`
- **Protocol**: JSON-RPC 2.0 (each WebSocket text message is one JSON-RPC envelope)

The server is built with Axum. The handler splits the WebSocket into a reader and
a writer. Incoming requests are dispatched by method name; responses and
notifications are broadcast to all connected clients via a `tokio::sync::broadcast`
channel.

## Request/Response Format

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "node.create",
  "params": { ... },
  "id": 1
}
```

**Success response**:
```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": 1
}
```

**Error response**:
```json
{
  "jsonrpc": "2.0",
  "error": { "code": -32000, "message": "node not found: <uuid>" },
  "id": 1
}
```

## RPC Methods

### node.create

Create a new node in the document tree. A nil UUID (`00000000-...`) for `parent_id`
is treated as "use root."

**Params** (`NodeCreateParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | yes | Node display name |
| `node_type` | `string` | yes | One of: `frame`, `text`, `rectangle`, `ellipse`, `group` |
| `parent_id` | `uuid` | yes | Parent node ID (nil = root) |
| `layout` | `Layout` | no | Position and dimensions |
| `style` | `Style` | no | Visual properties |
| `typography` | `Typography` | no | Text properties |

**Result** (`NodeCreateResult`):

| Field | Type | Description |
|-------|------|-------------|
| `id` | `uuid` | ID of the created node |

Broadcasts: `state.changed` with kind `node.created`.

### node.edit

Edit an existing node. Uses partial merges -- only fields present in the request
are overwritten. Unset fields are preserved.

**Params** (`NodeEditParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `uuid` | yes | Node to edit |
| `name` | `string` | no | New display name |
| `layout` | `PartialLayout` | no | Only set fields are merged (x, y, width, height are each optional) |
| `style` | `Style` | no | Only set fields are merged |
| `typography` | `Typography` | no | Only set fields are merged |

**Result**: `{"ok": true}`

Broadcasts: `state.changed` with kind `node.edited`.

### node.delete

Delete a node and all its descendants. Cannot delete the root node.

**Params** (`NodeDeleteParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `uuid` | yes | Node to delete |

**Result**: `{"ok": true}`

Broadcasts: `state.changed` with kind `node.deleted`.

### node.move

Move a node to a new parent. Rejects moves that would create cycles (moving a node
under itself or one of its descendants).

**Params** (`NodeMoveParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `uuid` | yes | Node to move |
| `new_parent_id` | `uuid` | yes | New parent node ID |

**Result**: `{"ok": true}`

Broadcasts: `state.changed` with kind `node.moved`.

### tree.get

Returns a text rendering of the full document tree.

**Params**: `{}` (none)

**Result** (`TreeGetResult`):

| Field | Type | Description |
|-------|------|-------------|
| `tree` | `string` | Indented text representation of the node tree |

### node.show

Returns the full JSON representation of a single node.

**Params** (`NodeShowParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `uuid` | yes | Node to show |

**Result** (`NodeShowResult`):

| Field | Type | Description |
|-------|------|-------------|
| `node` | `Node` | Complete node object |

### node.query

Search for nodes by name (case-insensitive substring match).

**Params** (`NodeQueryParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | yes | Search query (matched against node names) |

**Result** (`NodeQueryResult`):

| Field | Type | Description |
|-------|------|-------------|
| `nodes` | `Node[]` | All nodes whose name contains the query (case-insensitive) |

### root.get

Returns the root node's UUID.

**Params**: `{}` (none)

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `root_id` | `uuid` | The document root node ID |

### doc.save

Save the document to a JSON file on disk.

**Params** (`DocSaveParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | `string` | yes | Absolute or relative file path |

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `path` | `string` | Path that was written |
| `bytes` | `number` | File size in bytes |

### doc.load

Load a document from a JSON file, replacing the current store entirely.

**Params** (`DocLoadParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | `string` | yes | File path to load |

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `path` | `string` | Path that was loaded |
| `ok` | `bool` | Always `true` on success |

### doc.undo

Undo the last mutating operation. Restores the previous store snapshot from the
undo stack.

**Params**: `{}` (none)

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `ok` | `bool` | Always `true` on success |
| `undo_available` | `number` | Remaining undo steps |
| `redo_available` | `number` | Available redo steps |

Returns error `OPERATION_FAILED` if nothing to undo.

### doc.redo

Redo the last undone operation.

**Params**: `{}` (none)

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `ok` | `bool` | Always `true` on success |
| `undo_available` | `number` | Remaining undo steps |
| `redo_available` | `number` | Available redo steps |

Returns error `OPERATION_FAILED` if nothing to redo.

### component.list

List all available component templates.

**Params**: `{}` (none)

**Result** (`ComponentListResult`):

| Field | Type | Description |
|-------|------|-------------|
| `components` | `ComponentInfo[]` | Array of `{name, description}` objects |

Currently four templates: `stat-card`, `button`, `nav-item`, `chart-bar`.

### component.use

Instantiate a component template, creating one or more nodes.

**Params** (`ComponentUseParams`):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | yes | Template name (e.g. `"stat-card"`) |
| `parent_id` | `uuid` | no | Parent node (default: nil = root) |
| `x` | `number` | no | X position for the root component node |
| `y` | `number` | no | Y position for the root component node |
| `label` | `string` | no | Override text for the component's "Label" child |
| `value` | `string` | no | Override text for the component's "Value" child |

**Result** (`ComponentUseResult`):

| Field | Type | Description |
|-------|------|-------------|
| `ids` | `uuid[]` | IDs of all created nodes (first is the root component node) |

Broadcasts: `state.changed` with kind `node.created` for the root component node.

### doc.screenshot

Capture a PNG screenshot of the frontend canvas. Requires the Tauri app to be
running with the frontend loaded.

**Params**: `{}` (none)

**Result**:

| Field | Type | Description |
|-------|------|-------------|
| `png_base64` | `string` | Base64-encoded PNG image data |

Returns error `OPERATION_FAILED` if the screenshot emitter is not configured (app
not running), the frontend does not respond within 10 seconds, or the channel is
closed.

## Notifications

Notifications are JSON-RPC messages without an `id` field, broadcast to all
connected clients.

### state.changed

Sent after every mutating operation (create, edit, delete, move, component.use,
undo, redo).

```json
{
  "jsonrpc": "2.0",
  "method": "state.changed",
  "params": {
    "kind": "node.created",
    "id": "<uuid>",
    "parent_id": "<uuid>"
  }
}
```

The `params` object is a tagged enum (`StateChange`) with these variants:

| Kind | Fields | Trigger |
|------|--------|---------|
| `node.created` | `id`, `parent_id` | `node.create`, `component.use` |
| `node.edited` | `id` | `node.edit` |
| `node.deleted` | `id` | `node.delete` |
| `node.moved` | `id`, `new_parent_id` | `node.move` |

**Source**: `crates/protocol/src/lib.rs:170-181` (StateChange enum).

## Error Codes

Standard JSON-RPC 2.0 error codes plus application-specific codes.

| Code | Constant | Description |
|------|----------|-------------|
| `-32700` | `PARSE_ERROR` | Invalid JSON in the request |
| `-32600` | `INVALID_REQUEST` | Request is not a valid JSON-RPC object |
| `-32601` | `METHOD_NOT_FOUND` | Unknown RPC method name |
| `-32602` | `INVALID_PARAMS` | Params failed to deserialize into the expected type |
| `-32603` | `INTERNAL_ERROR` | Internal server error (e.g. serialization failure) |
| `-32000` | `NODE_NOT_FOUND` | Referenced node ID does not exist in the store |
| `-32001` | `OPERATION_FAILED` | Operation-specific failure (delete root, cyclic move, undo with empty stack, etc.) |

**Source**: `crates/protocol/src/lib.rs:219-228`.
