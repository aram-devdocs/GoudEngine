//! UI tree node.
//!
//! Each [`UiNode`] holds its identity, parent/child relationships, component,
//! layout properties, and computed layout rect.

use crate::core::math::{Rect, Vec2};

use super::component::UiComponent;
use super::layout::{UiAnchor, UiEdges, UiLayout};
use super::node_id::UiNodeId;
use super::theme::UiStyleOverrides;

/// A single node in the UI tree.
#[derive(Debug, Clone)]
pub struct UiNode {
    id: UiNodeId,
    parent: Option<UiNodeId>,
    children: Vec<UiNodeId>,
    component: Option<UiComponent>,
    anchor: UiAnchor,
    size: Vec2,
    margin: UiEdges,
    padding: UiEdges,
    layout: UiLayout,
    visible: bool,
    input_enabled: bool,
    focusable: bool,
    layout_enabled: bool,
    computed_rect: Rect,
    style_overrides: Option<UiStyleOverrides>,
}

impl UiNode {
    /// Creates a new node with defaults.
    pub fn new(id: UiNodeId) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            component: None,
            anchor: UiAnchor::TopLeft,
            size: Vec2::zero(),
            margin: UiEdges::ZERO,
            padding: UiEdges::ZERO,
            layout: UiLayout::None,
            visible: true,
            input_enabled: true,
            focusable: false,
            layout_enabled: true,
            computed_rect: Rect::default(),
            style_overrides: None,
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

    /// Sets this node's component.
    #[inline]
    pub fn set_component(&mut self, component: Option<UiComponent>) {
        self.focusable = component.as_ref().is_some_and(UiComponent::is_focusable);
        self.component = component;
    }

    /// Returns this node's component, if any.
    #[inline]
    pub fn component(&self) -> Option<&UiComponent> {
        self.component.as_ref()
    }

    /// Returns this node's anchor.
    #[inline]
    pub fn anchor(&self) -> UiAnchor {
        self.anchor
    }

    /// Sets this node's anchor.
    #[inline]
    pub fn set_anchor(&mut self, anchor: UiAnchor) {
        self.anchor = anchor;
    }

    /// Returns the explicit size for this node.
    #[inline]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Sets this node's explicit size.
    #[inline]
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Vec2::new(size.x.max(0.0), size.y.max(0.0));
    }

    /// Returns this node's margin.
    #[inline]
    pub fn margin(&self) -> UiEdges {
        self.margin
    }

    /// Sets this node's margin.
    #[inline]
    pub fn set_margin(&mut self, margin: UiEdges) {
        self.margin = margin;
    }

    /// Returns this node's padding.
    #[inline]
    pub fn padding(&self) -> UiEdges {
        self.padding
    }

    /// Sets this node's padding.
    #[inline]
    pub fn set_padding(&mut self, padding: UiEdges) {
        self.padding = padding;
    }

    /// Returns this node's child layout mode.
    #[inline]
    pub fn layout(&self) -> UiLayout {
        self.layout
    }

    /// Sets this node's child layout mode.
    #[inline]
    pub fn set_layout(&mut self, layout: UiLayout) {
        self.layout = layout;
    }

    /// Returns whether this node is visible.
    #[inline]
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Sets whether this node is visible.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns whether this node can receive pointer/hover input.
    #[inline]
    pub fn input_enabled(&self) -> bool {
        self.input_enabled
    }

    /// Sets whether this node can receive pointer/hover input.
    #[inline]
    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
    }

    /// Returns whether this node participates in focus traversal.
    #[inline]
    pub fn focusable(&self) -> bool {
        self.focusable
    }

    /// Sets whether this node participates in focus traversal.
    #[inline]
    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
    }

    /// Returns whether this node participates in layout.
    #[inline]
    pub fn layout_enabled(&self) -> bool {
        self.layout_enabled
    }

    /// Sets whether this node participates in layout.
    #[inline]
    pub fn set_layout_enabled(&mut self, layout_enabled: bool) {
        self.layout_enabled = layout_enabled;
    }

    /// Returns this node's computed layout rect in viewport coordinates.
    #[inline]
    pub fn computed_rect(&self) -> Rect {
        self.computed_rect
    }

    /// Sets this node's computed layout rect.
    #[inline]
    pub fn set_computed_rect(&mut self, rect: Rect) {
        self.computed_rect = rect;
    }

    /// Returns style overrides for this node.
    #[inline]
    pub fn style_overrides(&self) -> Option<&UiStyleOverrides> {
        self.style_overrides.as_ref()
    }

    /// Sets style overrides for this node.
    #[inline]
    pub fn set_style_overrides(&mut self, overrides: UiStyleOverrides) {
        self.style_overrides = Some(overrides);
    }

    /// Clears style overrides for this node.
    #[inline]
    pub fn clear_style_overrides(&mut self) {
        self.style_overrides = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{UiButton, UiFlexDirection, UiFlexLayout, UiJustify};

    #[test]
    fn test_new_node_defaults() {
        let id = UiNodeId::new(0, 1);
        let node = UiNode::new(id);

        assert_eq!(node.id(), id);
        assert_eq!(node.parent(), None);
        assert!(node.children().is_empty());
        assert!(node.component().is_none());
        assert_eq!(node.anchor(), UiAnchor::TopLeft);
        assert_eq!(node.size(), Vec2::zero());
        assert_eq!(node.margin(), UiEdges::ZERO);
        assert_eq!(node.padding(), UiEdges::ZERO);
        assert_eq!(node.layout(), UiLayout::None);
        assert!(node.visible());
        assert!(node.input_enabled());
        assert!(!node.focusable());
        assert!(node.layout_enabled());
        assert_eq!(node.computed_rect(), Rect::default());
        assert!(node.style_overrides().is_none());
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

        node.add_child(child_a);
        assert_eq!(node.children().len(), 2);

        assert!(node.remove_child(child_a));
        assert_eq!(node.children(), &[child_b]);
        assert!(!node.remove_child(child_a));
    }

    #[test]
    fn test_set_component_updates_focusable_for_button() {
        let id = UiNodeId::new(0, 1);
        let mut node = UiNode::new(id);

        node.set_component(Some(UiComponent::Panel));
        assert!(matches!(node.component(), Some(UiComponent::Panel)));
        assert!(!node.focusable());

        node.set_component(Some(UiComponent::Button(UiButton::default())));
        assert!(node.focusable());
    }

    #[test]
    fn test_set_layout_properties() {
        let id = UiNodeId::new(0, 1);
        let mut node = UiNode::new(id);

        node.set_anchor(UiAnchor::Center);
        node.set_size(Vec2::new(10.0, 20.0));
        node.set_margin(UiEdges::all(3.0));
        node.set_padding(UiEdges::new(1.0, 2.0, 3.0, 4.0));
        node.set_layout(UiLayout::Flex(UiFlexLayout {
            direction: UiFlexDirection::Column,
            justify: UiJustify::End,
            ..UiFlexLayout::default()
        }));

        assert_eq!(node.anchor(), UiAnchor::Center);
        assert_eq!(node.size(), Vec2::new(10.0, 20.0));
        assert_eq!(node.margin(), UiEdges::all(3.0));
        assert_eq!(node.padding(), UiEdges::new(1.0, 2.0, 3.0, 4.0));
        assert!(matches!(node.layout(), UiLayout::Flex(_)));
    }
}
