//! Integration tests combining System, BoxedSystem, and World interaction.

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

#[test]
fn test_system_modifies_world() {
    struct SpawnSystem;

    impl System for SpawnSystem {
        fn name(&self) -> &'static str {
            "SpawnSystem"
        }

        fn run(&mut self, world: &mut World) {
            world.spawn_empty();
        }
    }

    let mut world = World::new();
    let mut system = SpawnSystem;

    assert_eq!(world.entity_count(), 0);
    system.run(&mut world);
    assert_eq!(world.entity_count(), 1);
    system.run(&mut world);
    assert_eq!(world.entity_count(), 2);
}

#[test]
fn test_system_adds_components() {
    struct AddPositionSystem {
        entities: Vec<crate::ecs::Entity>,
    }

    impl System for AddPositionSystem {
        fn name(&self) -> &'static str {
            "AddPositionSystem"
        }

        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_write(ComponentId::of::<Position>());
            access
        }

        fn run(&mut self, world: &mut World) {
            for &entity in &self.entities {
                world.insert(entity, Position { x: 0.0, y: 0.0 });
            }
        }
    }

    let mut world = World::new();
    let e1 = world.spawn_empty();
    let e2 = world.spawn_empty();

    let mut system = AddPositionSystem {
        entities: vec![e1, e2],
    };
    system.run(&mut world);

    assert!(world.has::<Position>(e1));
    assert!(world.has::<Position>(e2));
}

#[test]
fn test_conditional_system_execution() {
    struct CountingSystem {
        count: u32,
        max_runs: u32,
    }

    impl System for CountingSystem {
        fn name(&self) -> &'static str {
            "CountingSystem"
        }

        fn should_run(&self, _world: &World) -> bool {
            self.count < self.max_runs
        }

        fn run(&mut self, _world: &mut World) {
            self.count += 1;
        }
    }

    let mut world = World::new();
    let mut system = CountingSystem {
        count: 0,
        max_runs: 3,
    };

    // Run only if should_run returns true
    for _ in 0..5 {
        if system.should_run(&world) {
            system.run(&mut world);
        }
    }

    assert_eq!(system.count, 3);
}

#[test]
fn test_boxed_system_pipeline() {
    struct IncrementCounter {
        counter: *mut u32,
    }
    // SAFETY: test-only raw pointer used in single-threaded context
    unsafe impl Send for IncrementCounter {}

    impl System for IncrementCounter {
        fn name(&self) -> &'static str {
            "IncrementCounter"
        }
        fn run(&mut self, _: &mut World) {
            // SAFETY: pointer is valid for the duration of this test
            unsafe {
                *self.counter += 1;
            }
        }
    }

    let mut counter: u32 = 0;
    let counter_ptr = &mut counter as *mut u32;

    let mut systems: Vec<BoxedSystem> = vec![
        BoxedSystem::new(IncrementCounter {
            counter: counter_ptr,
        }),
        BoxedSystem::new(IncrementCounter {
            counter: counter_ptr,
        }),
        BoxedSystem::new(IncrementCounter {
            counter: counter_ptr,
        }),
    ];

    let mut world = World::new();

    for system in &mut systems {
        system.run(&mut world);
    }

    assert_eq!(counter, 3);
}

#[test]
fn test_system_access_conflict_detection() {
    struct PositionWriter;
    impl System for PositionWriter {
        fn name(&self) -> &'static str {
            "PositionWriter"
        }
        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_write(ComponentId::of::<Position>());
            access
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct PositionReader;
    impl System for PositionReader {
        fn name(&self) -> &'static str {
            "PositionReader"
        }
        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_read(ComponentId::of::<Position>());
            access
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct VelocityReader;
    impl System for VelocityReader {
        fn name(&self) -> &'static str {
            "VelocityReader"
        }
        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_read(ComponentId::of::<Velocity>());
            access
        }
        fn run(&mut self, _: &mut World) {}
    }

    let writer = BoxedSystem::new(PositionWriter);
    let reader = BoxedSystem::new(PositionReader);
    let vel_reader = BoxedSystem::new(VelocityReader);

    // Writer conflicts with reader of same component
    assert!(writer.conflicts_with(&reader));
    assert!(reader.conflicts_with(&writer));

    // Writer doesn't conflict with reader of different component
    assert!(!writer.conflicts_with(&vel_reader));

    // Two readers don't conflict
    assert!(!reader.conflicts_with(&vel_reader));
}
