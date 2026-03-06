//! Tests for the `IntoSystem` trait.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::system::{BoxedSystem, IntoSystem, System};
use crate::ecs::{Component, World};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

struct MySystem;

impl System for MySystem {
    fn name(&self) -> &'static str {
        "MySystem"
    }

    fn run(&mut self, _world: &mut World) {}
}

#[test]
fn test_system_into_boxed() {
    let boxed: BoxedSystem = MySystem.into_system();
    assert_eq!(boxed.name(), "MySystem");
}

#[test]
fn test_into_system_preserves_behavior() {
    struct CounterSystem {
        count: u32,
    }
    impl System for CounterSystem {
        fn name(&self) -> &'static str {
            "CounterSystem"
        }
        fn run(&mut self, _: &mut World) {
            self.count += 1;
        }
    }

    let mut boxed = CounterSystem { count: 0 }.into_system();
    let mut world = World::new();

    boxed.run(&mut world);
    boxed.run(&mut world);
    // System runs without panics
}

#[test]
fn test_into_system_preserves_access() {
    struct AccessSystem;
    impl System for AccessSystem {
        fn name(&self) -> &'static str {
            "AccessSystem"
        }
        fn component_access(&self) -> Access {
            let mut access = Access::new();
            access.add_write(ComponentId::of::<Position>());
            access
        }
        fn run(&mut self, _: &mut World) {}
    }

    let boxed: BoxedSystem = AccessSystem.into_system();
    assert!(!boxed.is_read_only());
    assert!(boxed
        .component_access()
        .writes()
        .contains(&ComponentId::of::<Position>()));
}

#[test]
fn test_multiple_into_system() {
    struct SystemA;
    impl System for SystemA {
        fn name(&self) -> &'static str {
            "A"
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct SystemB;
    impl System for SystemB {
        fn name(&self) -> &'static str {
            "B"
        }
        fn run(&mut self, _: &mut World) {}
    }

    let systems: Vec<BoxedSystem> = vec![SystemA.into_system(), SystemB.into_system()];

    assert_eq!(systems[0].name(), "A");
    assert_eq!(systems[1].name(), "B");
}
