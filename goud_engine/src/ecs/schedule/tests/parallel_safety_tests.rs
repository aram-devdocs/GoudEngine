//! Tests verifying that conflicting systems are correctly serialized by the
//! scheduler and that all systems execute producing correct results.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::schedule::*;
use crate::ecs::system::System;
use crate::ecs::Component;
use crate::ecs::World;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::sync::Arc;

// ====================================================================
// Test components
// ====================================================================

#[derive(Debug, Clone, Copy)]
struct Counter {
    value: u32,
}
impl Component for Counter {}

#[derive(Debug, Clone, Copy)]
struct Health {
    hp: u32,
}
impl Component for Health {}

// ====================================================================
// test_write_write_conflict_serialized
// ====================================================================

/// Two systems both write to Counter; they must both execute (serialized)
/// and each mutation must be applied.
#[test]
fn test_write_write_conflict_serialized() {
    use crate::ecs::entity::Entity;

    struct IncrementByOne {
        entity: Entity,
    }
    impl System for IncrementByOne {
        fn name(&self) -> &'static str {
            "IncrementByOne"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value += 1;
            }
        }
    }

    struct IncrementByTen {
        entity: Entity,
    }
    impl System for IncrementByTen {
        fn name(&self) -> &'static str {
            "IncrementByTen"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value += 10;
            }
        }
    }

    let mut world = World::new();
    let entity = world.spawn().insert(Counter { value: 0 }).id();

    let mut stage = ParallelSystemStage::new("test");
    stage.add_system(IncrementByOne { entity });
    stage.add_system(IncrementByTen { entity });
    stage.run_parallel(&mut world);

    let counter = world.get::<Counter>(entity).expect("Counter must exist");
    assert_eq!(
        counter.value, 11,
        "Both writes must be applied: 0 + 1 + 10 = 11"
    );
}

// ====================================================================
// test_read_write_conflict_serialized
// ====================================================================

/// One writer and one reader conflict on Counter.
/// The reader must observe a consistent value (either the original or the
/// written one) — never a torn/partial state.
#[test]
fn test_read_write_conflict_serialized() {
    use crate::ecs::entity::Entity;

    struct SetCounter {
        entity: Entity,
    }
    impl System for SetCounter {
        fn name(&self) -> &'static str {
            "SetCounter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value = 42;
            }
        }
    }

    struct ReadCounter {
        entity: Entity,
        observed: Arc<AtomicU32>,
    }
    impl System for ReadCounter {
        fn name(&self) -> &'static str {
            "ReadCounter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get::<Counter>(self.entity) {
                self.observed.store(c.value, AtomicOrdering::SeqCst);
            }
        }
    }

    let mut world = World::new();
    let entity = world.spawn().insert(Counter { value: 0 }).id();
    let observed = Arc::new(AtomicU32::new(u32::MAX));

    let mut stage = ParallelSystemStage::new("test");
    stage.add_system(SetCounter { entity });
    stage.add_system(ReadCounter {
        entity,
        observed: observed.clone(),
    });
    stage.run_parallel(&mut world);

    let seen = observed.load(AtomicOrdering::SeqCst);
    assert!(
        seen == 0 || seen == 42,
        "Reader must observe either original (0) or written (42) value, got {seen}"
    );
}

// ====================================================================
// test_many_conflicting_writers_all_execute
// ====================================================================

/// Ten systems all write to Counter (all conflict with each other).
/// Every system must execute; the final value must be 10.
#[test]
fn test_many_conflicting_writers_all_execute() {
    use crate::ecs::entity::Entity;

    struct IncrWriter {
        entity: Entity,
        exec_counter: Arc<AtomicU32>,
    }
    impl System for IncrWriter {
        fn name(&self) -> &'static str {
            "IncrWriter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            self.exec_counter.fetch_add(1, AtomicOrdering::SeqCst);
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value += 1;
            }
        }
    }

    let mut world = World::new();
    let entity = world.spawn().insert(Counter { value: 0 }).id();
    let exec_counter = Arc::new(AtomicU32::new(0));

    let mut stage = ParallelSystemStage::new("test");
    for _ in 0..10 {
        stage.add_system(IncrWriter {
            entity,
            exec_counter: exec_counter.clone(),
        });
    }
    stage.run_parallel(&mut world);

    assert_eq!(
        exec_counter.load(AtomicOrdering::SeqCst),
        10,
        "All 10 systems must execute"
    );
    let counter = world.get::<Counter>(entity).expect("Counter must exist");
    assert_eq!(counter.value, 10, "All 10 increments must be applied");
}

// ====================================================================
// test_mixed_conflict_free_and_conflicting
// ====================================================================

