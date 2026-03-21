use crate::store::NodeStore;
use uuid::Uuid;

/// Render the node tree as a compact indented text format.
///
/// Example output:
/// ```text
/// Document (frame, 1920x1080)
///   Header (frame, 1920x80)
///     Logo (rectangle, 120x40) fill=#1a1a2e
///     Title (text) "Wisp v0.1"
///   Canvas (frame, 1920x1000)
///     Card (rectangle, 300x200) fill=#ffffff
/// ```
pub fn render_tree(store: &NodeStore) -> String {
    let mut output = String::new();
    render_node(store, store.root_id(), 0, &mut output);
    // Remove trailing newline
    if output.ends_with('\n') {
        output.pop();
    }
    output
}

fn render_node(store: &NodeStore, id: Uuid, depth: usize, output: &mut String) {
    let node = match store.get(id) {
        Ok(n) => n,
        Err(_) => return,
    };

    let indent = "  ".repeat(depth);
    let type_str = format!("{:?}", node.node_type).to_lowercase();

    // Start with name and type
    output.push_str(&indent);
    output.push_str(&node.name);

    // Build annotation parts
    let mut parts = vec![type_str];

    // Add dimensions if non-zero
    if node.layout.width > 0.0 || node.layout.height > 0.0 {
        parts.push(format!(
            "{}x{}",
            fmt_num(node.layout.width),
            fmt_num(node.layout.height)
        ));
    }

    output.push_str(&format!(" ({})", parts.join(", ")));

    // Add style annotations inline
    if let Some(ref fill) = node.style.fill {
        output.push_str(&format!(" fill={fill}"));
    }
    if let Some(ref stroke) = node.style.stroke {
        output.push_str(&format!(" stroke={stroke}"));
    }

    // Add text content for text nodes
    if let Some(ref content) = node.typography.content {
        output.push_str(&format!(" \"{content}\""));
    }

    output.push('\n');

    // Render children in order
    for &child_id in &node.children {
        render_node(store, child_id, depth + 1, output);
    }
}

/// Format a number: show as integer if it has no fractional part.
fn fmt_num(n: f64) -> String {
    if n == n.floor() {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeType;

    #[test]
    fn render_empty_document() {
        let store = NodeStore::new();
        let tree = render_tree(&store);
        assert_eq!(tree, "Document (frame, 1920x1080)");
    }

    #[test]
    fn render_nested_tree() {
        let mut store = NodeStore::new();
        let root = store.root_id();

        let header = store.add("Header", NodeType::Frame, root).unwrap();
        store.get_mut(header).unwrap().layout.width = 1920.0;
        store.get_mut(header).unwrap().layout.height = 80.0;

        let logo = store.add("Logo", NodeType::Rectangle, header).unwrap();
        store.get_mut(logo).unwrap().layout.width = 120.0;
        store.get_mut(logo).unwrap().layout.height = 40.0;
        store.get_mut(logo).unwrap().style.fill = Some("#1a1a2e".to_string());

        let title = store.add("Title", NodeType::Text, header).unwrap();
        store.get_mut(title).unwrap().typography.content = Some("Wisp v0.1".to_string());

        let tree = render_tree(&store);
        let expected = "\
Document (frame, 1920x1080)
  Header (frame, 1920x80)
    Logo (rectangle, 120x40) fill=#1a1a2e
    Title (text) \"Wisp v0.1\"";

        assert_eq!(tree, expected);
    }

    #[test]
    fn render_with_stroke() {
        let mut store = NodeStore::new();
        let root = store.root_id();
        let rect = store.add("Box", NodeType::Rectangle, root).unwrap();
        store.get_mut(rect).unwrap().layout.width = 100.0;
        store.get_mut(rect).unwrap().layout.height = 50.0;
        store.get_mut(rect).unwrap().style.fill = Some("#fff".to_string());
        store.get_mut(rect).unwrap().style.stroke = Some("#000".to_string());

        let tree = render_tree(&store);
        assert!(tree.contains("fill=#fff stroke=#000"));
    }

    #[test]
    fn fmt_num_integer() {
        assert_eq!(fmt_num(100.0), "100");
        assert_eq!(fmt_num(0.0), "0");
    }

    #[test]
    fn fmt_num_fractional() {
        assert_eq!(fmt_num(10.5), "10.5");
    }
}
