//! UI tree manager.
//!
//! [`UiManager`] owns the UI node tree, handling allocation, hierarchy
//! management, and cycle detection. It is independent of the ECS world.

use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};

use super::allocator::UiNodeAllocator;
use super::component::UiComponent;
use super::node::UiNode;
use super::node_id::UiNodeId;

// =============================================================================
// UiManager
// =============================================================================

/// Owns and manages the UI node tree.
///
/// Nodes are allocated via a generational arena and stored in a flat
/// `HashMap`. Parent/child relationships are maintained on the nodes
/// themselves; the manager enforces tree invariants (no cycles, proper
/// bookkeeping on removal).
#[derive(Debug)]
pub struct UiManager {
    /// Generational ID allocator.
    allocator: UiNodeAllocator,

    /// All live nodes, keyed by their ID.
    nodes: HashMap<UiNodeId, UiNode>,
}

impl UiManager {
    /// Creates an empty UI manager.
    pub fn new() -> Self {
        Self {
            allocator: UiNodeAllocator::new(),
            nodes: HashMap::new(),
        }
    }

    /// Creates a new node, optionally attaching a component.
    ///
    /// The node starts with no parent (root) and no children.
    pub fn create_node(&mut self, component: Option<UiComponent>) -> UiNodeId {
        let id = self.allocator.allocate();
        let mut node = UiNode::new(id);
        node.set_component(component);
        self.nodes.insert(id, node);
        id
    }

    /// Removes a node and its entire subtree recursively.
    ///
    /// Also detaches the node from its parent's children list.
    /// Returns `true` if the node existed and was removed.
    pub fn remove_node(&mut self, id: UiNodeId) -> bool {
        // Collect the subtree IDs via iterative traversal to avoid
        // borrow-checker issues with recursive mutable access.
        let subtree = self.collect_subtree(id);
        if subtree.is_empty() {
            return false;
        }

        // Detach the root of the subtree from its parent.
        if let Some(parent_id) = self.nodes.get(&id).and_then(|n| n.parent()) {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.remove_child(id);
            }
        }

        // Remove every node in the subtree.
        for node_id in &subtree {
            self.nodes.remove(node_id);
            self.allocator.deallocate(*node_id);
        }

        true
    }

    /// Sets the parent of `child` to `parent`.
    ///
    /// Passing `None` makes the child a root node. Detects cycles by walking
    /// the ancestor chain of the proposed parent.
    ///
    /// # Errors
    ///
    /// Returns `GoudError::InvalidState` if:
    /// - `child` does not exist
    /// - `parent` is `Some` but does not exist
    /// - Setting the parent would create a cycle
    pub fn set_parent(&mut self, child: UiNodeId, parent: Option<UiNodeId>) -> GoudResult<()> {
        // Validate child exists.
        if !self.nodes.contains_key(&child) {
            return Err(GoudError::InvalidState(
                "Child node does not exist".to_string(),
            ));
        }

        // Validate parent exists (if provided).
        if let Some(pid) = parent {
            if !self.nodes.contains_key(&pid) {
                return Err(GoudError::InvalidState(
                    "Parent node does not exist".to_string(),
                ));
            }

            // Cycle detection: walk ancestors of the proposed parent.
            // If we encounter `child`, attaching would create a cycle.
            if self.is_ancestor(child, pid) {
                return Err(GoudError::InvalidState(
                    "Setting this parent would create a cycle".to_string(),
                ));
            }
        }

        // Detach from old parent.
        let old_parent = self.nodes.get(&child).and_then(|n| n.parent());
        if let Some(old_pid) = old_parent {
            if let Some(old_p) = self.nodes.get_mut(&old_pid) {
                old_p.remove_child(child);
            }
        }

        // Attach to new parent.
        if let Some(pid) = parent {
            if let Some(new_p) = self.nodes.get_mut(&pid) {
                new_p.add_child(child);
            }
        }

        // Update child's parent field.
        if let Some(node) = self.nodes.get_mut(&child) {
            node.set_parent(parent);
        }

        Ok(())
    }

    /// Returns a reference to a node, if it exists.
    #[inline]
    pub fn get_node(&self, id: UiNodeId) -> Option<&UiNode> {
        self.nodes.get(&id)
    }

    /// Returns a mutable reference to a node, if it exists.
    #[inline]
    pub fn get_node_mut(&mut self, id: UiNodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(&id)
    }

    /// Returns the number of live nodes.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns IDs of all root nodes (nodes with no parent).
    pub fn root_nodes(&self) -> Vec<UiNodeId> {
        self.nodes
            .values()
            .filter(|n| n.parent().is_none())
            .map(|n| n.id())
            .collect()
    }

    /// Placeholder for future layout computation.
    pub fn update(&mut self) {
        // Will compute layout in a future iteration.
    }

    /// Placeholder for future UI rendering.
    pub fn render(&self) {
        // Will render UI in a future iteration.
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /// Returns `true` if `ancestor` is an ancestor of `node` (or is `node` itself).
    fn is_ancestor(&self, ancestor: UiNodeId, node: UiNodeId) -> bool {
        let mut current = Some(node);
        while let Some(cur) = current {
            if cur == ancestor {
                return true;
            }
            current = self.nodes.get(&cur).and_then(|n| n.parent());
        }
        false
    }

    /// Collects all node IDs in the subtree rooted at `root` (inclusive).
    /// Returns an empty vec if `root` does not exist.
    fn collect_subtree(&self, root: UiNodeId) -> Vec<UiNodeId> {
        let mut result = Vec::new();
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            if let Some(node) = self.nodes.get(&id) {
                result.push(id);
                for &child in node.children() {
                    stack.push(child);
                }
            }
        }

        result
    }
}