/// Counter-writers conflict among themselves; Health-writers conflict
/// among themselves; but Counter-writers and Health-writers do NOT conflict.
/// All systems must execute and both component values must be correct.
#[test]
fn test_mixed_conflict_free_and_conflicting() {
    use crate::ecs::entity::Entity;

    struct CounterWriter {
        entity: Entity,
        amount: u32,
    }
    impl System for CounterWriter {
        fn name(&self) -> &'static str {
            "CounterWriter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value += self.amount;
            }
        }
    }

    struct HealthWriter {
        entity: Entity,
        amount: u32,
    }
    impl System for HealthWriter {
        fn name(&self) -> &'static str {
            "HealthWriter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Health>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(h) = world.get_mut::<Health>(self.entity) {
                h.hp += self.amount;
            }
        }
    }

    let mut world = World::new();
    let entity = world
        .spawn()
        .insert(Counter { value: 0 })
        .insert(Health { hp: 0 })
        .id();

    let mut stage = ParallelSystemStage::new("test");
    // Three counter writers: total +6
    stage.add_system(CounterWriter { entity, amount: 1 });
    stage.add_system(CounterWriter { entity, amount: 2 });
    stage.add_system(CounterWriter { entity, amount: 3 });
    // Two health writers: total +30
    stage.add_system(HealthWriter { entity, amount: 10 });
    stage.add_system(HealthWriter { entity, amount: 20 });
    stage.run_parallel(&mut world);

    let counter = world.get::<Counter>(entity).expect("Counter must exist");
    assert_eq!(
        counter.value, 6,
        "All counter writes must be applied: 1+2+3=6"
    );

    let health = world.get::<Health>(entity).expect("Health must exist");
    assert_eq!(health.hp, 30, "All health writes must be applied: 10+20=30");
}

// ====================================================================
// test_ordered_writer_reader
// ====================================================================

/// Writer is ordered before reader via `set_before`.
/// Reader must always observe the written value (99).
#[test]
fn test_ordered_writer_reader() {
    use crate::ecs::entity::Entity;

    struct Writer {
        entity: Entity,
    }
    impl System for Writer {
        fn name(&self) -> &'static str {
            "Writer"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value = 99;
            }
        }
    }

    struct Reader {
        entity: Entity,
        observed: Arc<AtomicU32>,
    }
    impl System for Reader {
        fn name(&self) -> &'static str {
            "Reader"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get::<Counter>(self.entity) {
                self.observed.store(c.value, AtomicOrdering::SeqCst);
            }
        }
    }

    let mut world = World::new();
    let entity = world.spawn().insert(Counter { value: 0 }).id();
    let observed = Arc::new(AtomicU32::new(u32::MAX));

    let mut stage = ParallelSystemStage::new("test");
    let writer_id = stage.add_system(Writer { entity });
    let reader_id = stage.add_system(Reader {
        entity,
        observed: observed.clone(),
    });
    // Writer runs before reader
    stage.set_before(writer_id, reader_id);
    stage.run_parallel(&mut world);

    assert_eq!(
        observed.load(AtomicOrdering::SeqCst),
        99,
        "Reader must see the value written by the writer (99)"
    );
}

// ====================================================================
// test_stress_many_systems_shared_components
// ====================================================================

/// 50 systems with varied access: some read Counter, some write Counter,
/// some read Health, some write Health. All must execute.
#[test]
fn test_stress_many_systems_shared_components() {
    use crate::ecs::entity::Entity;

    struct CounterReader {
        entity: Entity,
        exec: Arc<AtomicU32>,
    }
    impl System for CounterReader {
        fn name(&self) -> &'static str {
            "CounterReader"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            let _ = world.get::<Counter>(self.entity);
            self.exec.fetch_add(1, AtomicOrdering::SeqCst);
        }
    }

    struct CounterWriterStress {
        entity: Entity,
        exec: Arc<AtomicU32>,
    }
    impl System for CounterWriterStress {
        fn name(&self) -> &'static str {
            "CounterWriterStress"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Counter>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(c) = world.get_mut::<Counter>(self.entity) {
                c.value = c.value.wrapping_add(1);
            }
            self.exec.fetch_add(1, AtomicOrdering::SeqCst);
        }
    }

    struct HealthReader {
        entity: Entity,
        exec: Arc<AtomicU32>,
    }
    impl System for HealthReader {
        fn name(&self) -> &'static str {
            "HealthReader"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Health>());
            a
        }
        fn run(&mut self, world: &mut World) {
            let _ = world.get::<Health>(self.entity);
            self.exec.fetch_add(1, AtomicOrdering::SeqCst);
        }
    }

    struct HealthWriterStress {
        entity: Entity,
        exec: Arc<AtomicU32>,
    }
    impl System for HealthWriterStress {
        fn name(&self) -> &'static str {
            "HealthWriterStress"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Health>());
            a
        }
        fn run(&mut self, world: &mut World) {
            if let Some(h) = world.get_mut::<Health>(self.entity) {
                h.hp = h.hp.wrapping_add(1);
            }
            self.exec.fetch_add(1, AtomicOrdering::SeqCst);
        }
    }

    let mut world = World::new();
    let entity = world
        .spawn()
        .insert(Counter { value: 0 })
        .insert(Health { hp: 0 })
        .id();
    let exec = Arc::new(AtomicU32::new(0));

    let mut stage = ParallelSystemStage::new("test");

    // 15 counter readers
    for _ in 0..15 {
        stage.add_system(CounterReader {
            entity,
            exec: exec.clone(),
        });
    }
    // 15 counter writers
    for _ in 0..15 {
        stage.add_system(CounterWriterStress {
            entity,
            exec: exec.clone(),
        });
    }
    // 10 health readers
    for _ in 0..10 {
        stage.add_system(HealthReader {
            entity,
            exec: exec.clone(),
        });
    }
    // 10 health writers
    for _ in 0..10 {
        stage.add_system(HealthWriterStress {
            entity,
            exec: exec.clone(),
        });
    }

    stage.run_parallel(&mut world);

    assert_eq!(
        exec.load(AtomicOrdering::SeqCst),
        50,
        "All 50 systems must execute"
    );
}
