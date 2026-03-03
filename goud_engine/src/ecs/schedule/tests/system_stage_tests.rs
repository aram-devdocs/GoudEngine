//! Tests for SystemStage.

use crate::ecs::schedule::*;
use crate::ecs::system::{System, SystemId};
use crate::ecs::World;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

// Helper systems for tests
struct SimpleSystem {
    name: &'static str,
}

impl System for SimpleSystem {
    fn name(&self) -> &'static str {
        self.name
    }
    fn run(&mut self, _: &mut World) {}
}

struct CounterSystem {
    name: &'static str,
    run_count: Arc<AtomicU32>,
}

impl System for CounterSystem {
    fn name(&self) -> &'static str {
        self.name
    }
    fn run(&mut self, _: &mut World) {
        self.run_count.fetch_add(1, Ordering::SeqCst);
    }
}

struct SpawnSystem;

impl System for SpawnSystem {
    fn name(&self) -> &'static str {
        "SpawnSystem"
    }
    fn run(&mut self, world: &mut World) {
        world.spawn_empty();
    }
}

struct ConditionalSystem {
    should_run: bool,
    ran: Arc<AtomicBool>,
}

impl System for ConditionalSystem {
    fn name(&self) -> &'static str {
        "ConditionalSystem"
    }
    fn should_run(&self, _: &World) -> bool {
        self.should_run
    }
    fn run(&mut self, _: &mut World) {
        self.ran.store(true, Ordering::SeqCst);
    }
}

// ====================================================================
// Construction Tests
// ====================================================================

#[test]
fn test_new() {
    let stage = SystemStage::new("Update");
    assert_eq!(stage.name(), "Update");
    assert_eq!(stage.system_count(), 0);
    assert!(stage.is_empty());
    assert!(!stage.is_initialized());
}

#[test]
fn test_with_capacity() {
    let stage = SystemStage::with_capacity("Physics", 10);
    assert_eq!(stage.name(), "Physics");
    assert_eq!(stage.system_count(), 0);
}

#[test]
fn test_from_core() {
    let stage = SystemStage::from_core(CoreStage::Update);
    assert_eq!(stage.name(), "Update");
}

#[test]
fn test_default() {
    let stage = SystemStage::default();
    assert_eq!(stage.name(), "DefaultStage");
    assert!(stage.is_empty());
}

// ====================================================================
// System Management Tests
// ====================================================================

#[test]
fn test_add_system() {
    let mut stage = SystemStage::new("Update");
    let id = stage.add_system(SimpleSystem { name: "SystemA" });
    assert!(id.is_valid());
    assert_eq!(stage.system_count(), 1);
    assert!(!stage.is_empty());
}

#[test]
fn test_add_multiple_systems() {
    let mut stage = SystemStage::new("Update");
    let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
    let id2 = stage.add_system(SimpleSystem { name: "SystemB" });
    assert_ne!(id1, id2);
    assert_eq!(stage.system_count(), 2);
}

#[test]
fn test_remove_system() {
    let mut stage = SystemStage::new("Update");
    let id = stage.add_system(SimpleSystem { name: "SystemA" });
    assert!(stage.remove_system(id));
    assert_eq!(stage.system_count(), 0);
    assert!(!stage.remove_system(id));
}

#[test]
fn test_get_system() {
    let mut stage = SystemStage::new("Update");
    let id = stage.add_system(SimpleSystem { name: "SystemA" });
    assert!(stage.get_system(id).is_some());
    assert_eq!(stage.get_system(id).unwrap().name(), "SystemA");
    assert!(stage.get_system(SystemId::new()).is_none());
}

#[test]
fn test_contains_system() {
    let mut stage = SystemStage::new("Update");
    let id = stage.add_system(SimpleSystem { name: "SystemA" });
    assert!(stage.contains_system(id));
    assert!(!stage.contains_system(SystemId::new()));
}

#[test]
fn test_system_ids() {
    let mut stage = SystemStage::new("Update");
    let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
    let id2 = stage.add_system(SimpleSystem { name: "SystemB" });
    let ids: Vec<_> = stage.system_ids().collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_systems_iterator() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "SystemA" });
    stage.add_system(SimpleSystem { name: "SystemB" });
    let names: Vec<_> = stage.systems().map(|s| s.name()).collect();
    assert_eq!(names, vec!["SystemA", "SystemB"]);
}

