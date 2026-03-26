# Architecture

This document describes Wisp's internal architecture: crate structure, data flow,
document model, and frontend design. Source file references are included throughout
so you can jump directly to the relevant code.

## Crate Dependency Graph

Wisp is a Cargo workspace with five crates. Dependencies flow in one direction:

```
core --> protocol --> server --> cli
                 \--> app (Tauri)
```

| Crate | Path | Purpose |
|-------|------|---------|
| `wisp-core` | `crates/core/` | Document model, node store, undo stack, tree rendering, component library |
| `wisp-protocol` | `crates/protocol/` | JSON-RPC 2.0 types, method param/result structs, error codes, notifications |
| `wisp-server` | `crates/server/` | Axum WebSocket server, RPC handler dispatch, shared `AppState` |
| `wisp-cli` | `crates/cli/` | Clap-based CLI that speaks JSON-RPC over WebSocket |
| `app` (Tauri) | `app/src-tauri/` | Tauri v2 desktop shell, IPC commands, screenshot bridge |

The frontend is a SolidJS SPA at `app/src/` bundled by Vite and served by Tauri's
webview.

## Data Flow

All mutations flow through the same path regardless of origin:

```
CLI command
    |
    v
WebSocket (JSON-RPC 2.0, port 9847)
    |
    v
wisp-server handler.rs  (process_message -> handle_*)
    |
    v
AppState.store (Arc<Mutex<NodeStore>>)
    |
    v
broadcast channel (state.changed notification)
    |
    v
All connected WebSocket clients
```

The Tauri app also exposes IPC commands (`get_tree`, `get_nodes`, `get_root_id`,
`create_node`, `edit_node`, `save_document`, `load_document`, `deliver_screenshot`)
that the SolidJS frontend calls via `@tauri-apps/api/core invoke()`. The `edit_node`
command is used by the frontend's drag-to-move and resize handles for direct
manipulation of node layout.

The WebSocket server starts inside Tauri's `setup` hook (`app/src-tauri/src/lib.rs:132`),
sharing the same `AppState` instance as the IPC commands.

**Source**: `crates/server/src/lib.rs` (router), `crates/server/src/handler.rs`
(dispatch), `app/src-tauri/src/lib.rs` (Tauri setup).

## Screenshot Pipeline

Screenshots flow through a request/response bridge between the server and the
Tauri frontend:

```
1. CLI sends "doc.screenshot" RPC
2. Server handler creates a oneshot channel, stores it in screenshot_bridge
3. Server calls screenshot_emitter (set by Tauri setup)
4. Emitter fires Tauri event "screenshot-request" with a request_id
5. Frontend listener (App.tsx:184) captures the DOM via html-to-image toPng()
6. Frontend calls IPC "deliver_screenshot" with base64 PNG data
7. Server resolves the oneshot channel, returns PNG to the CLI
8. CLI decodes base64 and writes the PNG file
```

Timeout: 10 seconds. If the frontend does not respond, the bridge entry is cleaned
up and an error is returned.

**Source**: `crates/server/src/state.rs` (ScreenshotBridge, request_screenshot,
deliver_screenshot), `app/src-tauri/src/lib.rs:136-142` (emitter setup),
`app/src/App.tsx:184-205` (DOM capture).

## Document Model

The document is a tree of nodes. Every node has the same structure regardless of
type; type-specific behavior is determined by `node_type`.

**Source**: `crates/core/src/model.rs`

### Node

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier (v4) |
| `name` | `String` | Display name |
| `node_type` | `NodeType` | Variant: Frame, Text, Rectangle, Ellipse, Group |
| `parent_id` | `Option<Uuid>` | Parent node (None for root) |
| `children` | `Vec<Uuid>` | Ordered child IDs |
| `layout` | `Layout` | Position and dimensions |
| `style` | `Style` | Visual properties |
| `typography` | `Typography` | Text-specific properties |
| `auto_layout` | `AutoLayout` | Flexbox layout properties (default: mode=none) |

### NodeType

One of: `Frame`, `Text`, `Rectangle`, `Ellipse`, `Group`. Serialized as snake_case
(`"frame"`, `"text"`, etc.).

### Layout

| Field | Type | Default |
|-------|------|---------|
| `x` | `f64` | `0.0` |
| `y` | `f64` | `0.0` |
| `width` | `f64` | `0.0` |
| `height` | `f64` | `0.0` |

### Style

All fields are `Option` -- only present fields are serialized.

| Field | Type | Description |
|-------|------|-------------|
| `fill` | `Option<String>` | Fill color as hex, e.g. `"#ff0000"` |
| `stroke` | `Option<String>` | Stroke color as hex |
| `stroke_width` | `Option<f64>` | Stroke width in pixels |
| `corner_radius` | `Option<f64>` | Corner radius in pixels |
| `opacity` | `Option<f64>` | Opacity from 0.0 to 1.0 |
| `z_index` | `Option<i32>` | Stacking order (higher = on top) |
| `clip` | `Option<bool>` | Clip children that overflow bounds (`overflow: hidden`) |

### Typography

All fields are `Option`.

| Field | Type | Description |
|-------|------|-------------|
| `content` | `Option<String>` | Text content |
| `font_family` | `Option<String>` | Font family name |
| `font_size` | `Option<f64>` | Font size in pixels |
| `font_weight` | `Option<u16>` | Font weight (e.g. 400, 700) |
| `line_height` | `Option<f64>` | Line height multiplier |
| `text_auto_size` | `Option<bool>` | When true, text wraps and height becomes auto |
| `color` | `Option<String>` | Text color as hex |
| `text_align` | `Option<TextAlign>` | Text alignment: `left` (default), `center`, `right` |

