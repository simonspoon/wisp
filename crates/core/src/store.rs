use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{Node, NodeType};

/// Errors from store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("node not found: {0}")]
    NotFound(Uuid),
    #[error("cannot move node under itself or its descendant")]
    CyclicMove,
    #[error("cannot delete root node")]
    DeleteRoot,
}

/// Flat HashMap-based document store with parent-child refs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStore {
    nodes: HashMap<Uuid, Node>,
    root_id: Uuid,
}

impl NodeStore {
    /// Create a new store with a root frame node.
    pub fn new() -> Self {
        let mut root = Node::new("Document", NodeType::Frame);
        let root_id = root.id;
        root.layout.width = 1920.0;
        root.layout.height = 1080.0;
        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);
        Self { nodes, root_id }
    }

    pub fn root_id(&self) -> Uuid {
        self.root_id
    }

    pub fn get(&self, id: Uuid) -> Result<&Node, StoreError> {
        self.nodes.get(&id).ok_or(StoreError::NotFound(id))
    }

    pub fn get_mut(&mut self, id: Uuid) -> Result<&mut Node, StoreError> {
        self.nodes.get_mut(&id).ok_or(StoreError::NotFound(id))
    }

    /// All nodes as a slice-like iterator.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Add a new node as a child of `parent_id`. Returns the new node's ID.
    pub fn add(
        &mut self,
        name: impl Into<String>,
        node_type: NodeType,
        parent_id: Uuid,
    ) -> Result<Uuid, StoreError> {
        // Verify parent exists
        if !self.nodes.contains_key(&parent_id) {
            return Err(StoreError::NotFound(parent_id));
        }

        let mut node = Node::new(name, node_type);
        node.parent_id = Some(parent_id);
        let id = node.id;

        self.nodes.insert(id, node);
        self.nodes.get_mut(&parent_id).unwrap().children.push(id);

        Ok(id)
    }

    /// Add a node with a pre-assigned ID.
    pub fn add_with_id(
        &mut self,
        id: Uuid,
        name: impl Into<String>,
        node_type: NodeType,
        parent_id: Uuid,
    ) -> Result<(), StoreError> {
        if !self.nodes.contains_key(&parent_id) {
            return Err(StoreError::NotFound(parent_id));
        }

        let mut node = Node::with_id(id, name, node_type);
        node.parent_id = Some(parent_id);

        self.nodes.insert(id, node);
        self.nodes.get_mut(&parent_id).unwrap().children.push(id);

        Ok(())
    }

    /// Delete a node and all its descendants. Cannot delete root.
    pub fn delete(&mut self, id: Uuid) -> Result<(), StoreError> {
        if id == self.root_id {
            return Err(StoreError::DeleteRoot);
        }

        let node = self.nodes.get(&id).ok_or(StoreError::NotFound(id))?;
        let parent_id = node.parent_id;

        // Collect all descendants
        let mut to_remove = vec![id];
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for &child_id in &node.children {
                    to_remove.push(child_id);
                    stack.push(child_id);
                }
            }
        }

        // Remove from parent's children list
        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.children.retain(|c| *c != id);
            }
        }

        // Remove all collected nodes
        for rid in to_remove {
            self.nodes.remove(&rid);
        }

        Ok(())
    }

    /// Move a node to a new parent. Prevents cycles.
    pub fn move_node(&mut self, id: Uuid, new_parent_id: Uuid) -> Result<(), StoreError> {
        if id == self.root_id {
            return Err(StoreError::CyclicMove);
        }

        // Check both nodes exist
        if !self.nodes.contains_key(&id) {
            return Err(StoreError::NotFound(id));
        }
        if !self.nodes.contains_key(&new_parent_id) {
            return Err(StoreError::NotFound(new_parent_id));
        }

        // Prevent moving under self or descendant
        if self.is_descendant_of(new_parent_id, id) {
            return Err(StoreError::CyclicMove);
        }

        // Remove from old parent
        let old_parent_id = self.nodes.get(&id).unwrap().parent_id;
        if let Some(pid) = old_parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.children.retain(|c| *c != id);
            }
        }

        // Add to new parent
        self.nodes
            .get_mut(&new_parent_id)
            .unwrap()
            .children
            .push(id);
        self.nodes.get_mut(&id).unwrap().parent_id = Some(new_parent_id);

        Ok(())
    }

    /// Check if `candidate` is a descendant of `ancestor`.
    fn is_descendant_of(&self, candidate: Uuid, ancestor: Uuid) -> bool {
        if candidate == ancestor {
            return true;
        }
        let mut current = candidate;
        while let Some(node) = self.nodes.get(&current) {
            match node.parent_id {
                Some(pid) if pid == ancestor => return true,
                Some(pid) => current = pid,
                None => return false,
            }
        }
        false
    }

    /// Get ordered children of a node.
    pub fn children(&self, id: Uuid) -> Result<Vec<&Node>, StoreError> {
        let node = self.get(id)?;
        let children: Vec<&Node> = node
            .children
            .iter()
            .filter_map(|cid| self.nodes.get(cid))
            .collect();
        Ok(children)
    }

    /// Number of nodes in the store.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeType;

    #[test]
    fn new_store_has_root() {
        let store = NodeStore::new();
        assert_eq!(store.len(), 1);
        let root = store.get(store.root_id()).unwrap();
        assert_eq!(root.name, "Document");
        assert_eq!(root.node_type, NodeType::Frame);
        assert!(root.parent_id.is_none());
    }

    #[test]
    fn add_child_node() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let id = store.add("Header", NodeType::Frame, root_id).unwrap();

        assert_eq!(store.len(), 2);
        let node = store.get(id).unwrap();
        assert_eq!(node.name, "Header");
        assert_eq!(node.parent_id, Some(root_id));

        let root = store.get(root_id).unwrap();
        assert!(root.children.contains(&id));
    }

    #[test]
    fn add_to_nonexistent_parent_fails() {
        let mut store = NodeStore::new();
        let fake_id = Uuid::new_v4();
        let result = store.add("Orphan", NodeType::Frame, fake_id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_node_removes_descendants() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let parent_id = store.add("Parent", NodeType::Frame, root_id).unwrap();
        let _child_id = store.add("Child", NodeType::Rectangle, parent_id).unwrap();
        let _grandchild_id = store
            .add("Grandchild", NodeType::Ellipse, parent_id)
            .unwrap();

        assert_eq!(store.len(), 4);
        store.delete(parent_id).unwrap();
        assert_eq!(store.len(), 1); // only root remains
    }

    #[test]
    fn delete_root_fails() {
        let mut store = NodeStore::new();
        let result = store.delete(store.root_id());
        assert!(result.is_err());
    }

    #[test]
    fn move_node_to_new_parent() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let a = store.add("A", NodeType::Frame, root_id).unwrap();
        let b = store.add("B", NodeType::Frame, root_id).unwrap();
        let c = store.add("C", NodeType::Rectangle, a).unwrap();

        store.move_node(c, b).unwrap();

        let node_c = store.get(c).unwrap();
        assert_eq!(node_c.parent_id, Some(b));
        assert!(!store.get(a).unwrap().children.contains(&c));
        assert!(store.get(b).unwrap().children.contains(&c));
    }

    #[test]
    fn move_node_prevents_cycle() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let a = store.add("A", NodeType::Frame, root_id).unwrap();
        let b = store.add("B", NodeType::Frame, a).unwrap();

        // Moving A under B would create a cycle
        let result = store.move_node(a, b);
        assert!(result.is_err());
    }

    #[test]
    fn edit_node_properties() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let id = store.add("Box", NodeType::Rectangle, root_id).unwrap();

        let node = store.get_mut(id).unwrap();
        node.name = "Red Box".to_string();
        node.style.fill = Some("#ff0000".to_string());
        node.layout.width = 200.0;
        node.layout.height = 100.0;

        let node = store.get(id).unwrap();
        assert_eq!(node.name, "Red Box");
        assert_eq!(node.style.fill.as_deref(), Some("#ff0000"));
        assert_eq!(node.layout.width, 200.0);
    }

    #[test]
    fn children_returns_ordered_list() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        let a = store.add("A", NodeType::Frame, root_id).unwrap();
        let b = store.add("B", NodeType::Frame, root_id).unwrap();
        let c = store.add("C", NodeType::Frame, root_id).unwrap();

        let children = store.children(root_id).unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].id, a);
        assert_eq!(children[1].id, b);
        assert_eq!(children[2].id, c);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut store = NodeStore::new();
        let root_id = store.root_id();
        store.add("Header", NodeType::Frame, root_id).unwrap();

        let json = serde_json::to_string(&store).unwrap();
        let restored: NodeStore = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.root_id(), store.root_id());
    }
}
