//! System ordering and chain tests for SystemStage.

use crate::ecs::schedule::*;
use crate::ecs::system::{System, SystemId};
use crate::ecs::World;

struct SysA;
impl System for SysA {
    fn name(&self) -> &'static str {
        "A"
    }
    fn run(&mut self, _: &mut World) {}
}

struct SysB;
impl System for SysB {
    fn name(&self) -> &'static str {
        "B"
    }
    fn run(&mut self, _: &mut World) {}
}

struct SysC;
impl System for SysC {
    fn name(&self) -> &'static str {
        "C"
    }
    fn run(&mut self, _: &mut World) {}
}

// ====================================================================
// add_ordering
// ====================================================================

#[test]
fn test_add_ordering_basic() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    assert!(stage.add_ordering(a, b));
    assert_eq!(stage.ordering_count(), 1);
}

#[test]
fn test_add_ordering_marks_dirty() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    assert!(!stage.is_order_dirty());
    stage.add_ordering(a, b);
    assert!(stage.is_order_dirty());
}

#[test]
fn test_add_ordering_missing_system() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    assert!(!stage.add_ordering(a, SystemId::new()));
    assert!(!stage.add_ordering(SystemId::new(), a));
}

#[test]
fn test_add_ordering_self_reference() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    assert!(!stage.add_ordering(a, a));
}

#[test]
fn test_add_ordering_duplicate() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    assert!(stage.add_ordering(a, b));
    assert!(stage.add_ordering(a, b)); // Duplicate returns true (no-op)
    assert_eq!(stage.ordering_count(), 1);
}

#[test]
fn test_set_before_after() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    assert!(stage.set_before(a, b));
    assert!(stage.set_after(b, a)); // Same as before(a, b)
    assert_eq!(stage.ordering_count(), 1);
}

#[test]
fn test_remove_orderings_for() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    let c = stage.add_system(SysC);
    stage.add_ordering(a, b);
    stage.add_ordering(b, c);
    let removed = stage.remove_orderings_for(b);
    assert_eq!(removed, 2);
    assert_eq!(stage.ordering_count(), 0);
}

#[test]
fn test_remove_orderings_for_none() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    assert_eq!(stage.remove_orderings_for(a), 0);
}

#[test]
fn test_clear_orderings() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    stage.add_ordering(a, b);
    stage.clear_orderings();
    assert_eq!(stage.ordering_count(), 0);
}

#[test]
fn test_orderings_for() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    let c = stage.add_system(SysC);
    stage.add_ordering(a, b);
    stage.add_ordering(b, c);
    let orderings = stage.orderings_for(b);
    assert_eq!(orderings.len(), 2);
}

// ====================================================================
// rebuild_order
// ====================================================================

#[test]
fn test_rebuild_order_basic() {
    let mut stage = SystemStage::new("Test");
    let b = stage.add_system(SysB);
    let a = stage.add_system(SysA);
    stage.add_ordering(a, b);
    stage.rebuild_order().unwrap();
    assert_eq!(stage.system_names(), vec!["A", "B"]);
    assert!(!stage.is_order_dirty());
}

#[test]
fn test_rebuild_order_no_orderings() {
    let mut stage = SystemStage::new("Test");
    stage.add_system(SysA);
    stage.add_system(SysB);
    assert!(stage.rebuild_order().is_ok());
}

#[test]
fn test_rebuild_order_cycle() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    stage.add_ordering(a, b);
    stage.add_ordering(b, a);
    assert!(stage.rebuild_order().is_err());
}

#[test]
fn test_rebuild_order_three_systems() {
    let mut stage = SystemStage::new("Test");
    let c = stage.add_system(SysC);
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    stage.add_ordering(a, b);
    stage.add_ordering(b, c);
    stage.rebuild_order().unwrap();
    assert_eq!(stage.system_names(), vec!["A", "B", "C"]);
}

#[test]
fn test_would_ordering_cycle() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    stage.add_ordering(a, b);
    assert!(stage.would_ordering_cycle(b, a));
    assert!(!stage.would_ordering_cycle(a, b));
}

#[test]
fn test_run_auto_rebuilds_order() {
    let mut stage = SystemStage::new("Test");
    let c = stage.add_system(SysC);
    let b = stage.add_system(SysB);
    let a = stage.add_system(SysA);
    stage.add_ordering(a, b);
    stage.add_ordering(b, c);
    let mut world = World::new();
    stage.run(&mut world);
    assert_eq!(stage.system_names(), vec!["A", "B", "C"]);
}

// ====================================================================
// chain_systems and add_chain
// ====================================================================

#[test]
fn test_chain_systems_empty() {
    let mut stage = SystemStage::new("Test");
    assert_eq!(stage.chain_systems([]), 0);
    assert_eq!(stage.ordering_count(), 0);
}

#[test]
fn test_chain_systems_single() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    assert_eq!(stage.chain_systems([a]), 0);
    assert_eq!(stage.ordering_count(), 0);
}

#[test]
fn test_chain_systems_two() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    assert_eq!(stage.chain_systems([a, b]), 1);
    assert_eq!(stage.ordering_count(), 1);
}

#[test]
fn test_chain_systems_three() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    let c = stage.add_system(SysC);
    assert_eq!(stage.chain_systems([a, b, c]), 2);
    assert_eq!(stage.ordering_count(), 2);
}

#[test]
fn test_chain_systems_enforces_order() {
    let mut stage = SystemStage::new("Test");
    let c = stage.add_system(SysC);
    let b = stage.add_system(SysB);
    let a = stage.add_system(SysA);
    stage.chain_systems([a, b, c]);
    stage.rebuild_order().expect("No cycles");
    assert_eq!(stage.system_names(), vec!["A", "B", "C"]);
}

#[test]
fn test_add_chain_empty() {
    let mut stage = SystemStage::new("Test");
    let chain = ChainedSystems::new("Empty");
    assert_eq!(stage.add_chain(&chain), 0);
}

#[test]
fn test_add_chain_with_systems() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    let c = stage.add_system(SysC);
    let mut chain = ChainedSystems::new("TestChain");
    chain.add(a);
    chain.add(b);
    chain.add(c);
    assert_eq!(stage.add_chain(&chain), 2);
    assert_eq!(stage.ordering_count(), 2);
}

#[test]
fn test_add_chain_enforces_order() {
    let mut stage = SystemStage::new("Test");
    let c = stage.add_system(SysC);
    let b = stage.add_system(SysB);
    let a = stage.add_system(SysA);
    let mut chain = ChainedSystems::new("TestChain");
    chain.add(a);
    chain.add(b);
    chain.add(c);
    stage.add_chain(&chain);
    stage.rebuild_order().expect("No cycles");
    assert_eq!(stage.system_names(), vec!["A", "B", "C"]);
}

#[test]
fn test_chain_marks_dirty() {
    let mut stage = SystemStage::new("Test");
    let a = stage.add_system(SysA);
    let b = stage.add_system(SysB);
    let _ = stage.rebuild_order();
    assert!(!stage.is_order_dirty());
    stage.chain_systems([a, b]);
    assert!(stage.is_order_dirty());
}
