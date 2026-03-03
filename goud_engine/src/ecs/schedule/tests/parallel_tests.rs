//! Tests for ParallelSystemStage and related types.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::schedule::*;
use crate::ecs::system::{System, SystemId};
use crate::ecs::Component;
use crate::ecs::World;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::sync::Arc;

// Test components
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

// ====================================================================
// Construction and basic API
// ====================================================================

#[test]
fn test_new() {
    let stage = ParallelSystemStage::new("Test");
    assert_eq!(stage.name(), "Test");
    assert_eq!(stage.system_count(), 0);
    assert!(stage.is_empty());
}

#[test]
fn test_with_capacity() {
    let stage = ParallelSystemStage::with_capacity("Test", 10);
    assert_eq!(stage.name(), "Test");
}

#[test]
fn test_from_core() {
    let stage = ParallelSystemStage::from_core(CoreStage::Update);
    assert_eq!(stage.name(), "Update");
}

#[test]
fn test_default() {
    let stage = ParallelSystemStage::default();
    assert_eq!(stage.name(), "ParallelStage");
}

#[test]
fn test_with_config() {
    let cfg = ParallelExecutionConfig::with_max_threads(4);
    let stage = ParallelSystemStage::with_config("Test", cfg);
    assert_eq!(stage.config().max_threads, 4);
}

#[test]
fn test_config_default() {
    let cfg = ParallelExecutionConfig::default();
    assert_eq!(cfg.max_threads, 0);
    assert!(cfg.auto_rebuild);
    assert!(cfg.respect_ordering);
}

#[test]
fn test_config_ignore_ordering() {
    let cfg = ParallelExecutionConfig::ignore_ordering();
    assert!(!cfg.respect_ordering);
}

#[test]
fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<ParallelSystemStage>();
    assert_sync::<ParallelSystemStage>();
}

#[test]
fn test_debug() {
    let stage = ParallelSystemStage::new("Test");
    let debug = format!("{:?}", stage);
    assert!(debug.contains("ParallelSystemStage"));
    assert!(debug.contains("Test"));
}

// ====================================================================
// System management
// ====================================================================

#[test]
fn test_add_system() {
    struct Sys;
    impl System for Sys {
        fn name(&self) -> &'static str {
            "Sys"
        }
        fn run(&mut self, _: &mut World) {}
    }
    let mut stage = ParallelSystemStage::new("Test");
    let id = stage.add_system(Sys);
    assert!(id.is_valid());
    assert_eq!(stage.system_count(), 1);
    assert!(stage.contains_system(id));
}

#[test]
fn test_remove_system() {
    struct Sys;
    impl System for Sys {
        fn name(&self) -> &'static str {
            "Sys"
        }
        fn run(&mut self, _: &mut World) {}
    }
    let mut stage = ParallelSystemStage::new("Test");
    let id = stage.add_system(Sys);
    assert!(stage.remove_system(id));
    assert_eq!(stage.system_count(), 0);
    assert!(!stage.remove_system(id));
}

#[test]
fn test_clear() {
    struct Sys;
    impl System for Sys {
        fn name(&self) -> &'static str {
            "Sys"
        }
        fn run(&mut self, _: &mut World) {}
    }
    let mut stage = ParallelSystemStage::new("Test");
    stage.add_system(Sys);
    stage.clear();
    assert!(stage.is_empty());
}

// ====================================================================
// Batch computation
// ====================================================================

