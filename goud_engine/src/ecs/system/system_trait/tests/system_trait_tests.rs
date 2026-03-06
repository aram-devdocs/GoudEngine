//! Tests for the `System` trait.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::system::System;
use crate::ecs::{Component, World};

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

struct TestSystem {
    run_count: u32,
    initialized: bool,
}

impl TestSystem {
    fn new() -> Self {
        Self {
            run_count: 0,
            initialized: false,
        }
    }
}

impl System for TestSystem {
    fn name(&self) -> &'static str {
        "TestSystem"
    }

    fn initialize(&mut self, _world: &mut World) {
        self.initialized = true;
    }

    fn run(&mut self, _world: &mut World) {
        self.run_count += 1;
    }
}

struct AccessTrackingSystem;

impl System for AccessTrackingSystem {
    fn name(&self) -> &'static str {
        "AccessTrackingSystem"
    }

    fn component_access(&self) -> Access {
        let mut access = Access::new();
        access.add_write(ComponentId::of::<Position>());
        access.add_read(ComponentId::of::<Velocity>());
        access
    }

    fn run(&mut self, _world: &mut World) {}
}

struct ConditionalSystem {
    should_run: bool,
}

impl System for ConditionalSystem {
    fn name(&self) -> &'static str {
        "ConditionalSystem"
    }

    fn should_run(&self, _world: &World) -> bool {
        self.should_run
    }

    fn run(&mut self, _world: &mut World) {}
}

#[test]
fn test_system_name() {
    let system = TestSystem::new();
    assert_eq!(system.name(), "TestSystem");
}

#[test]
fn test_system_run() {
    let mut system = TestSystem::new();
    let mut world = World::new();

    assert_eq!(system.run_count, 0);
    system.run(&mut world);
    assert_eq!(system.run_count, 1);
    system.run(&mut world);
    assert_eq!(system.run_count, 2);
}

#[test]
fn test_system_initialize() {
    let mut system = TestSystem::new();
    let mut world = World::new();

    assert!(!system.initialized);
    system.initialize(&mut world);
    assert!(system.initialized);
}

#[test]
fn test_system_component_access_default() {
    let system = TestSystem::new();
    let access = system.component_access();
    assert!(access.is_read_only());
}

#[test]
fn test_system_component_access_custom() {
    let system = AccessTrackingSystem;
    let access = system.component_access();

    assert!(!access.is_read_only());
    assert!(access.writes().contains(&ComponentId::of::<Position>()));
}

#[test]
fn test_system_should_run_default() {
    let system = TestSystem::new();
    let world = World::new();

    assert!(system.should_run(&world));
}

#[test]
fn test_system_should_run_custom() {
    let world = World::new();

    let system_yes = ConditionalSystem { should_run: true };
    let system_no = ConditionalSystem { should_run: false };

    assert!(system_yes.should_run(&world));
    assert!(!system_no.should_run(&world));
}

#[test]
fn test_system_is_read_only() {
    let system1 = TestSystem::new();
    let system2 = AccessTrackingSystem;

    assert!(system1.is_read_only());
    assert!(!system2.is_read_only());
}

#[test]
fn test_system_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<TestSystem>();
    assert_send::<AccessTrackingSystem>();
}