### TextAlign

One of: `Left` (default), `Center`, `Right`. Serialized as snake_case.

### AutoLayout

Flexbox-like layout for container nodes. The frontend renders these as CSS flexbox.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mode` | `LayoutMode` | `none` | `none` or `flex` |
| `direction` | `FlexDirection` | `column` | `row` or `column` |
| `align_items` | `FlexAlign` | `start` | Cross-axis alignment |
| `justify_content` | `Option<FlexAlign>` | none | Main-axis alignment |
| `gap` | `Option<f64>` | none | Gap between children in pixels |
| `padding` | `Option<f64>` | none | Uniform padding |
| `padding_horizontal` | `Option<f64>` | none | Horizontal padding (overrides `padding`) |
| `padding_vertical` | `Option<f64>` | none | Vertical padding (overrides `padding`) |

`FlexAlign` values: `start`, `center`, `end`, `stretch`, `space_between`.

### PartialLayout

Used by `node.edit` to support partial updates. Only fields that are `Some` are
merged into the existing layout.

| Field | Type |
|-------|------|
| `x` | `Option<f64>` |
| `y` | `Option<f64>` |
| `width` | `Option<f64>` |
| `height` | `Option<f64>` |

Style and Typography also support partial merges -- their `merge()` methods only
overwrite fields that are `Some` in the incoming struct.

## NodeStore

**Source**: `crates/core/src/store.rs`

`NodeStore` is a flat `HashMap<Uuid, Node>` with a designated `root_id`. It is the
single source of truth for the document tree.

```rust
pub struct NodeStore {
    nodes: HashMap<Uuid, Node>,
    root_id: Uuid,
}
```

The root node is a Frame named `"Document"` with dimensions 1920x1080, created
automatically by `NodeStore::new()`.

Key operations:

| Method | Description |
|--------|-------------|
| `add(name, node_type, parent_id)` | Create a child node. Returns new ID. |
| `add_with_id(id, name, node_type, parent_id)` | Create with a pre-assigned ID. |
| `get(id)` / `get_mut(id)` | Lookup by ID. Returns `Result<&Node, StoreError>`. |
| `delete(id)` | Remove node and all descendants. Cannot delete root. |
| `move_node(id, new_parent_id)` | Reparent a node. Rejects cycles. |
| `children(id)` | Ordered children of a node. |
| `nodes()` | Iterator over all nodes. |
| `len()` / `is_empty()` | Node count. |

Error types (`StoreError`): `NotFound(Uuid)`, `CyclicMove`, `DeleteRoot`.

NodeStore implements `Serialize`/`Deserialize` for save/load. The serialization
roundtrip is tested in `store.rs` tests.

## AppState

**Source**: `crates/server/src/state.rs`

`AppState` holds all shared mutable state for the server and is cloned into every
handler.

```rust
pub struct AppState {
    pub store: Arc<Mutex<NodeStore>>,
    pub undo_stack: Arc<Mutex<UndoStack>>,
    pub tx: broadcast::Sender<String>,
    pub screenshot_bridge: ScreenshotBridge,
    pub screenshot_emitter: Arc<Mutex<Option<ScreenshotEmitter>>>,
}
```

| Field | Type | Purpose |
|-------|------|---------|
| `store` | `Arc<Mutex<NodeStore>>` | The document |
| `undo_stack` | `Arc<Mutex<UndoStack>>` | Undo/redo history (snapshot-based) |
| `tx` | `broadcast::Sender<String>` | Notification broadcast (capacity 256) |
| `screenshot_bridge` | `Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>` | Pending screenshot requests |
| `screenshot_emitter` | `Arc<Mutex<Option<ScreenshotEmitter>>>` | Callback into Tauri to fire events |

Every mutating handler snapshots the store to the undo stack before applying changes,
then broadcasts a `state.changed` notification to all connected clients.

## Frontend

**Source**: `app/src/App.tsx`

The frontend is a SolidJS single-page application with a 3-panel layout:

| Panel | Position | Content |
|-------|----------|---------|
| Layers | Left sidebar | Tree view of the node hierarchy. Click to select. |
| Canvas | Center | Visual preview of the design. Nodes rendered as absolutely-positioned `<div>` elements. |
| Properties | Right sidebar | Details of the selected node (name, type, ID, position, size, fill, stroke, content, font size). |

### Polling

The frontend polls the Tauri backend every 500ms (`setInterval(refresh, 500)` at
`App.tsx:179`). Each poll calls three IPC commands in parallel: `get_tree`, `get_nodes`,
`get_root_id`. This is a deliberate simplicity choice -- future versions may switch
to push-based updates via the broadcast channel.

### Canvas Rendering

The canvas scales to fit the viewport. The root node's dimensions (default
1920x1080) are used to compute the scale factor. Each node is rendered by the
`CanvasNode` component. Non-flex children use `position: absolute` with the node's
layout coordinates. Flex children (`parentLayoutMode === "flex"`) use
`position: relative` and let CSS flexbox handle positioning. Style properties
(fill, stroke, corner radius, opacity, clip, z-index) and typography properties
(content, font family, font size, font weight, line height, color, text alignment)
are all applied as inline CSS. Nodes can be dragged to reposition and resized via
corner handles; both operations call the `edit_node` IPC command with partial
layout updates.

### Screenshot Capture

The frontend listens for `"screenshot-request"` Tauri events. When received, it
uses the `html-to-image` library's `toPng()` to capture the `.canvas-root` element
at 2x pixel ratio with a white background, then delivers the base64 PNG back via
the `deliver_screenshot` IPC command.
