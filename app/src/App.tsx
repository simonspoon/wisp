import { createSignal, onMount, onCleanup, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface WispNode {
  id: string;
  name: string;
  node_type: string;
  parent_id: string | null;
  children: string[];
  layout: { x: number; y: number; width: number; height: number };
  style: { fill?: string; stroke?: string; stroke_width?: number; corner_radius?: number; opacity?: number };
  typography: { content?: string; font_family?: string; font_size?: number; font_weight?: number; line_height?: number };
}

function App() {
  const [tree, setTree] = createSignal("");
  const [nodes, setNodes] = createSignal<WispNode[]>([]);
  const [rootId, setRootId] = createSignal("");
  const [selectedNode, setSelectedNode] = createSignal<WispNode | null>(null);
  const [error, setError] = createSignal("");
  const [canvasScale, setCanvasScale] = createSignal(0.5);

  let pollInterval: number;
  let canvasAreaRef: HTMLElement | undefined;

  async function refresh() {
    try {
      const [treeText, nodeList, root] = await Promise.all([
        invoke<string>("get_tree"),
        invoke<WispNode[]>("get_nodes"),
        invoke<string>("get_root_id"),
      ]);
      setTree(treeText);
      setNodes(nodeList);
      setRootId(root);
      setError("");
      updateScale();

      // Update selected node if it still exists
      const sel = selectedNode();
      if (sel) {
        const updated = nodeList.find((n) => n.id === sel.id);
        setSelectedNode(updated || null);
      }
    } catch (e: any) {
      setError(String(e));
    }
  }

  function updateScale() {
    if (!canvasAreaRef) return;
    const root = rootNode();
    if (!root) return;
    const padding = 80;
    const availW = canvasAreaRef.clientWidth - padding;
    const availH = canvasAreaRef.clientHeight - padding;
    const scaleW = availW / root.layout.width;
    const scaleH = availH / root.layout.height;
    setCanvasScale(Math.min(scaleW, scaleH, 1));
  }

  onMount(() => {
    refresh();
    pollInterval = setInterval(refresh, 500) as unknown as number;
    // Compute scale after first render
    setTimeout(updateScale, 100);
    window.addEventListener("resize", updateScale);
  });

  onCleanup(() => {
    clearInterval(pollInterval);
    window.removeEventListener("resize", updateScale);
  });

  function getChildren(parentId: string): WispNode[] {
    const parent = nodes().find((n) => n.id === parentId);
    if (!parent) return [];
    const nodeMap = new Map(nodes().map((n) => [n.id, n]));
    return parent.children
      .map((id) => nodeMap.get(id))
      .filter((n): n is WispNode => n !== undefined);
  }

  function renderNode(node: WispNode, depth: number) {
    const isSelected = () => selectedNode()?.id === node.id;
    const children = () => getChildren(node.id);
    const fillColor = () => node.style.fill || "transparent";
    const hasContent = () => node.node_type === "text" && node.typography.content;

    return (
      <div class="tree-node" style={{ "padding-left": `${depth * 16}px` }}>
        <div
          class={`node-row ${isSelected() ? "selected" : ""}`}
          onClick={() => setSelectedNode(node)}
        >
          <span class="node-icon">{nodeIcon(node.node_type)}</span>
          <span class="node-name">{node.name}</span>
          <Show when={node.layout.width > 0}>
            <span class="node-dims">
              {Math.round(node.layout.width)}x{Math.round(node.layout.height)}
            </span>
          </Show>
          <Show when={node.style.fill}>
            <span
              class="node-swatch"
              style={{ "background-color": fillColor() }}
            />
          </Show>
        </div>
        <For each={children()}>
          {(child) => renderNode(child, depth + 1)}
        </For>
      </div>
    );
  }

  function nodeIcon(nodeType: string): string {
    switch (nodeType) {
      case "frame": return "\u25a1";
      case "text": return "T";
      case "rectangle": return "\u25ad";
      case "ellipse": return "\u25cb";
      case "group": return "\u25a3";
      default: return "\u00b7";
    }
  }

  const rootNode = () => nodes().find((n) => n.id === rootId());

  return (
    <main class="wisp-app">
      <header class="toolbar">
        <h1>Wisp</h1>
        <span class="node-count">{nodes().length} nodes</span>
        <Show when={error()}>
          <span class="error">{error()}</span>
        </Show>
      </header>

      <div class="workspace">
        {/* Left panel: tree view */}
        <aside class="panel tree-panel">
          <h2>Layers</h2>
          <div class="tree-view">
            <Show when={rootNode()} fallback={<p class="empty">Loading...</p>}>
              {(node) => renderNode(node(), 0)}
            </Show>
          </div>
        </aside>

        {/* Center: canvas preview */}
        <section class="canvas-area" ref={canvasAreaRef}>
          <div class="canvas" style={{ "--canvas-scale": canvasScale().toString() }}>
            <Show when={rootNode()}>
              {(root) => (
                <div
                  class="canvas-root"
                  style={{
                    width: `${root().layout.width}px`,
                    height: `${root().layout.height}px`,
                  }}
                >
                  <For each={getChildren(root().id)}>
                    {(child) => <CanvasNode node={child} allNodes={nodes()} />}
                  </For>
                </div>
              )}
            </Show>
          </div>
        </section>

        {/* Right panel: properties */}
        <aside class="panel props-panel">
          <h2>Properties</h2>
          <Show when={selectedNode()} fallback={<p class="empty">Select a node</p>}>
            {(node) => (
              <div class="props">
                <div class="prop-group">
                  <label>Name</label>
                  <div class="prop-value">{node().name}</div>
                </div>
                <div class="prop-group">
                  <label>Type</label>
                  <div class="prop-value">{node().node_type}</div>
                </div>
                <div class="prop-group">
                  <label>ID</label>
                  <div class="prop-value id">{node().id}</div>
                </div>
                <div class="prop-group">
                  <label>Position</label>
                  <div class="prop-value">
                    X: {node().layout.x} Y: {node().layout.y}
                  </div>
                </div>
                <div class="prop-group">
                  <label>Size</label>
                  <div class="prop-value">
                    {node().layout.width} x {node().layout.height}
                  </div>
                </div>
                <Show when={node().style.fill}>
                  <div class="prop-group">
                    <label>Fill</label>
                    <div class="prop-value">
                      <span
                        class="color-swatch"
                        style={{ "background-color": node().style.fill }}
                      />
                      {node().style.fill}
                    </div>
                  </div>
                </Show>
                <Show when={node().style.stroke}>
                  <div class="prop-group">
                    <label>Stroke</label>
                    <div class="prop-value">{node().style.stroke}</div>
                  </div>
                </Show>
                <Show when={node().typography.content}>
                  <div class="prop-group">
                    <label>Content</label>
                    <div class="prop-value">{node().typography.content}</div>
                  </div>
                </Show>
                <Show when={node().typography.font_size}>
                  <div class="prop-group">
                    <label>Font Size</label>
                    <div class="prop-value">{node().typography.font_size}px</div>
                  </div>
                </Show>
              </div>
            )}
          </Show>
        </aside>
      </div>
    </main>
  );
}

function CanvasNode(props: { node: WispNode; allNodes: WispNode[] }) {
  const children = () => {
    const nodeMap = new Map(props.allNodes.map((n) => [n.id, n]));
    return props.node.children
      .map((id) => nodeMap.get(id))
      .filter((n): n is WispNode => n !== undefined);
  };

  const style = () => {
    const n = props.node;
    const s: Record<string, string> = {
      position: "absolute",
      left: `${n.layout.x}px`,
      top: `${n.layout.y}px`,
      width: n.layout.width > 0 ? `${n.layout.width}px` : "auto",
      height: n.layout.height > 0 ? `${n.layout.height}px` : "auto",
    };

    if (n.style.fill) s["background-color"] = n.style.fill;
    if (n.style.stroke) s["border"] = `${n.style.stroke_width || 1}px solid ${n.style.stroke}`;
    if (n.style.corner_radius) s["border-radius"] = `${n.style.corner_radius}px`;
    if (n.style.opacity !== undefined && n.style.opacity !== null) s["opacity"] = String(n.style.opacity);

    return s;
  };

  const isText = () => props.node.node_type === "text";

  return (
    <div class={`canvas-node canvas-${props.node.node_type}`} style={style()} title={props.node.name}>
      <Show when={isText() && props.node.typography.content}>
        <span
          style={{
            "font-family": props.node.typography.font_family || "inherit",
            "font-size": props.node.typography.font_size
              ? `${props.node.typography.font_size}px`
              : "inherit",
            "font-weight": props.node.typography.font_weight || "inherit",
          }}
        >
          {props.node.typography.content}
        </span>
      </Show>
      <For each={children()}>
        {(child) => <CanvasNode node={child} allNodes={props.allNodes} />}
      </For>
    </div>
  );
}

export default App;
