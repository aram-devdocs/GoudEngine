//! 2D physics synchronization system.
//!
//! Provides [`PhysicsStepSystem2D`] which steps a [`PhysicsProvider`] each frame
//! and synchronizes body positions back to ECS [`Transform2D`] components.
//!
//! # Usage
//!
//! ```rust,ignore
//! use goud_engine::ecs::app::App;
//! use goud_engine::ecs::schedule::CoreStage;
//! use goud_engine::ecs::systems::physics_sync_2d::{PhysicsStepSystem2D, PhysicsHandleMap2D};
//! use goud_engine::core::providers::impls::NullPhysicsProvider;
//!
//! let mut app = App::new();
//! app.insert_resource(PhysicsHandleMap2D::default());
//! app.add_system(
//!     CoreStage::Update,
//!     PhysicsStepSystem2D::new(Box::new(NullPhysicsProvider::new())),
//! );
//! ```

use std::collections::HashMap;

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::BodyHandle;
use crate::ecs::entity::Entity;
use crate::ecs::query::Access;
use crate::ecs::system::System;
use crate::ecs::World;

/// Resource that maps ECS entities to physics body handles.
///
/// This resource tracks which entities have been registered with the physics
/// provider. Users register entities by inserting them into this map before
/// the physics step runs.
///
/// # Example
///
/// ```rust,ignore
/// let mut map = PhysicsHandleMap2D::default();
/// map.entity_to_body.insert(entity, body_handle);
/// world.insert_resource(map);
/// ```
#[derive(Debug, Default)]
pub struct PhysicsHandleMap2D {
    /// Maps entities to their physics body handles.
    pub entity_to_body: HashMap<Entity, BodyHandle>,
}

/// System that steps the 2D physics simulation and writes positions back.
///
/// Each frame this system:
/// 1. Steps the physics provider by a fixed timestep (1/60 s).
/// 2. For each entity tracked in [`PhysicsHandleMap2D`], reads the body
///    position from the provider and updates the entity's `Transform2D`.
///
/// The system owns the physics provider. Users add bodies to the provider
/// directly and register the entity-to-handle mapping in `PhysicsHandleMap2D`.
pub struct PhysicsStepSystem2D {
    provider: Box<dyn PhysicsProvider>,
}

impl PhysicsStepSystem2D {
    /// Creates a new 2D physics step system with the given provider.
    pub fn new(provider: Box<dyn PhysicsProvider>) -> Self {
        Self { provider }
    }

    /// Returns a reference to the underlying physics provider.
    pub fn provider(&self) -> &dyn PhysicsProvider {
        &*self.provider
    }

    /// Returns a mutable reference to the underlying physics provider.
    pub fn provider_mut(&mut self) -> &mut dyn PhysicsProvider {
        &mut *self.provider
    }
}

impl System for PhysicsStepSystem2D {
    fn name(&self) -> &'static str {
        "PhysicsStepSystem2D"
    }

    fn component_access(&self) -> Access {
        // The system reads/writes Transform2D and reads the handle map resource,
        // but since we access these through World methods rather than queries,
        // we return empty access. The scheduler treats this as opaque.
        Access::new()
    }

    fn run(&mut self, world: &mut World) {
        // Step the physics simulation at a fixed 60 Hz timestep.
        const FIXED_DT: f32 = 1.0 / 60.0;
        if let Err(e) = self.provider.step(FIXED_DT) {
            log::error!("PhysicsStepSystem2D: step failed: {e}");
            return;
        }

        // Read back positions from the physics provider into Transform2D.
        // We need to take the handle map out of the world temporarily to
        // avoid holding an immutable borrow while mutating transforms.
        let handle_map = match world.get_resource_mut::<PhysicsHandleMap2D>() {
            Some(map) => {
                // Collect entries to avoid borrowing world during iteration.
                map.entity_to_body
                    .iter()
                    .map(|(&entity, &handle)| (entity, handle))
                    .collect::<Vec<_>>()
            }
            None => return,
        };

        for (entity, body_handle) in handle_map {
            if let Ok(pos) = self.provider.body_position(body_handle) {
                use crate::core::math::Vec2;
                use crate::ecs::components::transform2d::Transform2D;
                if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                    transform.set_position(Vec2::new(pos[0], pos[1]));
                }
            }
        }
    }
}

// SAFETY: PhysicsProvider: Provider requires Send + Sync + 'static.
// Box<dyn PhysicsProvider> is Send because Provider: Send.
// This makes PhysicsStepSystem2D: Send, satisfying the System trait bound.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::impls::NullPhysicsProvider;

    #[test]
    fn test_handle_map_default() {
        let map = PhysicsHandleMap2D::default();
        assert!(map.entity_to_body.is_empty());
    }

    #[test]
    fn test_system_construction() {
        let provider = NullPhysicsProvider::new();
        let system = PhysicsStepSystem2D::new(Box::new(provider));
        assert_eq!(system.name(), "PhysicsStepSystem2D");
    }

    #[test]
    fn test_system_run_empty_world() {
        let provider = NullPhysicsProvider::new();
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        let mut world = World::new();
        // Should not panic even without the resource.
        system.run(&mut world);
    }

    #[test]
    fn test_system_run_with_empty_handle_map() {
        let provider = NullPhysicsProvider::new();
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap2D::default());
        // Should not panic with an empty map.
        system.run(&mut world);
    }

    #[test]
    fn test_provider_accessors() {
        let provider = NullPhysicsProvider::new();
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        assert_eq!(system.provider().name(), "null");
        assert_eq!(system.provider_mut().gravity(), [0.0, 0.0]);
    }

    #[test]
    fn test_system_should_run() {
        let provider = NullPhysicsProvider::new();
        let system = PhysicsStepSystem2D::new(Box::new(provider));
        let world = World::new();
        assert!(system.should_run(&world));
    }

    #[test]
    fn test_system_component_access_is_empty() {
        let provider = NullPhysicsProvider::new();
        let system = PhysicsStepSystem2D::new(Box::new(provider));
        assert!(system.component_access().is_empty());
    }
}
