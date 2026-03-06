//! Tests for [`SystemMeta`].

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::system::SystemMeta;
use crate::ecs::Component;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[test]
fn test_new() {
    let meta = SystemMeta::new("TestSystem");
    assert_eq!(meta.name(), "TestSystem");
    assert!(meta.is_read_only());
}

#[test]
fn test_new_with_string() {
    let name = String::from("DynamicSystem");
    let meta = SystemMeta::new(name);
    assert_eq!(meta.name(), "DynamicSystem");
}

#[test]
fn test_set_name() {
    let mut meta = SystemMeta::new("OldName");
    meta.set_name("NewName");
    assert_eq!(meta.name(), "NewName");
}

#[test]
fn test_default() {
    let meta = SystemMeta::default();
    assert_eq!(meta.name(), "UnnamedSystem");
}

#[test]
fn test_component_access() {
    let mut meta = SystemMeta::new("Test");
    assert!(meta.component_access().is_read_only());

    meta.component_access_mut()
        .add_write(ComponentId::of::<Position>());
    assert!(!meta.is_read_only());
}

#[test]
fn test_set_component_access() {
    let mut meta = SystemMeta::new("Test");

    let mut access = Access::new();
    access.add_read(ComponentId::of::<Position>());
    meta.set_component_access(access);

    assert!(meta.component_access().is_read_only());
}

#[test]
fn test_conflicts_with_no_conflict() {
    let mut meta1 = SystemMeta::new("System1");
    let mut meta2 = SystemMeta::new("System2");

    meta1
        .component_access_mut()
        .add_read(ComponentId::of::<Position>());
    meta2
        .component_access_mut()
        .add_read(ComponentId::of::<Velocity>());

    assert!(!meta1.conflicts_with(&meta2));
}

#[test]
fn test_conflicts_with_read_read_same() {
    let mut meta1 = SystemMeta::new("System1");
    let mut meta2 = SystemMeta::new("System2");

    meta1
        .component_access_mut()
        .add_read(ComponentId::of::<Position>());
    meta2
        .component_access_mut()
        .add_read(ComponentId::of::<Position>());

    // Two reads don't conflict
    assert!(!meta1.conflicts_with(&meta2));
}

#[test]
fn test_conflicts_with_write_read() {
    let mut meta1 = SystemMeta::new("System1");
    let mut meta2 = SystemMeta::new("System2");

    meta1
        .component_access_mut()
        .add_write(ComponentId::of::<Position>());
    meta2
        .component_access_mut()
        .add_read(ComponentId::of::<Position>());

    assert!(meta1.conflicts_with(&meta2));
    assert!(meta2.conflicts_with(&meta1));
}

#[test]
fn test_conflicts_with_write_write() {
    let mut meta1 = SystemMeta::new("System1");
    let mut meta2 = SystemMeta::new("System2");

    meta1
        .component_access_mut()
        .add_write(ComponentId::of::<Position>());
    meta2
        .component_access_mut()
        .add_write(ComponentId::of::<Position>());

    assert!(meta1.conflicts_with(&meta2));
}

#[test]
fn test_is_read_only() {
    let mut meta = SystemMeta::new("Test");
    assert!(meta.is_read_only());

    meta.component_access_mut()
        .add_read(ComponentId::of::<Position>());
    assert!(meta.is_read_only());

    meta.component_access_mut()
        .add_write(ComponentId::of::<Velocity>());
    assert!(!meta.is_read_only());
}

#[test]
fn test_clone() {
    let mut meta = SystemMeta::new("Test");
    meta.component_access_mut()
        .add_write(ComponentId::of::<Position>());

    let cloned = meta.clone();
    assert_eq!(cloned.name(), "Test");
    assert!(!cloned.is_read_only());
}

#[test]
fn test_debug() {
    let meta = SystemMeta::new("TestSystem");
    let debug = format!("{:?}", meta);
    assert!(debug.contains("TestSystem"));
}
