use super::*;
use crate::core::math::Vec2;
use crate::ecs::components::global_transform2d::GlobalTransform2D;
use crate::ecs::components::hierarchy::{Children, Parent};
use crate::ecs::components::transform2d::Transform2D;
use crate::ecs::system::System;

// =========================================================================
// DefaultPlugins & add_plugin_group
// =========================================================================

#[test]
fn test_default_plugins_adds_transform_propagation() {
    let mut app = App::new();
    app.add_plugin_group(builtin_plugins::DefaultPlugins);
    let post = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::PostUpdate)
        .unwrap();
    assert_eq!(post.1.system_count(), 1);
}

#[test]
fn test_new_with_defaults_has_transform_system() {
    let app = App::new_with_defaults();
    let post = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::PostUpdate)
        .unwrap();
    assert_eq!(post.1.system_count(), 1);
}

#[test]
fn test_new_with_defaults_propagates_transforms() {
    let mut app = App::new_with_defaults();
    let parent = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(parent, Transform2D::from_position(Vec2::new(10.0, 0.0)));
    app.world_mut().insert(parent, GlobalTransform2D::IDENTITY);
    let child = app.world_mut().spawn_empty();
    app.world_mut()
        .insert(child, Transform2D::from_position(Vec2::new(5.0, 0.0)));
    app.world_mut().insert(child, GlobalTransform2D::IDENTITY);
    app.world_mut().insert(child, Parent::new(parent));
    let mut children = Children::new();
    children.push(child);
    app.world_mut().insert(parent, children);
    app.run_once();
    let g = app.world().get::<GlobalTransform2D>(child).unwrap();
    assert!((g.translation().x - 15.0).abs() < 0.001);
}

#[test]
fn test_default_plugins_idempotent() {
    let mut app = App::new();
    app.add_plugin_group(builtin_plugins::DefaultPlugins);
    app.add_plugin_group(builtin_plugins::DefaultPlugins);
    let post = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::PostUpdate)
        .unwrap();
    assert_eq!(post.1.system_count(), 1);
}

#[test]
fn test_minimal_app_has_no_default_systems() {
    let app = App::new();
    let total: usize = app.stages.iter().map(|(_, s)| s.system_count()).sum();
    assert_eq!(total, 0);
}

#[test]
fn test_add_plugin_group() {
    struct PluginX;
    impl Plugin for PluginX {
        fn build(&self, app: &mut App) {
            struct Noop;
            impl System for Noop {
                fn name(&self) -> &'static str {
                    "NoopX"
                }
                fn run(&mut self, _world: &mut World) {}
            }
            app.add_system(CoreStage::Update, Noop);
        }
    }
    struct TestGroup;
    impl plugin::PluginGroup for TestGroup {
        fn build(self, app: &mut App) {
            app.add_plugin(PluginX);
        }
    }
    let mut app = App::new();
    app.add_plugin_group(TestGroup);
    let upd = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    assert_eq!(upd.1.system_count(), 1);
}

#[test]
fn test_app_insert_non_send_resource() {
    use crate::ecs::resource::NonSendResource;
    use std::rc::Rc;
    struct GlContext {
        _id: Rc<u32>,
    }
    impl NonSendResource for GlContext {}
    let mut app = App::new();
    app.insert_non_send_resource(GlContext { _id: Rc::new(1) });
    assert!(app.world().contains_non_send_resource::<GlContext>());
}

// =========================================================================
// PluginGroup (legacy)
// =========================================================================

#[test]
fn test_plugin_group() {
    struct PluginA;
    impl Plugin for PluginA {
        fn build(&self, app: &mut App) {
            struct Noop;
            impl System for Noop {
                fn name(&self) -> &'static str {
                    "NoopA"
                }
                fn run(&mut self, _world: &mut World) {}
            }
            app.add_system(CoreStage::PreUpdate, Noop);
        }
    }

    struct PluginB;
    impl Plugin for PluginB {
        fn build(&self, app: &mut App) {
            struct Noop;
            impl System for Noop {
                fn name(&self) -> &'static str {
                    "NoopB"
                }
                fn run(&mut self, _world: &mut World) {}
            }
            app.add_system(CoreStage::Update, Noop);
        }
    }

    struct MyGroup;
    impl PluginGroup for MyGroup {
        fn build(self, app: &mut App) {
            app.add_plugin(PluginA);
            app.add_plugin(PluginB);
        }
    }

    let mut app = App::new();
    MyGroup.build(&mut app);

    let pre = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::PreUpdate)
        .unwrap();
    let upd = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    assert_eq!(pre.1.system_count(), 1);
    assert_eq!(upd.1.system_count(), 1);
}

// =========================================================================
// Named System Sets via App
// =========================================================================

#[test]
fn test_app_register_set() {
    let mut app = App::new();
    app.register_set(CoreStage::Update, "Physics");

    let stage = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    assert!(stage.1.get_set("Physics").is_some());
}

#[test]
fn test_app_add_system_to_set() {
    struct Noop;
    impl System for Noop {
        fn name(&self) -> &'static str {
            "Noop"
        }
        fn run(&mut self, _world: &mut World) {}
    }

    let mut app = App::new();
    app.register_set(CoreStage::Update, "Logic");
    app.add_system_to_set(CoreStage::Update, "Logic", Noop);

    let stage = app
        .stages
        .iter()
        .find(|(s, _)| *s == CoreStage::Update)
        .unwrap();
    let set = stage.1.get_set("Logic").unwrap();
    assert_eq!(set.len(), 1);
}

#[test]
fn test_app_configure_set() {
    use crate::ecs::schedule::named_system_sets::SetNameLabel;
    use crate::ecs::schedule::SystemSetConfig;

    struct Noop;
    impl System for Noop {
        fn name(&self) -> &'static str {
            "Noop"
        }
        fn run(&mut self, _world: &mut World) {}
    }

    let mut app = App::new();
    app.register_set(CoreStage::Update, "A");
    app.register_set(CoreStage::Update, "B");
    app.add_system_to_set(CoreStage::Update, "A", Noop);
    app.add_system_to_set(CoreStage::Update, "B", Noop);
    app.configure_set(
        CoreStage::Update,
        "A",
        SystemSetConfig::new().before(SetNameLabel("B")),
    );

    // Should run without panicking
    app.run_once();
}