#[test]
fn test_system_names() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "SystemA" });
    stage.add_system(SimpleSystem { name: "SystemB" });
    assert_eq!(stage.system_names(), vec!["SystemA", "SystemB"]);
}

#[test]
fn test_clear() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "SystemA" });
    stage.add_system(SimpleSystem { name: "SystemB" });
    assert_eq!(stage.system_count(), 2);
    stage.clear();
    assert_eq!(stage.system_count(), 0);
    assert!(stage.is_empty());
}

// ====================================================================
// Execution Tests
// ====================================================================

#[test]
fn test_run_empty_stage() {
    let mut stage = SystemStage::new("Update");
    let mut world = World::new();
    stage.run(&mut world);
    assert!(stage.is_initialized());
}

#[test]
fn test_run_single_system() {
    let counter = Arc::new(AtomicU32::new(0));
    let mut stage = SystemStage::new("Update");
    stage.add_system(CounterSystem {
        name: "Counter",
        run_count: counter.clone(),
    });
    let mut world = World::new();
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    stage.run(&mut world);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    stage.run(&mut world);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn test_run_multiple_systems() {
    let ca = Arc::new(AtomicU32::new(0));
    let cb = Arc::new(AtomicU32::new(0));
    let mut stage = SystemStage::new("Update");
    stage.add_system(CounterSystem {
        name: "A",
        run_count: ca.clone(),
    });
    stage.add_system(CounterSystem {
        name: "B",
        run_count: cb.clone(),
    });
    let mut world = World::new();
    stage.run(&mut world);
    assert_eq!(ca.load(Ordering::SeqCst), 1);
    assert_eq!(cb.load(Ordering::SeqCst), 1);
}

#[test]
fn test_run_system_modifies_world() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SpawnSystem);
    let mut world = World::new();
    assert_eq!(world.entity_count(), 0);
    stage.run(&mut world);
    assert_eq!(world.entity_count(), 1);
    stage.run(&mut world);
    assert_eq!(world.entity_count(), 2);
}

#[test]
fn test_run_respects_should_run() {
    let ran_yes = Arc::new(AtomicBool::new(false));
    let ran_no = Arc::new(AtomicBool::new(false));
    let mut stage = SystemStage::new("Update");
    stage.add_system(ConditionalSystem {
        should_run: true,
        ran: ran_yes.clone(),
    });
    stage.add_system(ConditionalSystem {
        should_run: false,
        ran: ran_no.clone(),
    });
    let mut world = World::new();
    stage.run(&mut world);
    assert!(ran_yes.load(Ordering::SeqCst));
    assert!(!ran_no.load(Ordering::SeqCst));
}

#[test]
fn test_run_single_system_by_id() {
    let ca = Arc::new(AtomicU32::new(0));
    let cb = Arc::new(AtomicU32::new(0));
    let mut stage = SystemStage::new("Update");
    let id_a = stage.add_system(CounterSystem {
        name: "A",
        run_count: ca.clone(),
    });
    stage.add_system(CounterSystem {
        name: "B",
        run_count: cb.clone(),
    });
    let mut world = World::new();
    let result = stage.run_system(id_a, &mut world);
    assert_eq!(result, Some(true));
    assert_eq!(ca.load(Ordering::SeqCst), 1);
    assert_eq!(cb.load(Ordering::SeqCst), 0);
}

#[test]
fn test_run_system_not_found() {
    let mut stage = SystemStage::new("Update");
    let mut world = World::new();
    assert_eq!(stage.run_system(SystemId::new(), &mut world), None);
}

#[test]
fn test_run_system_skipped() {
    let ran = Arc::new(AtomicBool::new(false));
    let mut stage = SystemStage::new("Update");
    let id = stage.add_system(ConditionalSystem {
        should_run: false,
        ran: ran.clone(),
    });
    let mut world = World::new();
    assert_eq!(stage.run_system(id, &mut world), Some(false));
    assert!(!ran.load(Ordering::SeqCst));
}

