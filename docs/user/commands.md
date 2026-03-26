# CLI Command Reference

Complete reference for the `wisp` command-line tool. Every command communicates
with the Wisp app over WebSocket (JSON-RPC 2.0).

**Source**: `crates/cli/src/main.rs`

## Global Flags

These flags apply to all commands.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--url` | `string` | `ws://127.0.0.1:9847/ws` | WebSocket server URL |
| `--json` | `bool` | `false` | Output raw JSON instead of formatted text |

## wisp node add

Create a new node in the document tree.

```
wisp node add <name> [flags]
```

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `<name>` | | `string` | (required) | Node display name |
| `--node-type` | `-t` | `string` | `frame` | Node type: `frame`, `text`, `rectangle`, `ellipse`, `group` |
| `--parent` | `-p` | `uuid` | root | Parent node ID (omit to add under root) |
| `--x` | `-x` | `f64` | `0.0` | X position |
| `--y` | `-y` | `f64` | `0.0` | Y position |
| `--width` | | `f64` | `0.0` | Width in pixels |
| `--height` | | `f64` | `0.0` | Height in pixels |
| `--fill` | | `string` | none | Fill color as hex (e.g. `"#ff0000"`) |
| `--stroke` | | `string` | none | Stroke color as hex |
| `--stroke-width` | | `f64` | none | Stroke width in pixels |
| `--text` | | `string` | none | Text content (for text nodes) |
| `--font-size` | | `f64` | none | Font size in pixels |
| `--font-family` | | `string` | none | Font family name (e.g. `"monospace"`) |
| `--font-weight` | | `u16` | none | Font weight (e.g. `400`, `700`) |
| `--color` | | `string` | none | Text color as hex |
| `--text-align` | | `string` | none | Text alignment: `left`, `center`, `right` |
| `--radius` | | `f64` | none | Corner radius in pixels |
| `--opacity` | | `f64` | none | Opacity from 0.0 to 1.0 |
| `--z-index` | | `i32` | none | Stacking order (higher = on top) |
| `--clip` | | `bool` | `false` | Clip children that overflow bounds |
| `--text-wrap` | | `bool` | `false` | Enable text wrapping (auto-size height) |
| `--layout-mode` | | `string` | none | Layout mode: `none`, `flex` |
| `--direction` | | `string` | none | Flex direction: `row`, `column` |
| `--align` | | `string` | none | Flex align items: `start`, `center`, `end`, `stretch` |
| `--justify` | | `string` | none | Flex justify content: `start`, `center`, `end`, `stretch`, `space_between` |
| `--gap` | | `f64` | none | Gap between flex children |
| `--padding` | | `f64` | none | Padding (all sides) |

Layout fields (`x`, `y`, `width`, `height`) are only sent if at least one is
specified. Style, typography, and auto-layout fields follow the same rule — each
group is only included if at least one field in the group is set.

**Output**: `Created node <uuid>`

**Example**:
```bash
wisp node add "Card" -t rectangle -x 100 -y 50 --width 300 --height 200 \
  --fill "#f0f0f0" --radius 12

# Flex container with children
wisp node add "Row" -t frame --layout-mode flex --direction row --gap 8 --padding 12

# Styled text
wisp node add "Title" -t text --text "Hello" --color "#ff0000" --font-family "monospace" --text-align center
```

## wisp node edit

Edit an existing node. Only specified fields are changed; all other properties
are preserved (partial merge).

```
wisp node edit <id> [flags]
```

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `<id>` | | `uuid` | (required) Node ID to edit |
| `--name` | | `string` | New display name |
| `--fill` | | `string` | Fill color |
| `--stroke` | | `string` | Stroke color |
| `--stroke-width` | | `f64` | Stroke width in pixels |
| `--x` | `-x` | `f64` | X position |
| `--y` | `-y` | `f64` | Y position |
| `--width` | | `f64` | Width |
| `--height` | | `f64` | Height |
| `--text` | | `string` | Text content |
| `--font-size` | | `f64` | Font size |
| `--font-family` | | `string` | Font family |
| `--font-weight` | | `u16` | Font weight |
| `--color` | | `string` | Text color |
| `--text-align` | | `string` | Text alignment: `left`, `center`, `right` |
| `--radius` | | `f64` | Corner radius |
| `--opacity` | | `f64` | Opacity |
| `--z-index` | | `i32` | Stacking order |
| `--clip` | | `bool` | Clip overflow |
| `--text-wrap` | | `bool` | Enable text wrapping |
| `--layout-mode` | | `string` | Layout mode: `none`, `flex` |
| `--direction` | | `string` | Flex direction: `row`, `column` |
| `--align` | | `string` | Flex align items |
| `--justify` | | `string` | Flex justify content |
| `--gap` | | `f64` | Flex gap |
| `--padding` | | `f64` | Padding |

