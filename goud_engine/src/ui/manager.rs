//! UI tree manager.
//!
//! [`UiManager`] owns UI nodes, computes deterministic layout, and processes UI
//! input semantics (hover, focus, and button activation/click dispatch).

mod input;
mod layout;

use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Rect;

use super::allocator::UiNodeAllocator;
use super::component::UiComponent;
use super::node::UiNode;
use super::node_id::UiNodeId;

/// Events emitted by the UI manager during input processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    /// Mouse pointer started hovering this node.
    HoverEnter(UiNodeId),
    /// Mouse pointer stopped hovering this node.
    HoverLeave(UiNodeId),
    /// Node focus changed.
    FocusChanged {
        /// Previous focused node.
        previous: Option<UiNodeId>,
        /// New focused node.
        current: Option<UiNodeId>,
    },
    /// Button-like activation happened.
    Click(UiNodeId),
}

/// Owns and manages the UI node tree.
#[derive(Debug)]
pub struct UiManager {
    allocator: UiNodeAllocator,
    nodes: HashMap<UiNodeId, UiNode>,
    layout_dirty: bool,
    layout_epoch: u64,
    viewport_size: (u32, u32),
    hovered_node: Option<UiNodeId>,
    focused_node: Option<UiNodeId>,
    pressed_pointer_node: Option<UiNodeId>,
    frame_events: Vec<UiEvent>,
}

impl UiManager {
    /// Creates an empty UI manager.
    pub fn new() -> Self {
        Self {
            allocator: UiNodeAllocator::new(),
            nodes: HashMap::new(),
            layout_dirty: true,
            layout_epoch: 0,
            viewport_size: (0, 0),
            hovered_node: None,
            focused_node: None,
            pressed_pointer_node: None,
            frame_events: Vec::new(),
        }
    }

    /// Creates a new node, optionally attaching a component.
    pub fn create_node(&mut self, component: Option<UiComponent>) -> UiNodeId {
        let id = self.allocator.allocate();
        let mut node = UiNode::new(id);
        node.set_component(component);
        self.nodes.insert(id, node);
        self.mark_layout_dirty();
        id
    }

    /// Removes a node and its subtree recursively.
    pub fn remove_node(&mut self, id: UiNodeId) -> bool {
        let subtree = self.collect_subtree(id);
        if subtree.is_empty() {
            return false;
        }

        if let Some(parent_id) = self.nodes.get(&id).and_then(UiNode::parent) {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.remove_child(id);
            }
        }

        for node_id in &subtree {
            self.nodes.remove(node_id);
            self.allocator.deallocate(*node_id);
        }

        if let Some(hovered) = self.hovered_node.filter(|n| subtree.contains(n)) {
            self.frame_events.push(UiEvent::HoverLeave(hovered));
            self.hovered_node = None;
        }
        if self.focused_node.is_some_and(|n| subtree.contains(&n)) {
            self.set_focus(None);
        }
        if self
            .pressed_pointer_node
            .is_some_and(|n| subtree.contains(&n))
        {
            self.pressed_pointer_node = None;
        }