// ====================================================================
// Initialization and Trait Tests
// ====================================================================

#[test]
fn test_initialization_happens_on_first_run() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    let mut world = World::new();
    assert!(!stage.is_initialized());
    stage.run(&mut world);
    assert!(stage.is_initialized());
}

#[test]
fn test_initialization_only_once() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    let mut world = World::new();
    stage.run(&mut world);
    stage.run(&mut world);
    assert!(stage.is_initialized());
}

#[test]
fn test_reset_initialized() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    let mut world = World::new();
    stage.run(&mut world);
    stage.reset_initialized();
    assert!(!stage.is_initialized());
}

#[test]
fn test_clear_resets_initialized() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    let mut world = World::new();
    stage.run(&mut world);
    stage.clear();
    assert!(!stage.is_initialized());
}

#[test]
fn test_stage_trait() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    let stage_ref: &dyn Stage = &stage;
    assert_eq!(stage_ref.name(), "Update");
    assert_eq!(stage_ref.system_count(), 1);
    assert!(!stage_ref.is_empty());
}

#[test]
fn test_debug() {
    let mut stage = SystemStage::new("Update");
    stage.add_system(SimpleSystem { name: "A" });
    stage.add_system(SimpleSystem { name: "B" });
    let debug = format!("{:?}", stage);
    assert!(debug.contains("SystemStage"));
    assert!(debug.contains("Update"));
    assert!(debug.contains("A"));
    assert!(debug.contains("B"));
}

#[test]
fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<SystemStage>();
    assert_sync::<SystemStage>();
}

// ====================================================================
// Edge Cases
// ====================================================================

#[test]
fn test_add_many_systems() {
    let mut stage = SystemStage::new("Update");
    for i in 0..100 {
        struct NumberedSystem(usize);
        impl System for NumberedSystem {
            fn name(&self) -> &'static str {
                "NumberedSystem"
            }
            fn run(&mut self, _: &mut World) {}
        }
        stage.add_system(NumberedSystem(i));
    }
    assert_eq!(stage.system_count(), 100);
}

#[test]
fn test_systems_run_in_order() {
    let order = Arc::new(AtomicUsize::new(0));
    struct OrderedSystem {
        expected_order: usize,
        order: Arc<AtomicUsize>,
        success: Arc<AtomicBool>,
    }
    impl System for OrderedSystem {
        fn name(&self) -> &'static str {
            "OrderedSystem"
        }
        fn run(&mut self, _: &mut World) {
            let current = self.order.fetch_add(1, Ordering::SeqCst);
            if current == self.expected_order {
                self.success.store(true, Ordering::SeqCst);
            }
        }
    }
    let successes: Vec<_> = (0..5).map(|_| Arc::new(AtomicBool::new(false))).collect();
    let mut stage = SystemStage::new("Update");
    for (i, success) in successes.iter().enumerate() {
        stage.add_system(OrderedSystem {
            expected_order: i,
            order: order.clone(),
            success: success.clone(),
        });
    }
    let mut world = World::new();
    stage.run(&mut world);
    for success in &successes {
        assert!(success.load(Ordering::SeqCst));
    }
}

#[test]
fn test_boxed_stage() {
    let stage = SystemStage::new("Update");
    let _boxed: Box<dyn Stage> = Box::new(stage);
}

#[test]
fn test_multiple_stages() {
    let mut pre_update = SystemStage::from_core(CoreStage::PreUpdate);
    let mut update = SystemStage::from_core(CoreStage::Update);
    let mut post_update = SystemStage::from_core(CoreStage::PostUpdate);
    pre_update.add_system(SimpleSystem { name: "Input" });
    update.add_system(SimpleSystem { name: "Physics" });
    update.add_system(SimpleSystem { name: "AI" });
    post_update.add_system(SimpleSystem { name: "Cleanup" });
    assert_eq!(pre_update.system_count(), 1);
    assert_eq!(update.system_count(), 2);
    assert_eq!(post_update.system_count(), 1);
    let mut world = World::new();
    pre_update.run(&mut world);
    update.run(&mut world);
    post_update.run(&mut world);
}
