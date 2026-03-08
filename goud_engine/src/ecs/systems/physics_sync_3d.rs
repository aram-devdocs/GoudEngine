//! 3D physics synchronization system.
//!
//! Provides [`PhysicsStepSystem3D`] which steps a [`PhysicsProvider3D`] each
//! frame and synchronizes body positions and rotations back to ECS
//! [`Transform`] components.
//!
//! # Usage
//!
//! ```rust,ignore
//! use goud_engine::ecs::app::App;
//! use goud_engine::ecs::schedule::CoreStage;
//! use goud_engine::ecs::systems::physics_sync_3d::{PhysicsStepSystem3D, PhysicsHandleMap3D};
//! use goud_engine::core::providers::impls::NullPhysicsProvider3D;
//!
//! let mut app = App::new();
//! app.insert_resource(PhysicsHandleMap3D::default());
//! app.add_system(
//!     CoreStage::Update,
//!     PhysicsStepSystem3D::new(Box::new(NullPhysicsProvider3D::new())),
//! );
//! ```

use std::collections::HashMap;
use std::time::Instant;

use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::BodyHandle;
use crate::ecs::entity::Entity;
use crate::ecs::physics_world::interpolation::PhysicsInterpolation;
use crate::ecs::physics_world::PhysicsWorld;
use crate::ecs::query::Access;
use crate::ecs::system::System;
use crate::ecs::World;

/// Resource that maps ECS entities to physics body handles for 3D.
#[derive(Debug, Default)]
pub struct PhysicsHandleMap3D {
    /// Maps entities to their physics body handles.
    pub entity_to_body: HashMap<Entity, BodyHandle>,
}

/// System that steps the 3D physics simulation and writes positions back.
///
/// Each frame this system:
/// 1. Steps the physics provider by a fixed timestep (1/60 s).
/// 2. For each entity tracked in [`PhysicsHandleMap3D`], reads the body
///    position and rotation from the provider and updates the entity's
///    `Transform`.
pub struct PhysicsStepSystem3D {
    provider: Box<dyn PhysicsProvider3D>,
    last_step: Option<Instant>,
}

impl PhysicsStepSystem3D {
    /// Creates a new 3D physics step system with the given provider.
    pub fn new(provider: Box<dyn PhysicsProvider3D>) -> Self {
        Self {
            provider,
            last_step: None,
        }
    }

    /// Returns a reference to the underlying physics provider.
    pub fn provider(&self) -> &dyn PhysicsProvider3D {
        &*self.provider
    }

    /// Returns a mutable reference to the underlying physics provider.
    pub fn provider_mut(&mut self) -> &mut dyn PhysicsProvider3D {
        &mut *self.provider
    }
}

impl System for PhysicsStepSystem3D {
    fn name(&self) -> &'static str {
        "PhysicsStepSystem3D"
    }

    fn component_access(&self) -> Access {
        Access::new()
    }

    fn run(&mut self, world: &mut World) {
        // Compute frame delta from wall clock.
        let now = Instant::now();
        let delta = match self.last_step {
            Some(prev) => now.duration_since(prev).as_secs_f32(),
            None => 1.0 / 60.0,
        };
        self.last_step = Some(now);

        // Use PhysicsWorld accumulator if present, otherwise fall back to a
        // single step at 1/60.
        if let Some(physics_world) = world.get_resource_mut::<PhysicsWorld>() {
            physics_world.advance(delta);
            let timestep = physics_world.timestep();
            while physics_world.should_step() {
                physics_world.step();
                if let Err(e) = self.provider.step(timestep) {
                    log::error!("PhysicsStepSystem3D: step failed: {e}");
                    return;
                }
            }
            let alpha = physics_world.interpolation_alpha();
            world.insert_resource(PhysicsInterpolation { alpha });
        } else {
            const FIXED_DT: f32 = 1.0 / 60.0;
            if let Err(e) = self.provider.step(FIXED_DT) {
                log::error!("PhysicsStepSystem3D: step failed: {e}");
                return;
            }
        }

        let handle_map = match world.get_resource_mut::<PhysicsHandleMap3D>() {
            Some(map) => map
                .entity_to_body
                .iter()
                .map(|(&entity, &handle)| (entity, handle))
                .collect::<Vec<_>>(),
            None => return,
        };

        for (entity, body_handle) in handle_map {
            if let Ok(pos) = self.provider.body_position(body_handle) {
                use crate::core::math::Vec3;
                use crate::ecs::components::transform::Transform;
                if let Some(transform) = world.get_mut::<Transform>(entity) {
                    transform.set_position(Vec3::new(pos[0], pos[1], pos[2]));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::impls::NullPhysicsProvider3D;

    #[test]
    fn test_handle_map_3d_default() {
        let map = PhysicsHandleMap3D::default();
        assert!(map.entity_to_body.is_empty());
    }

    #[test]
    fn test_system_3d_construction() {
        let provider = NullPhysicsProvider3D::new();
        let system = PhysicsStepSystem3D::new(Box::new(provider));
        assert_eq!(system.name(), "PhysicsStepSystem3D");
    }

    #[test]
    fn test_system_3d_run_empty_world() {
        let provider = NullPhysicsProvider3D::new();
        let mut system = PhysicsStepSystem3D::new(Box::new(provider));
        let mut world = World::new();
        system.run(&mut world);
    }

    #[test]
    fn test_system_3d_run_with_empty_handle_map() {
        let provider = NullPhysicsProvider3D::new();
        let mut system = PhysicsStepSystem3D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap3D::default());
        system.run(&mut world);
    }

    #[test]
    fn test_system_3d_provider_accessors() {
        let provider = NullPhysicsProvider3D::new();
        let mut system = PhysicsStepSystem3D::new(Box::new(provider));
        assert_eq!(system.provider().name(), "null");
        assert_eq!(system.provider_mut().gravity(), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_system_3d_should_run() {
        let provider = NullPhysicsProvider3D::new();
        let system = PhysicsStepSystem3D::new(Box::new(provider));
        let world = World::new();
        assert!(system.should_run(&world));
    }

    #[test]
    fn test_system_3d_with_physics_world_produces_interpolation() {
        let provider = NullPhysicsProvider3D::new();
        let mut system = PhysicsStepSystem3D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap3D::default());
        world.insert_resource(PhysicsWorld::new());

        system.run(&mut world);

        assert!(world.contains_resource::<PhysicsInterpolation>());
    }

    #[test]
    fn test_system_3d_fallback_without_physics_world() {
        let provider = NullPhysicsProvider3D::new();
        let mut system = PhysicsStepSystem3D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap3D::default());

        system.run(&mut world);

        assert!(!world.contains_resource::<PhysicsInterpolation>());
    }
}
