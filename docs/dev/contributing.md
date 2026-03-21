# Contributing Guide

How to build, test, and extend Wisp.

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust (stable) | Workspace build | [rustup.rs](https://rustup.rs) |
| Node.js (LTS) | Frontend build | [nodejs.org](https://nodejs.org) |
| pnpm | JS package manager | `npm install -g pnpm` |
| Xcode CLI tools | macOS native deps | `xcode-select --install` |

## Project Structure

```
wisp/
  Cargo.toml              # Workspace root (edition 2021)
  crates/
    core/                  # Document model, store, undo, tree, components
    protocol/              # JSON-RPC types, params, results, error codes
    server/                # Axum WebSocket server, RPC handler, AppState
    cli/                   # Clap CLI binary
  app/
    src-tauri/             # Tauri v2 desktop shell
    src/                   # SolidJS frontend (App.tsx, App.css)
    package.json           # pnpm project
  demo.sh                 # Full feature demo script
```

**Source**: `Cargo.toml` (workspace members), `app/package.json` (frontend deps).

## Build

```bash
# Build all Rust crates
cargo build --workspace

# Build the CLI in release mode
cargo build -p wisp-cli --release

# Install frontend dependencies (first time)
cd app && pnpm install
```

## Test

```bash
# Run all tests (54 total)
cargo test --workspace
```

Test breakdown:

| Crate | Tests | Source |
|-------|-------|--------|
| `wisp-core` | 31 | `crates/core/src/model.rs`, `store.rs`, `tree.rs`, `undo.rs`, `components.rs` |
| `wisp-protocol` | 7 | `crates/protocol/src/lib.rs` |
| `wisp-server` | 14 | `crates/server/src/handler.rs` (unit tests) |
| `wisp-server` (e2e) | 2 | `crates/server/tests/e2e.rs` (full WebSocket lifecycle) |

The e2e tests start a real WebSocket server on port 19847 and exercise the full
request/response flow including node create, edit, delete, show, query, and error
handling.

## Dev Run

Start the desktop app with hot-reload:

```bash
cd app
pnpm tauri dev
```

This runs the Vite dev server on port 1420 and the Tauri app with the WebSocket
server on port 9847. The CLI can connect immediately:

```bash
cargo run -p wisp-cli -- tree
```

## Adding a New RPC Method

When you add a new RPC method, changes ripple through four crates. Follow this
order:

### 1. Define types in protocol (`crates/protocol/src/lib.rs`)

Add param and result structs:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyMethodParams {
    pub id: Uuid,
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyMethodResult {
    // ...
}
```

### 2. Add handler in server (`crates/server/src/handler.rs`)

Add a `handle_my_method` async function following the existing pattern:
- Deserialize params with `serde_json::from_value`
- Lock `state.store` (and `state.undo_stack` if mutating)
- Snapshot for undo before mutation: `state.undo_stack.lock().await.push(&store)`
- Perform the operation
- Broadcast a notification if state changed
- Return `RpcResponse::success` or `RpcResponse::error`

Register it in the `match req.method.as_str()` dispatch block in `process_message`.

### 3. Add CLI command in cli (`crates/cli/src/main.rs`)

- Add a variant to the `Commands` enum (or `NodeAction` / `ComponentAction`)
- Add a case in `build_request()` to construct the JSON-RPC params
- Add a case in `format_response()` to display the result

### 4. Add tests

- Unit test in `handler.rs` `mod tests`
- Optionally extend the e2e test in `crates/server/tests/e2e.rs`
- Protocol roundtrip test in `crates/protocol/src/lib.rs` `mod tests`

## Adding a Component Template

Component templates live in `crates/core/src/components.rs`. Each template is a
function that creates nodes in the store and returns their IDs.

1. Add a new entry to `ComponentLibrary::new()`:

```rust
ComponentTemplate {
    name: "my-component".to_string(),
    description: "A useful component".to_string(),
    instantiate: Box::new(|store: &mut NodeStore, parent_id: Uuid| {
        let root_id = store.add("MyComponent", NodeType::Frame, parent_id)?;
        let node = store.get_mut(root_id)?;
        node.layout.width = 200.0;
        node.layout.height = 100.0;
        // ... add children ...
        Ok(vec![root_id])
    }),
}
```

2. Update the test assertion for `component.list` count in
   `crates/server/src/handler.rs` (currently expects 4 components).

3. The template is automatically available via `component.list` and `component.use`
   RPC methods and the `wisp components use` CLI command.

## Code Conventions

| Convention | Crate(s) | Example |
|-----------|----------|---------|
| `thiserror` for error types | core | `StoreError` in `crates/core/src/store.rs` |
| `clap` derive macros for CLI | cli | `Cli`, `Commands` in `crates/cli/src/main.rs` |
| `serde` derive for serialization | all | Every model type |
| `tokio::sync::Mutex` for shared state | server | `AppState` fields in `crates/server/src/state.rs` |
| Snapshot-before-mutate for undo | server | Every mutating handler pushes to `undo_stack` before changing `store` |
| Nil UUID means "use root" | server, cli | `handle_node_create`, `handle_component_use` in `handler.rs` |
| `broadcast::channel` for notifications | server | `AppState.tx` (capacity 256) |

### Workspace Dependencies

Shared dependency versions are pinned in the workspace `Cargo.toml`:

| Dependency | Version | Features |
|-----------|---------|----------|
| `serde` | 1 | `derive` |
| `serde_json` | 1 | |
| `uuid` | 1 | `v4`, `serde` |
| `thiserror` | 2 | |
| `tokio` | 1 | `full` |

**Source**: `Cargo.toml` (workspace root).
