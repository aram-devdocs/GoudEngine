//! # SDK Component Re-exports
//!
//! This module provides convenient re-exports of all ECS components
//! for use in Rust-native game development.
//!
//! ## Available Components
//!
//! ### Spatial Transformations
//!
//! - [`Transform2D`]: 2D local-space position, rotation, scale
//! - [`Transform`]: 3D local-space position, rotation, scale
//! - [`GlobalTransform2D`]: 2D world-space transformation (computed from hierarchy)
//! - [`GlobalTransform`]: 3D world-space transformation (computed from hierarchy)
//! - [`Mat3x3`]: 3x3 transformation matrix for 2D transforms
//!
//! ### Rendering
//!
//! - [`Sprite`]: 2D sprite component for textured quads
//!
//! ### Audio
//!
//! - [`AudioSource`]: Audio playback component
//! - [`AudioChannel`]: Audio channel for mixing
//! - [`AttenuationModel`]: Distance-based volume falloff
//!
//! ### Physics
//!
//! - [`RigidBody`]: Physics body (dynamic, kinematic, static)
//! - [`RigidBodyType`]: Type of rigid body
//! - [`Collider`]: Collision shape
//! - [`ColliderShape`]: Geometric shapes for colliders
//!
//! ### Hierarchy
//!
//! - [`Parent`]: Parent entity reference
//! - [`Children`]: Child entity list
//! - [`Name`]: Human-readable entity name
//!
//! ## Example
//!
//! ```rust
//! use goud_engine::sdk::components::{Transform2D, Sprite, RigidBody, RigidBodyType};
//! use goud_engine::core::math::Vec2;
//!
//! // Create components for a game entity
//! let transform = Transform2D::from_position(Vec2::new(100.0, 200.0));
//! let rigidbody = RigidBody::new(RigidBodyType::Dynamic);
//! ```

// =============================================================================
// Spatial Transforms
// =============================================================================

pub use crate::ecs::components::GlobalTransform;
pub use crate::ecs::components::GlobalTransform2D;
pub use crate::ecs::components::Mat3x3;
pub use crate::ecs::components::Transform;
pub use crate::ecs::components::Transform2D;

// =============================================================================
// Rendering
// =============================================================================

pub use crate::ecs::components::Sprite;

// =============================================================================
// Audio
// =============================================================================

pub use crate::ecs::components::AttenuationModel;
pub use crate::ecs::components::AudioChannel;
pub use crate::ecs::components::AudioSource;

// =============================================================================
// Physics
// =============================================================================

pub use crate::ecs::components::Collider;
pub use crate::ecs::components::ColliderShape;
pub use crate::ecs::components::RigidBody;
pub use crate::ecs::components::RigidBodyType;

// =============================================================================
// Hierarchy
// =============================================================================

pub use crate::ecs::components::Children;
pub use crate::ecs::components::Name;
pub use crate::ecs::components::Parent;

// =============================================================================
// Transform Propagation
// =============================================================================

pub use crate::ecs::components::propagation::{propagate_transforms, propagate_transforms_2d};

// Note: Component trait and Vec2 are re-exported at the sdk level
// to avoid duplicate exports. Use `use goud_engine::sdk::Component;`
// or `use goud_engine::sdk::Vec2;` instead.

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::Component;

    #[test]
    fn test_transform2d_reexport() {
        let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
        assert_eq!(t.position, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_global_transform2d_reexport() {
        let gt = GlobalTransform2D::IDENTITY;
        assert_eq!(gt.translation(), Vec2::zero());
    }

    #[test]
    fn test_rigidbody_reexport() {
        let rb = RigidBody::new(RigidBodyType::Dynamic);
        assert_eq!(rb.body_type, RigidBodyType::Dynamic);
    }

    #[test]
    fn test_hierarchy_reexport() {
        let name = Name::new("Test Entity");
        assert_eq!(name.as_str(), "Test Entity");

        let children = Children::new();
        assert!(children.is_empty());
    }

    #[test]
    fn test_component_trait_reexport() {
        // Verify Component trait is accessible
        fn assert_component<T: Component>() {}
        assert_component::<Transform2D>();
        assert_component::<Sprite>();
        assert_component::<RigidBody>();
    }
}
