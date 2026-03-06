//! Tests for [`BoxedSystem`].

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::system::{BoxedSystem, System};
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

struct SimpleSystem {
    run_count: u32,
}

impl System for SimpleSystem {
    fn name(&self) -> &'static str {
        "SimpleSystem"
    }

    fn run(&mut self, _world: &mut World) {
        self.run_count += 1;
    }
}

struct WriteSystem;

impl System for WriteSystem {
    fn name(&self) -> &'static str {
        "WriteSystem"
    }

    fn component_access(&self) -> Access {
        let mut access = Access::new();
        access.add_write(ComponentId::of::<Position>());
        access
    }

    fn run(&mut self, _world: &mut World) {}
}

struct ReadSystem;

impl System for ReadSystem {
    fn name(&self) -> &'static str {
        "ReadSystem"
    }

    fn component_access(&self) -> Access {
        let mut access = Access::new();
        access.add_read(ComponentId::of::<Position>());
        access
    }

    fn run(&mut self, _world: &mut World) {}
}

#[test]
fn test_new() {
    let boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
    assert_eq!(boxed.name(), "SimpleSystem");
    assert!(boxed.id().is_valid());
}

#[test]
fn test_unique_ids() {
    let boxed1 = BoxedSystem::new(SimpleSystem { run_count: 0 });
    let boxed2 = BoxedSystem::new(SimpleSystem { run_count: 0 });

    assert_ne!(boxed1.id(), boxed2.id());
}

#[test]
fn test_run() {
    let mut boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
    let mut world = World::new();

    boxed.run(&mut world);
    boxed.run(&mut world);
    // Can't directly check run_count, but we can verify no panics
}

#[test]
fn test_component_access() {
    let boxed = BoxedSystem::new(WriteSystem);
    let access = boxed.component_access();

    assert!(access.writes().contains(&ComponentId::of::<Position>()));
}

#[test]
fn test_is_read_only() {
    let write_system = BoxedSystem::new(WriteSystem);
    let read_system = BoxedSystem::new(ReadSystem);

    assert!(!write_system.is_read_only());
    assert!(read_system.is_read_only());
}

#[test]
fn test_conflicts_with() {
    let write = BoxedSystem::new(WriteSystem);
    let read = BoxedSystem::new(ReadSystem);

    // Write and Read of same component conflict
    assert!(write.conflicts_with(&read));
    assert!(read.conflicts_with(&write));
}

#[test]
fn test_no_conflict_different_components() {
    struct VelocityWriter;
    impl System for VelocityWriter {
        fn name(&self) -> &'static str {
            "VelocityWriter"
        }
        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_write(ComponentId::of::<Velocity>());
            access
        }
        fn run(&mut self, _: &mut World) {}
    }

    let pos_write = BoxedSystem::new(WriteSystem);
    let vel_write = BoxedSystem::new(VelocityWriter);

    assert!(!pos_write.conflicts_with(&vel_write));
}

#[test]
fn test_should_run() {
    struct AlwaysSkip;
    impl System for AlwaysSkip {
        fn name(&self) -> &'static str {
            "AlwaysSkip"
        }
        fn should_run(&self, _: &World) -> bool {
            false
        }
        fn run(&mut self, _: &mut World) {}
    }

    let world = World::new();
    let normal = BoxedSystem::new(SimpleSystem { run_count: 0 });
    let skip = BoxedSystem::new(AlwaysSkip);

    assert!(normal.should_run(&world));
    assert!(!skip.should_run(&world));
}

#[test]
fn test_initialize() {
    struct InitSystem {
        initialized: bool,
    }
    impl System for InitSystem {
        fn name(&self) -> &'static str {
            "InitSystem"
        }
        fn initialize(&mut self, _: &mut World) {
            self.initialized = true;
        }
        fn run(&mut self, _: &mut World) {}
    }

    let mut boxed = BoxedSystem::new(InitSystem { initialized: false });
    let mut world = World::new();

    boxed.initialize(&mut world);
    // Can verify no panics; internal state isn't accessible
}

#[test]
fn test_debug() {
    let boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
    let debug = format!("{:?}", boxed);

    assert!(debug.contains("BoxedSystem"));
    assert!(debug.contains("SimpleSystem"));
}

#[test]
fn test_collection_of_different_systems() {
    let systems: Vec<BoxedSystem> = vec![
        BoxedSystem::new(SimpleSystem { run_count: 0 }),
        BoxedSystem::new(WriteSystem),
        BoxedSystem::new(ReadSystem),
    ];

    assert_eq!(systems.len(), 3);
    assert_eq!(systems[0].name(), "SimpleSystem");
    assert_eq!(systems[1].name(), "WriteSystem");
    assert_eq!(systems[2].name(), "ReadSystem");
}

#[test]
fn test_run_multiple_systems() {
    let mut world = World::new();
    let mut systems: Vec<BoxedSystem> = vec![
        BoxedSystem::new(SimpleSystem { run_count: 0 }),
        BoxedSystem::new(WriteSystem),
    ];

    for system in &mut systems {
        system.run(&mut world);
    }
    // Verify no panics during execution
}