#[test]
fn test_non_conflicting_same_batch() {
    struct PosWriter;
    impl System for PosWriter {
        fn name(&self) -> &'static str {
            "PosWriter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct VelWriter;
    impl System for VelWriter {
        fn name(&self) -> &'static str {
            "VelWriter"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Velocity>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    let mut stage = ParallelSystemStage::new("Test");
    stage.add_system(PosWriter);
    stage.add_system(VelWriter);
    stage.rebuild_batches().unwrap();
    assert_eq!(stage.batch_count(), 1);
    assert_eq!(stage.batches()[0].len(), 2);
}

#[test]
fn test_conflicting_different_batches() {
    struct WriterA;
    impl System for WriterA {
        fn name(&self) -> &'static str {
            "WriterA"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct WriterB;
    impl System for WriterB {
        fn name(&self) -> &'static str {
            "WriterB"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    let mut stage = ParallelSystemStage::new("Test");
    stage.add_system(WriterA);
    stage.add_system(WriterB);
    stage.rebuild_batches().unwrap();
    assert_eq!(stage.batch_count(), 2);
}

#[test]
fn test_run_parallel_basic() {
    struct CounterSystem {
        counter: Arc<AtomicU32>,
        id: u32,
    }
    impl System for CounterSystem {
        fn name(&self) -> &'static str {
            "CounterSystem"
        }
        fn run(&mut self, _: &mut World) {
            self.counter.fetch_add(1, AtomicOrdering::SeqCst);
        }
    }

    let counter = Arc::new(AtomicU32::new(0));
    let mut stage = ParallelSystemStage::new("Test");
    for i in 0..100 {
        stage.add_system(CounterSystem {
            counter: counter.clone(),
            id: i,
        });
    }
    let mut world = World::new();
    stage.run(&mut world);
    assert_eq!(counter.load(AtomicOrdering::SeqCst), 100);
    assert_eq!(stage.batch_count(), 1);
    assert_eq!(stage.batches()[0].len(), 100);
}

// ====================================================================
// Conflict detection
// ====================================================================

#[test]
fn test_conflict_detection() {
    struct WriterA;
    impl System for WriterA {
        fn name(&self) -> &'static str {
            "WriterA"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct ReaderA;
    impl System for ReaderA {
        fn name(&self) -> &'static str {
            "ReaderA"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    let mut stage = ParallelSystemStage::new("Test");
    stage.add_system(WriterA);
    stage.add_system(ReaderA);
    assert!(stage.has_conflicts());
    let conflicts = stage.find_conflicts();
    assert_eq!(conflicts.len(), 1);
}

#[test]
fn test_read_only_and_writing_systems() {
    struct Reader;
    impl System for Reader {
        fn name(&self) -> &'static str {
            "Reader"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_read(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    struct Writer;
    impl System for Writer {
        fn name(&self) -> &'static str {
            "Writer"
        }
        fn component_access(&self) -> Access {
            let mut a = Access::new();
            a.add_write(ComponentId::of::<Position>());
            a
        }
        fn run(&mut self, _: &mut World) {}
    }

    let mut stage = ParallelSystemStage::new("Test");
    let reader_id = stage.add_system(Reader);
    let writer_id = stage.add_system(Writer);
    let read_only = stage.read_only_systems();
    let writing = stage.writing_systems();
    assert_eq!(read_only.len(), 1);
    assert!(read_only.contains(&reader_id));
    assert_eq!(writing.len(), 1);
    assert!(writing.contains(&writer_id));
}

// ====================================================================
// Execution stats
// ====================================================================

#[test]
fn test_parallelism_ratio_empty() {
    let stats = ParallelExecutionStats::default();
    assert_eq!(stats.parallelism_ratio(), 0.0);
}

#[test]
fn test_parallelism_ratio_all_parallel() {
    let stats = ParallelExecutionStats {
        batch_count: 1,
        system_count: 10,
        parallel_systems: 10,
        sequential_systems: 0,
        max_parallelism: 10,
    };
    assert_eq!(stats.parallelism_ratio(), 1.0);
}

#[test]
fn test_parallelism_ratio_all_sequential() {
    let stats = ParallelExecutionStats {
        batch_count: 10,
        system_count: 10,
        parallel_systems: 0,
        sequential_systems: 10,
        max_parallelism: 1,
    };
    assert_eq!(stats.parallelism_ratio(), 0.0);
}

// ====================================================================
// Parallel batch
// ====================================================================

#[test]
fn test_parallel_batch() {
    let mut batch = ParallelBatch::new();
    assert!(batch.is_empty());
    assert!(!batch.can_parallelize());
    batch.add(SystemId::from_raw(1), true);
    assert_eq!(batch.len(), 1);
    assert!(batch.all_read_only);
    assert!(!batch.can_parallelize());
    batch.add(SystemId::from_raw(2), false);
    assert_eq!(batch.len(), 2);
    assert!(!batch.all_read_only);
    assert!(batch.can_parallelize());
}
