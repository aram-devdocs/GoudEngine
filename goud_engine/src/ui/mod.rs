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
mod manager;
mod node;
mod node_id;

pub use allocator::UiNodeAllocator;
pub use component::UiComponent;
pub use manager::UiManager;
pub use node::UiNode;
pub use node_id::UiNodeId;
