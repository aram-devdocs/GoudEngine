use super::*;
use crate::core::math::{Vec2, Vec3};
use crate::ecs::components::global_transform::GlobalTransform;
use crate::ecs::components::global_transform2d::GlobalTransform2D;
use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::components::transform::Transform;
use crate::ecs::components::transform2d::Transform2D;
use crate::ecs::system::System;

// =========================================================================
// App basics
// =========================================================================

#[test]
fn test_app_new_has_all_stages() {
    let app = App::new();
    assert_eq!(app.stages.len(), CoreStage::count());
}

#[test]
fn test_app_default_is_same_as_new() {
    let app = App::default();
    assert_eq!(app.stages.len(), CoreStage::count());
}

#[test]
fn test_app_debug_format() {
    let app = App::new();
    let debug = format!("{:?}", app);
    assert!(debug.contains("App"));
    assert!(debug.contains("stage_count"));
}

#[test]
fn test_add_system_to_update() {
    struct Counter {
        count: u32,
    }
    impl System for Counter {
        fn name(&self) -> &'static str {
            "Counter"
        }
        fn run(&mut self, _world: &mut World) {
            self.count += 1;
        }
    }

    let mut app = App::new();
    app.add_system(CoreStage::Update, Counter { count: 0 });

    let update_stage = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    assert_eq!(update_stage.1.system_count(), 1);
}

#[test]
fn test_run_once_executes_systems() {
    struct Marker;
    impl crate::ecs::Component for Marker {}

    struct SpawnSystem;
    impl System for SpawnSystem {
        fn name(&self) -> &'static str {
            "SpawnSystem"
        }
        fn run(&mut self, world: &mut World) {
            let e = world.spawn_empty();
            world.insert(e, Marker);
        }
    }

    let mut app = App::new();
    app.add_system(CoreStage::Update, SpawnSystem);
    app.run_once();

    assert_eq!(app.world().entity_count(), 1);
}

#[test]
fn test_update_is_equivalent_to_run_once() {
    struct Marker;
    impl crate::ecs::Component for Marker {}

    struct SpawnSystem;
    impl System for SpawnSystem {
        fn name(&self) -> &'static str {
            "SpawnSystem"
        }
        fn run(&mut self, world: &mut World) {
            let e = world.spawn_empty();
            world.insert(e, Marker);
        }
    }

    let mut app = App::new();
    app.add_system(CoreStage::Update, SpawnSystem);
    app.update();

    assert_eq!(app.world().entity_count(), 1);
}

#[test]
fn test_insert_resource() {
    struct GameTime {
        elapsed: f32,
    }

    let mut app = App::new();
    app.insert_resource(GameTime { elapsed: 0.0 });

    let time = app.world().get_resource::<GameTime>().unwrap();
    assert!((time.elapsed - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_world_mut_access() {
    let mut app = App::new();
    let entity = app.world_mut().spawn_empty();
    assert_eq!(app.world().entity_count(), 1);
    assert!(app.world().is_alive(entity));
}

// =========================================================================
// Plugin tests
// =========================================================================

#[test]
fn test_add_plugin() {
    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn build(&self, app: &mut App) {
            struct Noop;
            impl System for Noop {
                fn name(&self) -> &'static str {
                    "Noop"
                }
                fn run(&mut self, _world: &mut World) {}
            }
            app.add_system(CoreStage::Update, Noop);
        }
    }

    let mut app = App::new();
    app.add_plugin(TestPlugin);

    assert!(app
        .initialized_plugins
        .contains(&TypeId::of::<TestPlugin>()));
}

#[test]
fn test_duplicate_plugin_ignored() {
    struct CountPlugin;
    impl Plugin for CountPlugin {
        fn build(&self, app: &mut App) {
            struct Noop;
            impl System for Noop {
                fn name(&self) -> &'static str {
                    "Noop"
                }
                fn run(&mut self, _world: &mut World) {}
            }
            app.add_system(CoreStage::Update, Noop);
        }
    }

    let mut app = App::new();
    app.add_plugin(CountPlugin);
    app.add_plugin(CountPlugin); // Should be silently ignored

    // Only 1 system should exist, not 2
    let update = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    assert_eq!(update.1.system_count(), 1);
}

