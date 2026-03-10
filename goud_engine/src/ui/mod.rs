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
pub use theme::{UiComponentVisual, UiStyleOverrides, UiTheme, UiVisualStyle};
pub use visuals::{resolve_widget_visual, UiInteractionState};
