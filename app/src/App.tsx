import { createSignal, onMount, onCleanup, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toPng } from "html-to-image";
import "./App.css";

interface WispNode {
  id: string;
  name: string;
  node_type: string;
  parent_id: string | null;
  children: string[];
  layout: { x: number; y: number; width: number; height: number };
  style: { fill?: string; stroke?: string; stroke_width?: number; corner_radius?: number; opacity?: number; z_index?: number; clip?: boolean };
  typography: { content?: string; font_family?: string; font_size?: number; font_weight?: number; line_height?: number; text_auto_size?: boolean; color?: string; text_align?: string };
  auto_layout: {
    mode: string;
    direction: string;
    align_items: string;
    justify_content?: string;
    gap?: number;
    padding?: number;
    padding_horizontal?: number;
    padding_vertical?: number;
  };
}

interface DragState {
  id: string;
  startMouseX: number;
  startMouseY: number;
  origX: number;
  origY: number;
}

interface ResizeState {
  id: string;
  startMouseX: number;
  startMouseY: number;
  origW: number;
  origH: number;
}

function App() {
  const [tree, setTree] = createSignal("");
  const [nodes, setNodes] = createSignal<WispNode[]>([]);
  const [rootId, setRootId] = createSignal("");
  const [selectedNode, setSelectedNode] = createSignal<WispNode | null>(null);
  const [error, setError] = createSignal("");
  const [canvasScale, setCanvasScale] = createSignal(0.5);
  const [dragging, setDragging] = createSignal<DragState | null>(null);
  const [resizing, setResizing] = createSignal<ResizeState | null>(null);

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

  function handleCanvasMouseMove(e: MouseEvent) {
    const scale = canvasScale();
    const drag = dragging();
    if (drag) {
      const dx = (e.clientX - drag.startMouseX) / scale;
      const dy = (e.clientY - drag.startMouseY) / scale;
      const newX = Math.round(drag.origX + dx);
      const newY = Math.round(drag.origY + dy);
      // Optimistic local update
      setNodes((prev) =>
        prev.map((n) =>
          n.id === drag.id ? { ...n, layout: { ...n.layout, x: newX, y: newY } } : n
        )
      );
      return;
    }
    const resize = resizing();
    if (resize) {
      const dx = (e.clientX - resize.startMouseX) / scale;
      const dy = (e.clientY - resize.startMouseY) / scale;
      const newW = Math.max(1, Math.round(resize.origW + dx));
      const newH = Math.max(1, Math.round(resize.origH + dy));
      setNodes((prev) =>
        prev.map((n) =>
          n.id === resize.id ? { ...n, layout: { ...n.layout, width: newW, height: newH } } : n
        )
      );
      return;
    }
  }

  async function handleCanvasMouseUp() {
    const drag = dragging();
    if (drag) {
      const node = nodes().find((n) => n.id === drag.id);
      if (node) {
        await invoke("edit_node", { id: drag.id, x: node.layout.x, y: node.layout.y });
      }
      setDragging(null);
      refresh();
      return;
    }
    const resize = resizing();
    if (resize) {
      const node = nodes().find((n) => n.id === resize.id);
      if (node) {
        await invoke("edit_node", { id: resize.id, width: node.layout.width, height: node.layout.height });
      }
      setResizing(null);
      refresh();
      return;
    }
  }

  function startDrag(nodeId: string, e: MouseEvent) {
    e.stopPropagation();
    const node = nodes().find((n) => n.id === nodeId);
    if (!node) return;
    setSelectedNode(node);
    setDragging({
      id: nodeId,
      startMouseX: e.clientX,
      startMouseY: e.clientY,
      origX: node.layout.x,
      origY: node.layout.y,
    });
  }

  function startResize(nodeId: string, e: MouseEvent) {
    e.stopPropagation();
    e.preventDefault();
    const node = nodes().find((n) => n.id === nodeId);
    if (!node) return;
    setResizing({
      id: nodeId,
      startMouseX: e.clientX,
      startMouseY: e.clientY,
      origW: node.layout.width,
      origH: node.layout.height,
    });
  }

  let unlistenScreenshot: (() => void) | undefined;

  onMount(async () => {
    refresh();
    pollInterval = setInterval(refresh, 500) as unknown as number;
    setTimeout(updateScale, 100);
    window.addEventListener("resize", updateScale);

    // Listen for screenshot requests from the server
    unlistenScreenshot = await listen<string>("screenshot-request", async (event) => {
      const requestId = event.payload;
      const canvasRoot = document.querySelector(".canvas-root") as HTMLElement;
      if (!canvasRoot) {
        await invoke("deliver_screenshot", {
          requestId,
          pngBase64: "",
        });
        return;
      }
      try {
        const dataUrl = await toPng(canvasRoot, {
          pixelRatio: 2,
          backgroundColor: "#ffffff",
        });
        const base64 = dataUrl.replace(/^data:image\/png;base64,/, "");
        await invoke("deliver_screenshot", { requestId, pngBase64: base64 });
      } catch (e) {
        console.error("Screenshot capture failed:", e);
        await invoke("deliver_screenshot", { requestId, pngBase64: "" });
      }
    });
  });

  onCleanup(() => {
    clearInterval(pollInterval);
    window.removeEventListener("resize", updateScale);
    unlistenScreenshot?.();
  });

  function getChildren(parentId: string): WispNode[] {
    const parent = nodes().find((n) => n.id === parentId);
    if (!parent) return [];
    const nodeMap = new Map(nodes().map((n) => [n.id, n]));
    return parent.children
      .map((id) => nodeMap.get(id))
      .filter((n): n is WispNode => n !== undefined)
      .sort((a, b) => (a.style.z_index ?? 0) - (b.style.z_index ?? 0));
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
        <section
          class="canvas-area"
          ref={canvasAreaRef}
          onMouseMove={handleCanvasMouseMove}
          onMouseUp={handleCanvasMouseUp}
          onMouseLeave={handleCanvasMouseUp}
        >
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
                    {(child) => (
                      <CanvasNode
                        node={child}
                        allNodes={nodes()}
                        selectedId={selectedNode()?.id ?? null}
                        onDragStart={startDrag}
                        onResizeStart={startResize}
                        onSelect={(n) => setSelectedNode(n)}
                      />
                    )}
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

interface CanvasNodeProps {
  node: WispNode;
  allNodes: WispNode[];
  selectedId: string | null;
  parentLayoutMode?: string;
  onDragStart: (id: string, e: MouseEvent) => void;
  onResizeStart: (id: string, e: MouseEvent) => void;
  onSelect: (node: WispNode) => void;
}

function CanvasNode(props: CanvasNodeProps) {
  const children = () => {
    const nodeMap = new Map(props.allNodes.map((n) => [n.id, n]));
    return props.node.children
      .map((id) => nodeMap.get(id))
      .filter((n): n is WispNode => n !== undefined)
      .sort((a, b) => (a.style.z_index ?? 0) - (b.style.z_index ?? 0));
  };

  const isSelected = () => props.selectedId === props.node.id;

  const isFlex = () => props.node.auto_layout?.mode === "flex";
  const isFlexChild = () => props.parentLayoutMode === "flex";

  const mapAlign = (a: string | undefined): string => {
    switch (a) {
      case "start": return "flex-start";
      case "center": return "center";
      case "end": return "flex-end";
      case "stretch": return "stretch";
      case "space_between": return "space-between";
      default: return "flex-start";
    }
  };

  const style = () => {
    const n = props.node;
    const s: Record<string, string> = {};

    // Positioning: flex children don't use absolute positioning
    if (isFlexChild()) {
      s["position"] = "relative";
      s["width"] = n.layout.width > 0 ? `${n.layout.width}px` : "auto";
      s["height"] = n.layout.height > 0 ? `${n.layout.height}px` : "auto";
    } else {
      s["position"] = "absolute";
      s["left"] = `${n.layout.x}px`;
      s["top"] = `${n.layout.y}px`;
      s["width"] = n.layout.width > 0 ? `${n.layout.width}px` : "auto";
      s["height"] = n.layout.height > 0 ? `${n.layout.height}px` : "auto";
    }

    // Flex container properties
    if (isFlex()) {
      s["display"] = "flex";
      s["flex-direction"] = n.auto_layout.direction === "row" ? "row" : "column";
      s["align-items"] = mapAlign(n.auto_layout.align_items);
      if (n.auto_layout.justify_content) {
        s["justify-content"] = mapAlign(n.auto_layout.justify_content);
      }
      if (n.auto_layout.gap !== undefined) {
        s["gap"] = `${n.auto_layout.gap}px`;
      }
      const ph = n.auto_layout.padding_horizontal ?? n.auto_layout.padding ?? 0;
      const pv = n.auto_layout.padding_vertical ?? n.auto_layout.padding ?? 0;
      if (ph > 0 || pv > 0) {
        s["padding"] = `${pv}px ${ph}px`;
      }
    }

    if (n.style.fill) s["background-color"] = n.style.fill;
    if (n.style.stroke) s["border"] = `${n.style.stroke_width || 1}px solid ${n.style.stroke}`;
    if (n.style.corner_radius) s["border-radius"] = `${n.style.corner_radius}px`;
    if (n.style.opacity !== undefined && n.style.opacity !== null) s["opacity"] = String(n.style.opacity);
    if (n.style.z_index !== undefined && n.style.z_index !== null) s["z-index"] = String(n.style.z_index);
    if (n.style.clip) s["overflow"] = "hidden";

    // Text color
    if (n.typography.color) s["color"] = n.typography.color;

    // Text alignment
    if (n.typography.text_align) s["text-align"] = n.typography.text_align;

    // Text wrapping: auto-size height when enabled
    if (n.node_type === "text" && n.typography.text_auto_size) {
      s["height"] = "auto";
      s["word-wrap"] = "break-word";
      s["overflow-wrap"] = "break-word";
      s["white-space"] = "normal";
    }

    return s;
  };

  const isText = () => props.node.node_type === "text";

  return (
    <div
      class={`canvas-node canvas-${props.node.node_type} ${isSelected() ? "canvas-selected" : ""}`}
      style={style()}
      title={props.node.name}
      onMouseDown={(e: MouseEvent) => props.onDragStart(props.node.id, e)}
    >
      <Show when={isText() && props.node.typography.content}>
        <span
          style={{
            "font-family": props.node.typography.font_family || "inherit",
            "font-size": props.node.typography.font_size
              ? `${props.node.typography.font_size}px`
              : "inherit",
            "font-weight": props.node.typography.font_weight || "inherit",
            "line-height": props.node.typography.line_height
              ? String(props.node.typography.line_height)
              : "inherit",
            "width": "100%",
          }}
        >
          {props.node.typography.content}
        </span>
      </Show>
      <For each={children()}>
        {(child) => (
          <CanvasNode
            node={child}
            allNodes={props.allNodes}
            selectedId={props.selectedId}
            parentLayoutMode={props.node.auto_layout?.mode}
            onDragStart={props.onDragStart}
            onResizeStart={props.onResizeStart}
            onSelect={props.onSelect}
          />
        )}
      </For>
      <Show when={isSelected()}>
        <div
          class="resize-handle"
          onMouseDown={(e: MouseEvent) => props.onResizeStart(props.node.id, e)}
        />
      </Show>
    </div>
  );
}

export default App;
