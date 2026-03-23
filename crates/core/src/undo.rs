use crate::store::NodeStore;

/// Snapshot-based undo/redo stack.
///
/// Each mutation saves a clone of the entire NodeStore before the change.
/// This is simple and correct for document sizes typical in design tools.
#[derive(Debug, Clone)]
pub struct UndoStack {
    /// Past states (newest last). The top is the state before the current.
    undo_stack: Vec<NodeStore>,
    /// Future states for redo (newest last).
    redo_stack: Vec<NodeStore>,
    /// Maximum number of undo levels.
    max_depth: usize,
}

impl UndoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Push the current state onto the undo stack before a mutation.
    /// Clears the redo stack (branching invalidates future).
    pub fn push(&mut self, state: &NodeStore) {
        self.undo_stack.push(state.clone());
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo: pop the last saved state and push the current state onto redo.
    /// Returns the state to restore, or None if nothing to undo.
    pub fn undo(&mut self, current: &NodeStore) -> Option<NodeStore> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        Some(prev)
    }

    /// Redo: pop the last redo state and push the current state onto undo.
    /// Returns the state to restore, or None if nothing to redo.
    pub fn redo(&mut self, current: &NodeStore) -> Option<NodeStore> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        Some(next)
    }

    /// Number of undo levels available.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of redo levels available.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeType;

    #[test]
    fn undo_restores_previous_state() {
        let mut stack = UndoStack::new(10);
        let mut store = NodeStore::new();
        let root_id = store.root_id();

        // Save state before adding
        stack.push(&store);
        store.add("A", NodeType::Frame, root_id).unwrap();
        assert_eq!(store.len(), 2);

        // Undo
        let restored = stack.undo(&store).unwrap();
        assert_eq!(restored.len(), 1);
    }

    #[test]
    fn redo_restores_undone_state() {
        let mut stack = UndoStack::new(10);
        let mut store = NodeStore::new();
        let root_id = store.root_id();

        stack.push(&store);
        store.add("A", NodeType::Frame, root_id).unwrap();

        // Undo
        let prev = stack.undo(&store).unwrap();
        assert_eq!(prev.len(), 1);

        // Redo
        let redone = stack.redo(&prev).unwrap();
        assert_eq!(redone.len(), 2);
    }

    #[test]
    fn new_mutation_clears_redo() {
        let mut stack = UndoStack::new(10);
        let mut store = NodeStore::new();
        let root_id = store.root_id();

        stack.push(&store);
        store.add("A", NodeType::Frame, root_id).unwrap();

        // Undo
        store = stack.undo(&store).unwrap();

        // New mutation — redo should be cleared
        stack.push(&store);
        store.add("B", NodeType::Frame, root_id).unwrap();

        assert_eq!(stack.redo_count(), 0);
    }

    #[test]
    fn max_depth_evicts_oldest() {
        let mut stack = UndoStack::new(3);
        let mut store = NodeStore::new();
        let root_id = store.root_id();

        for i in 0..5 {
            stack.push(&store);
            store
                .add(format!("Node{i}"), NodeType::Frame, root_id)
                .unwrap();
        }

        assert_eq!(stack.undo_count(), 3);
    }

    #[test]
    fn undo_on_empty_returns_none() {
        let mut stack = UndoStack::new(10);
        let store = NodeStore::new();
        assert!(stack.undo(&store).is_none());
    }

    #[test]
    fn redo_on_empty_returns_none() {
        let mut stack = UndoStack::new(10);
        let store = NodeStore::new();
        assert!(stack.redo(&store).is_none());
    }

    #[test]
    fn multiple_undo_redo_cycles() {
        let mut stack = UndoStack::new(10);
        let mut store = NodeStore::new();
        let root_id = store.root_id();

        // Add A
        stack.push(&store);
        store.add("A", NodeType::Frame, root_id).unwrap();
        assert_eq!(store.len(), 2);

        // Add B
        stack.push(&store);
        store.add("B", NodeType::Frame, root_id).unwrap();
        assert_eq!(store.len(), 3);

        // Undo B
        store = stack.undo(&store).unwrap();
        assert_eq!(store.len(), 2);

        // Undo A
        store = stack.undo(&store).unwrap();
        assert_eq!(store.len(), 1);

        // Redo A
        store = stack.redo(&store).unwrap();
        assert_eq!(store.len(), 2);

        // Redo B
        store = stack.redo(&store).unwrap();
        assert_eq!(store.len(), 3);
    }
}
