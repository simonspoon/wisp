# Wisp

Desktop UI design tool for agents. Both human and agent share the same live canvas.

Wisp is a Tauri desktop app with a companion CLI. Agents build and inspect visual UI designs via JSON-RPC over WebSocket, while the human sees the same canvas rendered live.

## Install

### From GitHub Releases

Download the latest release from [Releases](https://github.com/simonspoon/wisp/releases):

- **macOS**: `.dmg` (desktop app) + `wisp` CLI binary
- **Windows**: `.msi` (desktop app) + `wisp.exe` CLI binary
- **Linux**: `.deb` / `.AppImage` (desktop app) + `wisp` CLI binary

### From source

```bash
# Build CLI
cargo build -p wisp-cli --release
# Binary at: target/release/wisp

# Start desktop app (dev mode)
cd app
pnpm install
pnpm tauri dev
```

## Usage

1. Launch the desktop app (starts the WebSocket server on port 9847)
2. Use the CLI to create and manipulate designs:

```bash
wisp add-node --type rectangle --x 100 --y 100 --width 200 --height 150
wisp add-node --type text --x 120 --y 120 --text "Hello"
wisp tree
wisp screenshot > design.png
```

See [docs/](docs/) for full documentation.

## Architecture

4 Rust crates + Tauri app:

- **wisp-core** — Document model, undo stack, component library
- **wisp-protocol** — JSON-RPC 2.0 types and method signatures
- **wisp-server** — Axum WebSocket server, RPC handler dispatch
- **wisp-cli** — CLI binary, WebSocket client
- **app** — Tauri v2 desktop shell with SolidJS frontend

## License

MIT
