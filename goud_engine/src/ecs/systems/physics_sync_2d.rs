//! 2D physics synchronization system.
//!
//! Provides [`PhysicsStepSystem2D`] which steps a [`PhysicsProvider`] each frame
//! and synchronizes body positions back to ECS
//! [`Transform2D`](crate::ecs::components::Transform2D) components.
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
use std::time::Instant;

use crate::core::event::Events;
use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::{BodyHandle, CollisionEvent};
use crate::ecs::entity::Entity;
use crate::ecs::physics_world::interpolation::PhysicsInterpolation;
use crate::ecs::physics_world::PhysicsWorld;
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
/// 1. Steps the physics provider using the `PhysicsWorld` accumulator when
///    available, falling back to a single step at 1/60 s otherwise.
/// 2. For each entity tracked in [`PhysicsHandleMap2D`], reads the body
///    position from the provider and updates the entity's `Transform2D`.
///
/// The system owns the physics provider. Users add bodies to the provider
/// directly and register the entity-to-handle mapping in `PhysicsHandleMap2D`.
pub struct PhysicsStepSystem2D {
    provider: Box<dyn PhysicsProvider>,
    last_step: Option<Instant>,
}

impl PhysicsStepSystem2D {
    /// Creates a new 2D physics step system with the given provider.
    pub fn new(provider: Box<dyn PhysicsProvider>) -> Self {
        Self {
            provider,
            last_step: None,
        }
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
        // Compute frame delta from wall clock.
        let now = Instant::now();
        let delta = match self.last_step {
            Some(prev) => now.duration_since(prev).as_secs_f32(),
            None => 1.0 / 60.0, // Assume 60 FPS on first frame
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
                    log::error!("PhysicsStepSystem2D: step failed: {e}");
                    return;
                }
            }
            let alpha = physics_world.interpolation_alpha();
            world.insert_resource(PhysicsInterpolation { alpha });
        } else {
            // Fallback: single step at default 60 Hz
            const FIXED_DT: f32 = 1.0 / 60.0;
            if let Err(e) = self.provider.step(FIXED_DT) {
                log::error!("PhysicsStepSystem2D: step failed: {e}");
                return;
            }
        }

        // Forward provider collision events into ECS Events resource when present.
        let collision_events = self.provider.drain_collision_events();
        if !collision_events.is_empty() {
            if let Some(events) = world.get_resource_mut::<Events<CollisionEvent>>() {
                for event in collision_events {
                    events.send(event);
                }
            }
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
    use crate::core::error::GoudResult;
    use crate::core::event::Events;
    use crate::core::providers::impls::NullPhysicsProvider;
    use crate::core::providers::types::{
        BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, CollisionEventKind,
        ContactPair, DebugShape, JointDesc, JointHandle, PhysicsCapabilities, RaycastHit,
    };
    use crate::core::providers::{Provider, ProviderLifecycle};

    struct TestCollisionEventProvider {
        capabilities: PhysicsCapabilities,
        pending_events: Vec<CollisionEvent>,
    }

    impl TestCollisionEventProvider {
        fn with_events(events: Vec<CollisionEvent>) -> Self {
            Self {
                capabilities: PhysicsCapabilities::default(),
                pending_events: events,
            }
        }
    }

    impl Provider for TestCollisionEventProvider {
        fn name(&self) -> &str {
            "test-collision-events"
        }

        fn version(&self) -> &str {
            "0.0-test"
        }

        fn capabilities(&self) -> Box<dyn std::any::Any> {
            Box::new(self.capabilities.clone())
        }
    }

    impl ProviderLifecycle for TestCollisionEventProvider {
        fn init(&mut self) -> GoudResult<()> {
            Ok(())
        }

        fn update(&mut self, _delta: f32) -> GoudResult<()> {
            Ok(())
        }

        fn shutdown(&mut self) {}
    }

    impl PhysicsProvider for TestCollisionEventProvider {
        fn physics_capabilities(&self) -> &PhysicsCapabilities {
            &self.capabilities
        }

        fn step(&mut self, _delta: f32) -> GoudResult<()> {
            Ok(())
        }

        fn set_gravity(&mut self, _gravity: [f32; 2]) {}

        fn gravity(&self) -> [f32; 2] {
            [0.0, 0.0]
        }

        fn set_timestep(&mut self, _dt: f32) {}

        fn timestep(&self) -> f32 {
            1.0 / 60.0
        }

        fn create_body(&mut self, _desc: &BodyDesc) -> GoudResult<BodyHandle> {
            Ok(BodyHandle(1))
        }

        fn destroy_body(&mut self, _handle: BodyHandle) {}

        fn body_position(&self, _handle: BodyHandle) -> GoudResult<[f32; 2]> {
            Ok([0.0, 0.0])
        }

        fn set_body_position(&mut self, _handle: BodyHandle, _pos: [f32; 2]) -> GoudResult<()> {
            Ok(())
        }

        fn body_velocity(&self, _handle: BodyHandle) -> GoudResult<[f32; 2]> {
            Ok([0.0, 0.0])
        }

        fn set_body_velocity(&mut self, _handle: BodyHandle, _vel: [f32; 2]) -> GoudResult<()> {
            Ok(())
        }

        fn apply_force(&mut self, _handle: BodyHandle, _force: [f32; 2]) -> GoudResult<()> {
            Ok(())
        }

        fn apply_impulse(&mut self, _handle: BodyHandle, _impulse: [f32; 2]) -> GoudResult<()> {
            Ok(())
        }

        fn body_gravity_scale(&self, _handle: BodyHandle) -> GoudResult<f32> {
            Ok(1.0)
        }

        fn set_body_gravity_scale(&mut self, _handle: BodyHandle, _scale: f32) -> GoudResult<()> {
            Ok(())
        }

        fn create_collider(
            &mut self,
            _body: BodyHandle,
            _desc: &ColliderDesc,
        ) -> GoudResult<ColliderHandle> {
            Ok(ColliderHandle(1))
        }

        fn destroy_collider(&mut self, _handle: ColliderHandle) {}

        fn collider_friction(&self, _handle: ColliderHandle) -> GoudResult<f32> {
            Ok(0.5)
        }

        fn set_collider_friction(
            &mut self,
            _handle: ColliderHandle,
            _friction: f32,
        ) -> GoudResult<()> {
            Ok(())
        }

        fn collider_restitution(&self, _handle: ColliderHandle) -> GoudResult<f32> {
            Ok(0.0)
        }

        fn set_collider_restitution(
            &mut self,
            _handle: ColliderHandle,
            _restitution: f32,
        ) -> GoudResult<()> {
            Ok(())
        }

        fn raycast(&self, _origin: [f32; 2], _dir: [f32; 2], _max_dist: f32) -> Option<RaycastHit> {
            None
        }

        fn overlap_circle(&self, _center: [f32; 2], _radius: f32) -> Vec<BodyHandle> {
            Vec::new()
        }

        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            std::mem::take(&mut self.pending_events)
        }

        fn contact_pairs(&self) -> Vec<ContactPair> {
            Vec::new()
        }

        fn create_joint(&mut self, _desc: &JointDesc) -> GoudResult<JointHandle> {
            Ok(JointHandle(1))
        }

        fn destroy_joint(&mut self, _handle: JointHandle) {}

        fn debug_shapes(&self) -> Vec<DebugShape> {
            Vec::new()
        }

        fn physics_diagnostics(&self) -> crate::core::providers::diagnostics::PhysicsDiagnosticsV1 {
            Default::default()
        }
    }

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

    #[test]
    fn test_system_run_with_physics_world_produces_interpolation() {
        let provider = NullPhysicsProvider::new();
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap2D::default());
        world.insert_resource(PhysicsWorld::new());

        system.run(&mut world);

        // After running with a PhysicsWorld, a PhysicsInterpolation resource
        // should be inserted.
        assert!(world.contains_resource::<PhysicsInterpolation>());
    }

    #[test]
    fn test_system_fallback_without_physics_world() {
        let provider = NullPhysicsProvider::new();
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap2D::default());

        // No PhysicsWorld -- should still run without panic, using fallback.
        system.run(&mut world);

        // No PhysicsInterpolation since we used fallback path.
        assert!(!world.contains_resource::<PhysicsInterpolation>());
    }

    #[test]
    fn test_system_forwards_collision_events_into_events_resource() {
        let expected_event = CollisionEvent {
            body_a: BodyHandle(11),
            body_b: BodyHandle(22),
            kind: CollisionEventKind::Enter,
        };
        let provider = TestCollisionEventProvider::with_events(vec![expected_event.clone()]);
        let mut system = PhysicsStepSystem2D::new(Box::new(provider));
        let mut world = World::new();
        world.insert_resource(PhysicsHandleMap2D::default());
        world.insert_resource(Events::<CollisionEvent>::new());

        system.run(&mut world);

        let events = world
            .get_resource_mut::<Events<CollisionEvent>>()
            .expect("Events<CollisionEvent> resource should exist");
        assert_eq!(
            events.len(),
            1,
            "physics step should forward drained provider collision events into Events<CollisionEvent>"
        );

        events.update();
        let collected: Vec<_> = events.reader().read().cloned().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].body_a, expected_event.body_a);
        assert_eq!(collected[0].body_b, expected_event.body_b);
        assert_eq!(collected[0].kind, expected_event.kind);
    }
}
