# Wisp

A desktop design canvas that agents can see and control.

AI agents are blind to visual output — they generate UI code but can't see what it looks like. Wisp fixes that. It's a live design surface where agents build, inspect, and iterate on visual layouts through a CLI, while humans see the same canvas in real time.

**How it works:** The Wisp desktop app runs a WebSocket server. The `wisp` CLI sends JSON-RPC commands to create, edit, and arrange design nodes. Both human and agent share the same live canvas — changes appear instantly.

```
┌─────────┐     WebSocket      ┌──────────────┐
│ wisp CLI │ ───────────────── │  Wisp App    │
│ (agent)  │   JSON-RPC 2.0    │  (desktop)   │
└─────────┘                    └──────────────┘
                                     │
                               live canvas
                               renders here
```

## Quick Start

### 1. Install

**Homebrew (CLI only):**
```bash
brew install simonspoon/tap/wisp-cli
```

**GitHub Releases (CLI + desktop app):**
Download from [Releases](https://github.com/simonspoon/wisp/releases) — available for macOS (.dmg), Windows (.msi), and Linux (.deb / .AppImage).

**From source:**
```bash
# CLI
cargo build -p wisp-cli --release
# Binary at: target/release/wisp

# Desktop app (dev mode)
cd app && pnpm install && pnpm tauri dev
```

### 2. Launch the app

Open the Wisp desktop app. It starts a WebSocket server on `ws://127.0.0.1:9847/ws`.

### 3. Build something

```bash
# Create a header bar
wisp node add "Header" -t frame --width 800 --height 60 --fill "#1e40af"

# Add a title inside it
HEADER=$(wisp node add "Header" -t frame --width 800 --height 60 --fill "#1e40af" --json | jq -r .id)
wisp node add "Title" -t text --parent $HEADER -x 16 -y 16 --text "Dashboard" --font-size 24

# See the tree
wisp tree

# Capture what's on the canvas
wisp screenshot --out design.png
```

Every command updates the canvas instantly — look at the app window as you type.

## What You Can Do

| Command | What it does |
|---------|-------------|
| `wisp node add` | Create frames, text, rectangles, ellipses, groups |
| `wisp node edit` | Change any property (partial merge — only what you specify) |
| `wisp node delete` | Remove a node and its children |
| `wisp node show` | Inspect a node's full JSON |
| `wisp tree` | Print the document tree |
| `wisp components list` | List built-in templates (stat-card, button, nav-item, chart-bar) |
| `wisp components use` | Stamp down a template with custom values |
| `wisp screenshot` | Capture the canvas as a 2x PNG |
| `wisp save` / `load` | Persist and restore designs as JSON |
| `wisp undo` / `redo` | 100 levels of undo history |
| `wisp session` | Interactive REPL — keeps the connection open |
| `wisp watch` | Stream real-time change notifications |

See the [CLI Command Reference](docs/user/commands.md) for every flag and option.

## For Agents

Wisp is designed for agent workflows. An agent can:

1. **Build** a layout with `wisp node add` and `wisp components use`
2. **Inspect** the result with `wisp tree` and `wisp node show`
3. **Capture** a screenshot with `wisp screenshot` to verify visually
4. **Iterate** — edit nodes, undo mistakes, try alternatives
5. **Save** the design for later with `wisp save`

The CLI outputs structured text (or `--json` for machine-readable output), making it straightforward to integrate with any agent framework.

## Demo

The included demo script builds a full dashboard with headers, sidebars, stat cards, charts, and activity feeds:

```bash
./demo.sh
```

It exercises every major feature: node CRUD, components, partial edits, undo/redo, save/load, sessions, and screenshots.

## Architecture

4 Rust crates + a Tauri desktop app:

| Crate | Role |
|-------|------|
| `wisp-core` | Document model, undo stack, component library |
| `wisp-protocol` | JSON-RPC 2.0 types and method signatures |
| `wisp-server` | Axum WebSocket server, RPC dispatch |
| `wisp-cli` | CLI binary, WebSocket client |
| `app/` | Tauri v2 desktop shell with SolidJS frontend |

See [docs/](docs/) for full documentation — [architecture](docs/dev/architecture.md), [protocol](docs/dev/protocol.md), [contributing](docs/dev/contributing.md).

## License

MIT
