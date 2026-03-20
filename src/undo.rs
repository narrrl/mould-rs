use crate::format::ConfigItem;
use std::collections::HashMap;

/// Represents a single snapshot of the application state for undo/redo purposes.
pub struct EditAction {
    /// The complete list of configuration items at the time of the action.
    pub state: Vec<ConfigItem>,
    /// The index of the item that was selected during this action.
    pub selected: usize,
}

/// A node in the undo tree, representing a point in the application's history.
pub struct UndoNode {
    /// The state data captured at this history point.
    pub action: EditAction,
    /// ID of the parent node (previous state). Root node has None.
    pub parent: Option<usize>,
    /// IDs of all states that branched off from this one.
    pub children: Vec<usize>,
}

/// A non-linear undo/redo system that tracks history as a branching tree.
///
/// Unlike a simple stack, an UndoTree allows users to undo several steps, 
/// make a new change (creating a branch), and still navigate through 
/// the most recent history path.
pub struct UndoTree {
    /// Map of node IDs to their respective history nodes.
    nodes: HashMap<usize, UndoNode>,
    /// The ID of the node representing the current application state.
    current_node: usize,
    /// Counter for assigning unique IDs to new nodes.
    next_id: usize,
    /// Tracks the most recently active branch for each node.
    /// This allows the 'redo' operation to follow the path the user 
    /// actually took when multiple branches exist.
    latest_branch: HashMap<usize, usize>,
}

impl UndoTree {
    /// Creates a new undo tree initialized with the starting application state.
    pub fn new(initial_state: Vec<ConfigItem>, initial_selected: usize) -> Self {
        let root_id = 0;
        let root_node = UndoNode {
            action: EditAction {
                state: initial_state,
                selected: initial_selected,
            },
            parent: None,
            children: Vec::new(),
        };

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            current_node: root_id,
            next_id: 1,
            latest_branch: HashMap::new(),
        }
    }

    /// Pushes a new state onto the tree, branching off from the current node.
    ///
    /// This creates a new child node for the current position and updates
    /// the branch tracking to ensure this new path is preferred during redo.
    pub fn push(&mut self, state: Vec<ConfigItem>, selected: usize) {
        let new_id = self.next_id;
        self.next_id += 1;

        let new_node = UndoNode {
            action: EditAction { state, selected },
            parent: Some(self.current_node),
            children: Vec::new(),
        };

        // Add to nodes
        self.nodes.insert(new_id, new_node);

        // Update parent's children
        if let Some(parent_node) = self.nodes.get_mut(&self.current_node) {
            parent_node.children.push(new_id);
        }

        // Record this as the latest branch for the parent
        self.latest_branch.insert(self.current_node, new_id);

        // Move current pointer
        self.current_node = new_id;
    }

    /// Moves the current pointer back to the parent node and returns the previous state.
    /// Returns None if the current node is the root (no more history to undo).
    pub fn undo(&mut self) -> Option<&EditAction> {
        if let Some(current) = self.nodes.get(&self.current_node)
            && let Some(parent_id) = current.parent {
                self.current_node = parent_id;
                return self.nodes.get(&parent_id).map(|n| &n.action);
            }
        None
    }

    /// Moves the current pointer forward to the latest child branch and returns the state.
    ///
    /// Redo follows the `latest_branch` map to decide which path to take 
    /// if multiple branches exist. If no branch is recorded, it defaults to the 
    /// most recently created child.
    pub fn redo(&mut self) -> Option<&EditAction> {
        if let Some(next_id) = self.latest_branch.get(&self.current_node).copied() {
            self.current_node = next_id;
            return self.nodes.get(&next_id).map(|n| &n.action);
        } else {
            // Fallback: if there is no recorded latest branch but there are children
            let current_id = self.current_node;
            if let Some(current) = self.nodes.get(&current_id)
                && let Some(&first_child_id) = current.children.last() {
                    self.current_node = first_child_id;
                    self.latest_branch.insert(current_id, first_child_id);
                    return self.nodes.get(&first_child_id).map(|n| &n.action);
                }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{ItemStatus, ValueType};

    fn dummy_item(key: &str) -> ConfigItem {
        ConfigItem {
            key: key.to_string(),
            path: vec![],
            value: Some(key.to_string()),
            template_value: None,
            default_value: None,
            depth: 0,
            is_group: false,
            status: ItemStatus::Present,
            value_type: ValueType::String,
        }
    }

    #[test]
    fn test_undo_redo_tree() {
        let state1 = vec![dummy_item("A")];
        let mut tree = UndoTree::new(state1.clone(), 0);

        // Push state 2
        let state2 = vec![dummy_item("B")];
        tree.push(state2.clone(), 1);

        // Push state 3
        let state3 = vec![dummy_item("C")];
        tree.push(state3.clone(), 2);

        // Undo -> State 2
        let action = tree.undo().unwrap();
        assert_eq!(action.state[0].key, "B");
        assert_eq!(action.selected, 1);

        // Undo -> State 1
        let action = tree.undo().unwrap();
        assert_eq!(action.state[0].key, "A");
        assert_eq!(action.selected, 0);

        // Undo again -> None (already at root)
        assert!(tree.undo().is_none());

        // Redo -> State 2
        let action = tree.redo().unwrap();
        assert_eq!(action.state[0].key, "B");
        assert_eq!(action.selected, 1);

        // Redo -> State 3
        let action = tree.redo().unwrap();
        assert_eq!(action.state[0].key, "C");
        assert_eq!(action.selected, 2);

        // Branching: Undo twice to State 1
        tree.undo();
        tree.undo();
        
        // Push State 4 (from State 1)
        let state4 = vec![dummy_item("D")];
        tree.push(state4.clone(), 3);

        // Undo -> State 1
        let action = tree.undo().unwrap();
        assert_eq!(action.state[0].key, "A");

        // Redo -> State 4 (follows latest branch D, not old branch B)
        let action = tree.redo().unwrap();
        assert_eq!(action.state[0].key, "D");
    }

    #[test]
    fn test_redo_fallback_fix() {
        let state1 = vec![dummy_item("A")];
        let mut tree = UndoTree::new(state1.clone(), 0);

        let state2 = vec![dummy_item("B")];
        tree.push(state2.clone(), 1);

        tree.undo();
        // Redo should move to state 2
        let action = tree.redo().unwrap();
        assert_eq!(action.state[0].key, "B");

        // Calling redo again should NOT change the current node or returned action
        // (since it's already at the latest child)
        assert!(tree.redo().is_none());
    }
}
