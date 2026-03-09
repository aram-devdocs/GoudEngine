//! Physics plugins for 2D and 3D ECS integration.
//!
//! These plugins register the necessary resources for physics synchronization.
//! The physics step systems must be added separately because they require
//! ownership of a [`PhysicsProvider`] instance, and [`Plugin::build`] takes
//! `&self` (preventing move semantics).
//!
//! # Usage
//!
//! ```rust,ignore
//! use goud_engine::ecs::app::{App, PhysicsPlugin2D};
//! use goud_engine::ecs::schedule::CoreStage;
//! use goud_engine::ecs::systems::PhysicsStepSystem2D;
//! use goud_engine::core::providers::impls::NullPhysicsProvider;
//!
//! let mut app = App::new();
//! app.add_plugin(PhysicsPlugin2D);
//! app.add_system(
//!     CoreStage::Update,
//!     PhysicsStepSystem2D::new(Box::new(NullPhysicsProvider::new())),
//! );
//! ```

use super::plugin::Plugin;
use super::App;
use crate::core::event::Events;
use crate::core::providers::types::CollisionEvent;
use crate::ecs::systems::physics_sync_2d::PhysicsHandleMap2D;
use crate::ecs::systems::physics_sync_3d::PhysicsHandleMap3D;

/// Plugin that registers 2D physics resources.
///
/// Inserts a default [`PhysicsHandleMap2D`] resource into the world.
/// After adding this plugin, add a [`PhysicsStepSystem2D`] to the desired
/// stage (typically [`CoreStage::Update`]).
///
/// [`PhysicsStepSystem2D`]: crate::ecs::systems::PhysicsStepSystem2D
/// [`CoreStage::Update`]: crate::ecs::schedule::CoreStage::Update
#[derive(Debug, Default, Clone)]
pub struct PhysicsPlugin2D;

impl Plugin for PhysicsPlugin2D {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsHandleMap2D::default());
        app.insert_resource(Events::<CollisionEvent>::new());
    }

    fn name(&self) -> &'static str {
        "PhysicsPlugin2D"
    }
}

/// Plugin that registers 3D physics resources.
///
/// Inserts a default [`PhysicsHandleMap3D`] resource into the world.
/// After adding this plugin, add a [`PhysicsStepSystem3D`] to the desired
/// stage (typically [`CoreStage::Update`]).
///
/// [`PhysicsStepSystem3D`]: crate::ecs::systems::PhysicsStepSystem3D
/// [`CoreStage::Update`]: crate::ecs::schedule::CoreStage::Update
#[derive(Debug, Default, Clone)]
pub struct PhysicsPlugin3D;

impl Plugin for PhysicsPlugin3D {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsHandleMap3D::default());
    }

    fn name(&self) -> &'static str {
        "PhysicsPlugin3D"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Events;
    use crate::core::providers::types::CollisionEvent;

    #[test]
    fn test_physics_plugin_2d_build() {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin2D);
        assert!(app.world().get_resource::<PhysicsHandleMap2D>().is_some());
        assert!(app
            .world()
            .get_resource::<Events<CollisionEvent>>()
            .is_some());
    }

    #[test]
    fn test_physics_plugin_2d_build_registers_collision_events_resource() {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin2D);
        assert!(
            app.world().get_resource::<Events<CollisionEvent>>().is_some(),
            "PhysicsPlugin2D should register Events<CollisionEvent> so collision events are readable"
        );
    }

    #[test]
    fn test_physics_plugin_2d_name() {
        let plugin = PhysicsPlugin2D;
        assert_eq!(plugin.name(), "PhysicsPlugin2D");
    }

    #[test]
    fn test_physics_plugin_3d_build() {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin3D);
        assert!(app.world().get_resource::<PhysicsHandleMap3D>().is_some());
    }

    #[test]
    fn test_physics_plugin_3d_name() {
        let plugin = PhysicsPlugin3D;
        assert_eq!(plugin.name(), "PhysicsPlugin3D");
    }

    #[test]
    fn test_both_plugins_can_coexist() {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin2D);
        app.add_plugin(PhysicsPlugin3D);
        assert!(app.world().get_resource::<PhysicsHandleMap2D>().is_some());
        assert!(app.world().get_resource::<PhysicsHandleMap3D>().is_some());
        assert!(app
            .world()
            .get_resource::<Events<CollisionEvent>>()
            .is_some());
    }

    #[test]
    fn test_duplicate_plugin_ignored() {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin2D);
        // Second add should be silently ignored (no panic).
        app.add_plugin(PhysicsPlugin2D);
    }
}
