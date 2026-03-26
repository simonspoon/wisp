# Getting Started

This guide walks you through installing Wisp, starting the app, and building your
first design with the CLI.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | LTS (20+) | [nodejs.org](https://nodejs.org) |
| pnpm | 8+ | `npm install -g pnpm` |

macOS users also need Xcode command line tools: `xcode-select --install`.

## Build the CLI

From the project root:

```bash
cargo build -p wisp-cli --release
```

The binary is at `target/release/wisp`. You can copy it to your PATH or run it
directly.

## Start the App

The Wisp desktop app hosts both the visual canvas and the WebSocket server that
the CLI connects to. Start it with:

```bash
cd app
pnpm install      # first time only
pnpm tauri dev
```

The app opens a 1280x800 window. The WebSocket server starts on port 9847.

**Source**: `app/src-tauri/tauri.conf.json` (window config, build commands).

## First Design Walkthrough

With the app running, open a terminal and try these commands. Each one sends a
JSON-RPC request to the server over WebSocket.

### 1. View the empty tree

```bash
wisp tree
```

You should see just the root `Document` frame (1920x1080).

### 2. Create a frame

```bash
wisp node add "Header" -t frame --width 1920 --height 80 --fill "#1e40af"
```

This creates a blue frame named "Header" under the root. The CLI prints the new
node's UUID.

### 3. Add text inside the header

```bash
wisp node add "Title" -t text -x 24 -y 22 --parent <header-id> \
  --text "My First Design" --font-size 28
```

Replace `<header-id>` with the UUID from step 2.

### 4. View the updated tree

```bash
wisp tree
```

The tree now shows the header with the title nested inside it. The canvas in the
app window updates automatically (it polls every 500ms).

### 5. Take a screenshot

```bash
wisp screenshot --out my-design.png
```

This captures the canvas as a 2x resolution PNG file. Requires the app to be
running with the frontend loaded.

### 6. Save the document

```bash
wisp save my-design.json
```

This writes the full document tree to a JSON file. You can reload it later with
`wisp load my-design.json`.

## Demo Script

The project includes a comprehensive demo script that builds a full dashboard
design with headers, sidebars, stat cards, charts, and activity feeds. It
exercises every major feature: node CRUD, components, partial edits, undo/redo,
save/load, interactive sessions, and screenshots.

```bash
./demo.sh
```

The script builds the CLI if needed, starts the app, waits for the server, and
runs through the full demo automatically. Press Ctrl+C to stop.

**Source**: `demo.sh` (299 lines, showcases all v0.3 features).

## What Next

- See [CLI Command Reference](commands.md) for every command and flag
- See [Architecture](../dev/architecture.md) for how the system works internally
- See [JSON-RPC Protocol](../dev/protocol.md) if you want to script Wisp directly
  over WebSocket
- See [Contributing Guide](../dev/contributing.md) to extend Wisp
