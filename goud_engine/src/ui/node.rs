//! UI tree node.
//!
//! Each [`UiNode`] holds its identity, parent/child relationships, and an
//! optional [`UiComponent`] describing what kind of widget it represents.

use super::component::UiComponent;
use super::node_id::UiNodeId;

// =============================================================================
// UiNode
// =============================================================================

/// A single node in the UI tree.
///
/// Nodes form a hierarchy via parent/child references. Each node optionally
/// carries a [`UiComponent`] that determines its visual behaviour.
#[derive(Debug, Clone)]
pub struct UiNode {
    /// This node's unique identifier.
    id: UiNodeId,

    /// Parent node, or `None` for root nodes.
    parent: Option<UiNodeId>,

    /// Ordered list of child node IDs.
    children: Vec<UiNodeId>,

    /// The widget attached to this node, if any.
    component: Option<UiComponent>,
}

impl UiNode {
    /// Creates a new node with the given ID and no parent, children, or component.
    pub fn new(id: UiNodeId) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            component: None,
        }
    }

    /// Returns this node's ID.
    #[inline]
    pub fn id(&self) -> UiNodeId {
        self.id
    }

    /// Returns the parent node ID, if any.
    #[inline]
    pub fn parent(&self) -> Option<UiNodeId> {
        self.parent
    }

    /// Returns a slice of child node IDs.
    #[inline]
    pub fn children(&self) -> &[UiNodeId] {
        &self.children
    }

    /// Sets the parent of this node.
    #[inline]
    pub fn set_parent(&mut self, parent: Option<UiNodeId>) {
        self.parent = parent;
    }

    /// Adds a child to this node. Does nothing if the child is already present.
    pub fn add_child(&mut self, child: UiNodeId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Removes a child from this node. Returns `true` if the child was present.
    pub fn remove_child(&mut self, child: UiNodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&c| c == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Sets the component for this node.
    #[inline]
    pub fn set_component(&mut self, component: Option<UiComponent>) {
        self.component = component;
    }

    /// Returns a reference to this node's component, if any.
    #[inline]
    pub fn component(&self) -> Option<&UiComponent> {
        self.component.as_ref()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_node_defaults() {
        let id = UiNodeId::new(0, 1);
        let node = UiNode::new(id);

        assert_eq!(node.id(), id);
        assert_eq!(node.parent(), None);
        assert!(node.children().is_empty());
        assert!(node.component().is_none());
    }

    #[test]
    fn test_set_parent() {
        let id = UiNodeId::new(0, 1);
        let parent_id = UiNodeId::new(1, 1);
        let mut node = UiNode::new(id);

        node.set_parent(Some(parent_id));
        assert_eq!(node.parent(), Some(parent_id));

        node.set_parent(None);
        assert_eq!(node.parent(), None);
    }

    #[test]
    fn test_add_and_remove_children() {
        let id = UiNodeId::new(0, 1);
        let child_a = UiNodeId::new(1, 1);
        let child_b = UiNodeId::new(2, 1);
        let mut node = UiNode::new(id);

        node.add_child(child_a);
        node.add_child(child_b);
        assert_eq!(node.children(), &[child_a, child_b]);

        // Adding duplicate is a no-op
        node.add_child(child_a);
        assert_eq!(node.children().len(), 2);

        assert!(node.remove_child(child_a));
        assert_eq!(node.children(), &[child_b]);

        // Removing non-existent child returns false
        assert!(!node.remove_child(child_a));
    }

    #[test]
    fn test_set_component() {
        let id = UiNodeId::new(0, 1);
        let mut node = UiNode::new(id);

        node.set_component(Some(UiComponent::Panel));
        assert!(matches!(node.component(), Some(UiComponent::Panel)));

        node.set_component(None);
        assert!(node.component().is_none());
    }
}
