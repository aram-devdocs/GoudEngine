//! UI node tree system.
//!
//! This module provides a standalone UI hierarchy, separate from the ECS
//! [`World`](crate::ecs::World). Nodes form a tree via parent/child
//! relationships and each node can carry a [`UiComponent`] describing its
//! widget type.
//!
//! # Key Types
//!
//! - [`UiNodeId`] -- generational identifier for a UI node
//! - [`UiNode`] -- a single node in the tree
//! - [`UiComponent`] -- the widget variant attached to a node
//! - [`UiManager`] -- owns and manages the full node tree

mod allocator;
mod component;
mod layout;
mod manager;
mod node;
mod node_id;
mod render_commands;
#[cfg(test)]
mod tests;
mod theme;
mod visuals;

pub use allocator::UiNodeAllocator;
pub use component::{UiButton, UiComponent, UiImage, UiLabel, UiSlider};
pub use layout::{UiAlign, UiAnchor, UiEdges, UiFlexDirection, UiFlexLayout, UiJustify, UiLayout};
pub use manager::{UiEvent, UiManager};
pub use node::UiNode;
pub use node_id::UiNodeId;
pub use render_commands::{UiQuadCommand, UiRenderCommand, UiTextCommand, UiTexturedQuadCommand};
pub use theme::{
    UiComponentVisual, UiStyleOverrides, UiTheme, UiVisualStyle, UI_DEFAULT_FONT_ASSET_PATH,
    UI_DEFAULT_FONT_FAMILY,
};
pub use visuals::{resolve_widget_visual, UiInteractionState};

/// Common representation of an engine `UiEvent` mapped to packed node IDs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PackedUiEvent {
    /// Numeric UI event kind code.
    pub event_kind: u32,
    /// Packed source node ID.
    pub node_id: u64,
    /// Packed previous focus/hover node ID, or `u64::MAX`.
    pub previous_node_id: u64,
    /// Packed current focus/hover node ID, or `u64::MAX`.
    pub current_node_id: u64,
}

/// Maps an FFI/SDK widget-kind code to a [`UiComponent`].
///
/// * `-1` -> `Some(None)` (explicitly no component)
/// * `0` -> `Some(Some(UiComponent::Panel))`
/// * `1` -> `Some(Some(UiComponent::Button(UiButton::default())))`
/// * `2` -> `Some(Some(UiComponent::Label(UiLabel::default())))`
/// * `3` -> `Some(Some(UiComponent::Image(UiImage::default())))`
/// * `4` -> `Some(Some(UiComponent::Slider(UiSlider::new(0.0, 1.0, 0.0))))`
/// * anything else -> `None`
pub(crate) fn component_from_widget_kind(widget_kind: i32) -> Option<Option<UiComponent>> {
    match widget_kind {
        -1 => Some(None),
        0 => Some(Some(UiComponent::Panel)),
        1 => Some(Some(UiComponent::Button(UiButton::default()))),
        2 => Some(Some(UiComponent::Label(UiLabel::default()))),
        3 => Some(Some(UiComponent::Image(UiImage::default()))),
        4 => Some(Some(UiComponent::Slider(UiSlider::new(0.0, 1.0, 0.0)))),
        _ => None,
    }
}

/// Maps an engine UI event to packed IDs for FFI/WASM consumers.
pub(crate) fn map_ui_event(event: UiEvent) -> PackedUiEvent {
    let invalid = u64::MAX;
    let pack_node_id = |id: UiNodeId| (id.index() as u64) | ((id.generation() as u64) << 32);

    match event {
        UiEvent::HoverEnter(node) => PackedUiEvent {
            event_kind: 0,
            node_id: pack_node_id(node),
            previous_node_id: invalid,
            current_node_id: invalid,
        },
        UiEvent::HoverLeave(node) => PackedUiEvent {
            event_kind: 1,
            node_id: pack_node_id(node),
            previous_node_id: invalid,
            current_node_id: invalid,
        },
        UiEvent::FocusChanged { previous, current } => PackedUiEvent {
            event_kind: 2,
            node_id: current.or(previous).map(pack_node_id).unwrap_or(invalid),
            previous_node_id: previous.map(pack_node_id).unwrap_or(invalid),
            current_node_id: current.map(pack_node_id).unwrap_or(invalid),
        },
        UiEvent::Click(node) => PackedUiEvent {
            event_kind: 3,
            node_id: pack_node_id(node),
            previous_node_id: invalid,
            current_node_id: invalid,
        },
    }
}