impl Default for UiManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let mut mgr = UiManager::new();
        let id = mgr.create_node(None);
        assert_eq!(mgr.node_count(), 1);
        assert!(mgr.get_node(id).is_some());
    }

    #[test]
    fn test_create_node_with_component() {
        let mut mgr = UiManager::new();
        let id = mgr.create_node(Some(UiComponent::Panel));
        let node = mgr.get_node(id).unwrap();
        assert!(matches!(node.component(), Some(UiComponent::Panel)));
    }

    #[test]
    fn test_remove_node() {
        let mut mgr = UiManager::new();
        let id = mgr.create_node(None);
        assert!(mgr.remove_node(id));
        assert_eq!(mgr.node_count(), 0);
        assert!(mgr.get_node(id).is_none());
    }

    #[test]
    fn test_remove_nonexistent_returns_false() {
        let mut mgr = UiManager::new();
        assert!(!mgr.remove_node(UiNodeId::INVALID));
    }

    #[test]
    fn test_recursive_removal() {
        let mut mgr = UiManager::new();
        let root = mgr.create_node(None);
        let child = mgr.create_node(None);
        let grandchild = mgr.create_node(None);

        mgr.set_parent(child, Some(root)).unwrap();
        mgr.set_parent(grandchild, Some(child)).unwrap();

        assert!(mgr.remove_node(root));
        assert_eq!(mgr.node_count(), 0);
    }

    #[test]
    fn test_remove_subtree_preserves_siblings() {
        let mut mgr = UiManager::new();
        let root = mgr.create_node(None);
        let child_a = mgr.create_node(None);
        let child_b = mgr.create_node(None);

        mgr.set_parent(child_a, Some(root)).unwrap();
        mgr.set_parent(child_b, Some(root)).unwrap();

        assert!(mgr.remove_node(child_a));
        assert_eq!(mgr.node_count(), 2);
        assert!(mgr.get_node(root).is_some());
        assert!(mgr.get_node(child_b).is_some());
    }

    #[test]
    fn test_set_parent() {
        let mut mgr = UiManager::new();
        let parent = mgr.create_node(None);
        let child = mgr.create_node(None);

        mgr.set_parent(child, Some(parent)).unwrap();

        let child_node = mgr.get_node(child).unwrap();
        assert_eq!(child_node.parent(), Some(parent));

        let parent_node = mgr.get_node(parent).unwrap();
        assert!(parent_node.children().contains(&child));
    }

    #[test]
    fn test_set_parent_none_detaches() {
        let mut mgr = UiManager::new();
        let parent = mgr.create_node(None);
        let child = mgr.create_node(None);

        mgr.set_parent(child, Some(parent)).unwrap();
        mgr.set_parent(child, None).unwrap();

        assert_eq!(mgr.get_node(child).unwrap().parent(), None);
        assert!(mgr.get_node(parent).unwrap().children().is_empty());
    }

    #[test]
    fn test_set_parent_reparent() {
        let mut mgr = UiManager::new();
        let p1 = mgr.create_node(None);
        let p2 = mgr.create_node(None);
        let child = mgr.create_node(None);

        mgr.set_parent(child, Some(p1)).unwrap();
        mgr.set_parent(child, Some(p2)).unwrap();

        assert!(mgr.get_node(p1).unwrap().children().is_empty());
        assert!(mgr.get_node(p2).unwrap().children().contains(&child));
        assert_eq!(mgr.get_node(child).unwrap().parent(), Some(p2));
    }

    #[test]
    fn test_cycle_detection_direct() {
        let mut mgr = UiManager::new();
        let a = mgr.create_node(None);
        let b = mgr.create_node(None);

        mgr.set_parent(b, Some(a)).unwrap();

        // Trying to make a a child of b should fail (a -> b -> a cycle).
        let result = mgr.set_parent(a, Some(b));
        assert!(result.is_err());
    }

    #[test]
    fn test_cycle_detection_indirect() {
        let mut mgr = UiManager::new();
        let a = mgr.create_node(None);
        let b = mgr.create_node(None);
        let c = mgr.create_node(None);

        mgr.set_parent(b, Some(a)).unwrap();
        mgr.set_parent(c, Some(b)).unwrap();

        // a -> b -> c; trying c as parent of a creates a -> b -> c -> a.
        let result = mgr.set_parent(a, Some(c));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_parent_self_cycle() {
        let mut mgr = UiManager::new();
        let a = mgr.create_node(None);

        let result = mgr.set_parent(a, Some(a));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_parent_nonexistent_child() {
        let mut mgr = UiManager::new();
        let parent = mgr.create_node(None);

        let result = mgr.set_parent(UiNodeId::INVALID, Some(parent));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_parent_nonexistent_parent() {
        let mut mgr = UiManager::new();
        let child = mgr.create_node(None);

        let result = mgr.set_parent(child, Some(UiNodeId::INVALID));
        assert!(result.is_err());
    }

    #[test]
    fn test_root_nodes() {
        let mut mgr = UiManager::new();
        let a = mgr.create_node(None);
        let b = mgr.create_node(None);
        let c = mgr.create_node(None);

        mgr.set_parent(c, Some(a)).unwrap();

        let roots = mgr.root_nodes();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&a));
        assert!(roots.contains(&b));
        assert!(!roots.contains(&c));
    }

    #[test]
    fn test_node_count() {
        let mut mgr = UiManager::new();
        assert_eq!(mgr.node_count(), 0);

        let a = mgr.create_node(None);
        let _b = mgr.create_node(None);
        assert_eq!(mgr.node_count(), 2);

        mgr.remove_node(a);
        assert_eq!(mgr.node_count(), 1);
    }

    #[test]
    fn test_get_node_mut() {
        let mut mgr = UiManager::new();
        let id = mgr.create_node(None);

        let node = mgr.get_node_mut(id).unwrap();
        node.set_component(Some(UiComponent::Panel));

        assert!(matches!(
            mgr.get_node(id).unwrap().component(),
            Some(UiComponent::Panel)
        ));
    }

    #[test]
    fn test_update_and_render_do_not_panic() {
        // Currently, update() and render() are placeholders with no observable
        // side effects, so this test only verifies they don't panic.
        // Assertions will be added once these methods are implemented.
        let mut mgr = UiManager::new();
        mgr.create_node(None);
        mgr.update();
        mgr.render();
    }

    #[test]
    fn test_remove_child_detaches_from_parent() {
        let mut mgr = UiManager::new();
        let parent = mgr.create_node(None);
        let child = mgr.create_node(None);
        mgr.set_parent(child, Some(parent)).unwrap();

        mgr.remove_node(child);

        let parent_node = mgr.get_node(parent).unwrap();
        assert!(parent_node.children().is_empty());
    }
}