        self.mark_layout_dirty();
        true
    }

    /// Sets `child`'s parent to `parent`, or detaches with `None`.
    pub fn set_parent(&mut self, child: UiNodeId, parent: Option<UiNodeId>) -> GoudResult<()> {
        if !self.nodes.contains_key(&child) {
            return Err(GoudError::InvalidState(
                "Child node does not exist".to_string(),
            ));
        }

        if let Some(pid) = parent {
            if !self.nodes.contains_key(&pid) {
                return Err(GoudError::InvalidState(
                    "Parent node does not exist".to_string(),
                ));
            }
            if self.is_ancestor(child, pid) {
                return Err(GoudError::InvalidState(
                    "Setting this parent would create a cycle".to_string(),
                ));
            }
        }

        let old_parent = self.nodes.get(&child).and_then(UiNode::parent);
        if let Some(old_pid) = old_parent {
            if let Some(old_parent_node) = self.nodes.get_mut(&old_pid) {
                old_parent_node.remove_child(child);
            }
        }

        if let Some(pid) = parent {
            if let Some(new_parent) = self.nodes.get_mut(&pid) {
                new_parent.add_child(child);
            }
        }

        if let Some(node) = self.nodes.get_mut(&child) {
            node.set_parent(parent);
        }

        self.mark_layout_dirty();
        Ok(())
    }

    /// Returns a reference to a node.
    #[inline]
    pub fn get_node(&self, id: UiNodeId) -> Option<&UiNode> {
        self.nodes.get(&id)
    }

    /// Returns a mutable reference to a node, marking layout dirty conservatively.
    #[inline]
    pub fn get_node_mut(&mut self, id: UiNodeId) -> Option<&mut UiNode> {
        self.mark_layout_dirty();
        self.nodes.get_mut(&id)
    }

    /// Returns IDs of all root nodes (deterministic order).
    pub fn root_nodes(&self) -> Vec<UiNodeId> {
        let mut roots: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.parent().is_none())
            .map(UiNode::id)
            .collect();
        sort_node_ids(&mut roots);
        roots
    }

    /// Returns the number of live nodes.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Sets viewport size. Layout is dirtied if changed.
    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        if self.viewport_size != (width, height) {
            self.viewport_size = (width, height);
            self.mark_layout_dirty();
        }
    }

    /// Returns viewport size in pixels.
    #[inline]
    pub fn viewport_size(&self) -> (u32, u32) {
        self.viewport_size
    }

    /// Returns the layout epoch, incremented once per recompute pass.
    #[inline]
    pub fn layout_epoch(&self) -> u64 {
        self.layout_epoch
    }

    /// Returns a node's computed rect.
    #[inline]
    pub fn computed_rect(&self, id: UiNodeId) -> Option<Rect> {
        self.nodes.get(&id).map(UiNode::computed_rect)
    }

    /// Returns the currently hovered node.
    #[inline]
    pub fn hovered_node(&self) -> Option<UiNodeId> {
        self.hovered_node
    }

    /// Returns the currently focused node.
    #[inline]
    pub fn focused_node(&self) -> Option<UiNodeId> {
        self.focused_node
    }

    /// Returns all UI events emitted in the current frame.
    #[inline]
    pub fn events(&self) -> &[UiEvent] {
        &self.frame_events
    }

    /// Takes and clears all UI events for this frame.
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.frame_events)
    }

    fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
    }

    fn node_focusable(&self, node_id: UiNodeId) -> bool {
        self.nodes
            .get(&node_id)
            .is_some_and(|n| self.node_and_ancestors_input_enabled(node_id) && n.focusable())
    }

    fn node_is_clickable_button(&self, node_id: UiNodeId) -> bool {
        self.nodes.get(&node_id).is_some_and(|n| {
            self.node_and_ancestors_input_enabled(node_id)
                && n.component()
                    .is_some_and(|component| component.is_button() && component.is_interactive())
        })
    }

    fn set_focus(&mut self, next: Option<UiNodeId>) {
        if self.focused_node == next {
            return;
        }
        let previous = self.focused_node;
        self.focused_node = next;
        self.frame_events.push(UiEvent::FocusChanged {
            previous,
            current: next,
        });
    }

    fn clear_stale_ui_state(&mut self) {
        if let Some(hovered) = self.hovered_node {
            if !self.node_exists_and_interactive(hovered) {
                self.frame_events.push(UiEvent::HoverLeave(hovered));
                self.hovered_node = None;
            }
        }
        if self.focused_node.is_some_and(|id| !self.node_focusable(id)) {
            self.set_focus(None);
        }
        if self
            .pressed_pointer_node
            .is_some_and(|id| !self.node_exists_and_interactive(id))
        {
            self.pressed_pointer_node = None;
        }
    }

    fn node_exists_and_interactive(&self, id: UiNodeId) -> bool {
        self.nodes.get(&id).is_some_and(|node| {
            self.node_and_ancestors_input_enabled(id)
                && node.component().is_some_and(UiComponent::is_interactive)
        })
    }

    fn node_and_ancestors_input_enabled(&self, node_id: UiNodeId) -> bool {
        let mut current = Some(node_id);
        while let Some(id) = current {
            let Some(node) = self.nodes.get(&id) else {
                return false;
            };
            if !node.visible() || !node.input_enabled() {
                return false;
            }
            current = node.parent();
        }
        true
    }

    fn is_ancestor(&self, ancestor: UiNodeId, node: UiNodeId) -> bool {
        let mut current = Some(node);
        while let Some(cur) = current {
            if cur == ancestor {
                return true;
            }
            current = self.nodes.get(&cur).and_then(UiNode::parent);
        }
        false
    }

    fn collect_subtree(&self, root: UiNodeId) -> Vec<UiNodeId> {
        let mut out = Vec::new();
        let mut stack = vec![root];

        while let Some(id) = stack.pop() {
            if let Some(node) = self.nodes.get(&id) {
                out.push(id);
                for &child in node.children() {
                    stack.push(child);
                }
            }
        }

        out
    }

    /// Placeholder for future UI rendering.
    pub fn render(&self) {
        // Rendering is implemented in later iterations.
    }
}

impl Default for UiManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn sort_node_ids(ids: &mut [UiNodeId]) {
    ids.sort_by_key(|id| (id.index(), id.generation()));
}