#[test]
#[should_panic(expected = "unmet dependency")]
fn test_plugin_missing_dependency_panics() {
    struct DepPlugin;
    impl Plugin for DepPlugin {
        fn build(&self, _app: &mut App) {}
        fn dependencies(&self) -> Vec<TypeId> {
            vec![TypeId::of::<MissingPlugin>()]
        }
    }

    struct MissingPlugin;
    impl Plugin for MissingPlugin {
        fn build(&self, _app: &mut App) {}
    }

    let mut app = App::new();
    app.add_plugin(DepPlugin); // Should panic
}

// =========================================================================
// TransformPropagationPlugin integration
// =========================================================================

#[test]
fn test_transform_propagation_plugin_adds_system() {
    let mut app = App::new();
    app.add_plugin(TransformPropagationPlugin);

    let post_update = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::PostUpdate)
        .unwrap();
    assert_eq!(
        post_update.1.system_count(),
        1,
        "TransformPropagationPlugin should add 1 system to PostUpdate"
    );
}

#[test]
fn test_plugin_propagates_3d_on_run() {
    let mut app = App::new();
    app.add_plugin(TransformPropagationPlugin);

    let parent = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
    app.world_mut().insert(parent, GlobalTransform::IDENTITY);

    let child = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
    app.world_mut().insert(child, GlobalTransform::IDENTITY);
    app.world_mut().insert(child, Parent::new(parent));

    let mut children = Children::new();
    children.push(child);
    app.world_mut().insert(parent, children);

    app.run_once();

    let child_global = app.world().get::<GlobalTransform>(child).unwrap();
    assert!(
        (child_global.translation().x - 15.0).abs() < 0.001,
        "Child 3D global x should be 15.0 after plugin run, got {}",
        child_global.translation().x
    );
}

#[test]
fn test_plugin_propagates_2d_on_run() {
    let mut app = App::new();
    app.add_plugin(TransformPropagationPlugin);

    let parent = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(parent, Transform2D::from_position(Vec2::new(100.0, 0.0)));
    app.world_mut().insert(parent, GlobalTransform2D::IDENTITY);

    let child = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(child, Transform2D::from_position(Vec2::new(50.0, 0.0)));
    app.world_mut().insert(child, GlobalTransform2D::IDENTITY);
    app.world_mut().insert(child, Parent::new(parent));

    let mut children = Children::new();
    children.push(child);
    app.world_mut().insert(parent, children);

    app.run_once();

    let child_global = app.world().get::<GlobalTransform2D>(child).unwrap();
    assert!(
        (child_global.translation().x - 150.0).abs() < 0.001,
        "Child 2D global x should be 150.0 after plugin run, got {}",
        child_global.translation().x
    );
}

#[test]
fn test_transform_repropagation_across_multiple_updates() {
    let mut app = App::new();
    app.add_plugin(TransformPropagationPlugin);

    // Create parent at (10, 0, 0)
    let parent = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(parent, Transform::from_position(Vec3::new(10.0, 0.0, 0.0)));
    app.world_mut().insert(parent, GlobalTransform::IDENTITY);

    // Create child at local (5, 0, 0)
    let child = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(child, Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
    app.world_mut().insert(child, GlobalTransform::IDENTITY);
    app.world_mut().insert(child, Parent::new(parent));

    // Set up parent's children
    let mut children = Children::new();
    children.push(child);
    app.world_mut().insert(parent, children);

    // First update: verify initial propagation
    app.update();

    let child_global = app.world().get::<GlobalTransform>(child).unwrap();
    let child_pos = child_global.translation();
    assert!(
        (child_pos.x - 15.0).abs() < 0.001,
        "Child global x should be 15.0 after first update, got {}",
        child_pos.x
    );

    // Modify parent's transform
    let parent_transform = Transform::from_position(Vec3::new(20.0, 0.0, 0.0));
    app.world_mut().insert(parent, parent_transform);

    // Second update: verify re-propagation
    app.update();

    let child_global = app.world().get::<GlobalTransform>(child).unwrap();
    let child_pos = child_global.translation();
    assert!(
        (child_pos.x - 25.0).abs() < 0.001,
        "Child global x should be 25.0 after parent modification and update, got {}",
        child_pos.x
    );
}