Layout, style, typography, and auto-layout are each sent as partial objects --
only the fields you specify are included, and only those fields are merged on the
server. For example, `wisp node edit <id> --fill "#red"` changes only the fill;
position, size, and all other style fields remain unchanged.

**Output**: `Node updated`

**Example**:
```bash
wisp node edit abc123-... --fill "#00ff00" -x 200
```

## wisp node delete

Delete a node and all its descendants. Cannot delete the root node.

```
wisp node delete <id>
```

| Argument | Type | Description |
|----------|------|-------------|
| `<id>` | `uuid` | (required) Node ID to delete |

**Output**: `Node deleted`

## wisp node show

Display the full JSON details of a node.

```
wisp node show <id>
```

| Argument | Type | Description |
|----------|------|-------------|
| `<id>` | `uuid` | (required) Node ID to show |

**Output**: Pretty-printed JSON of the node object (id, name, node_type, parent_id,
children, layout, style, typography).

## wisp tree

Print the full document tree as indented text.

```
wisp tree
```

No arguments. Shows all nodes with their names, types, dimensions, and style
properties.

**Output**: Indented tree text (one line per node).

## wisp save

Save the current document to a JSON file.

```
wisp save <path>
```

| Argument | Type | Description |
|----------|------|-------------|
| `<path>` | `string` | (required) File path to save to |

The path is canonicalized to an absolute path before sending to the server.

**Output**: `Saved to <path> (<bytes> bytes)`

## wisp load

Load a document from a JSON file, replacing the current document entirely.

```
wisp load <path>
```

| Argument | Type | Description |
|----------|------|-------------|
| `<path>` | `string` | (required) File path to load from |

The path must exist. It is canonicalized before sending.

**Output**: `Loaded from <path>`

## wisp undo

Undo the last mutating operation. Each create, edit, delete, move, and component
instantiation pushes a snapshot to the undo stack.

```
wisp undo
```

**Output**: `Undone (undo: <n>, redo: <n>)`

Returns an error if there is nothing to undo.

## wisp redo

Redo the last undone operation.

```
wisp redo
```

**Output**: `Redone (undo: <n>, redo: <n>)`

Returns an error if there is nothing to redo.

## wisp components list

List all available component templates.

```
wisp components list
```

**Output**: One line per template with name and description. Currently available:

| Template | Description |
|----------|-------------|
| `stat-card` | Stat card with label, value, and change text (4 nodes) |
| `button` | Button component (2 nodes) |
| `nav-item` | Navigation item (2 nodes) |
| `chart-bar` | Chart bar (2 nodes) |

## wisp components use

Instantiate a component template, creating one or more nodes.

```
wisp components use <name> [flags]
```

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `<name>` | | `string` | (required) | Template name (e.g. `stat-card`) |
| `--parent` | `-p` | `uuid` | root | Parent node ID |
| `--x` | `-x` | `f64` | none | X position for the root component node |
| `--y` | `-y` | `f64` | none | Y position for the root component node |
| `--label` | | `string` | none | Override the component's "Label" text |
| `--value` | | `string` | none | Override the component's "Value" text |

**Output**: `Created <name> (<n> nodes)` followed by `root: <uuid>`.

**Example**:
```bash
wisp components use stat-card --parent $MAIN_ID -x 32 -y 32 \
  --label "Revenue" --value "$1,234"
```

## wisp screenshot

Capture a PNG screenshot of the canvas. Requires the Tauri app to be running with
the frontend loaded.

```
wisp screenshot [flags]
```

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--out` | `-o` | `string` | `wisp-screenshot.png` | Output file path |

The screenshot is captured at 2x pixel ratio with a white background via the
`html-to-image` library in the frontend. The server has a 10-second timeout.

**Output**: `Screenshot saved to <path> (<bytes> bytes)`

## wisp session

Start an interactive session that keeps the WebSocket connection open for multiple
commands. Useful for scripting or rapid iteration.

```
wisp session
```

Inside the session, type any regular wisp command without the `wisp` prefix:

```
wisp> tree
wisp> node add "Box" -t rectangle --fill "#ff0000"
wisp> undo
wisp> quit
```

Special session commands:
- `quit` or `exit` -- end the session
- `session` and `watch` cannot be nested

The session respects the `--json` global flag if passed on the initial invocation.

## wisp watch

Stream real-time state change notifications until interrupted.

```
wisp watch
```

Prints one line per notification:

```
[created] <uuid>
[edited] <uuid>
[deleted] <uuid>
[moved] <uuid> -> <new-parent-uuid>
```

With `--json`, prints the full notification params as pretty-printed JSON.

Press Ctrl+C to stop.
