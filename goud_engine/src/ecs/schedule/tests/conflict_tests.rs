//! Conflict detection tests for SystemStage.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::schedule::*;
use crate::ecs::system::System;
use crate::ecs::Component;
use crate::ecs::World;

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

#[derive(Debug, Clone, Copy)]
struct Health(f32);
impl Component for Health {}

struct PositionWriter;
impl System for PositionWriter {
    fn name(&self) -> &'static str {
        "PositionWriter"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_write(ComponentId::of::<Position>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct PositionReader;
impl System for PositionReader {
    fn name(&self) -> &'static str {
        "PositionReader"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_read(ComponentId::of::<Position>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct VelocityWriter;
impl System for VelocityWriter {
    fn name(&self) -> &'static str {
        "VelocityWriter"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_write(ComponentId::of::<Velocity>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct VelocityReader;
impl System for VelocityReader {
    fn name(&self) -> &'static str {
        "VelocityReader"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_read(ComponentId::of::<Velocity>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct MovementSystem;
impl System for MovementSystem {
    fn name(&self) -> &'static str {
        "MovementSystem"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_write(ComponentId::of::<Position>());
        a.add_read(ComponentId::of::<Velocity>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct HealthWriter;
impl System for HealthWriter {
    fn name(&self) -> &'static str {
        "HealthWriter"
    }
    fn component_access(&self) -> Access {
        let mut a = Access::new();
        a.add_write(ComponentId::of::<Health>());
        a
    }
    fn run(&mut self, _: &mut World) {}
}

struct NoAccessSystem;
impl System for NoAccessSystem {
    fn name(&self) -> &'static str {
        "NoAccessSystem"
    }
    fn run(&mut self, _: &mut World) {}
}

// ====================================================================
// has_conflicts
// ====================================================================

#[test]
fn test_has_conflicts_empty_stage() {
    assert!(!SystemStage::new("Test").has_conflicts());
}

#[test]
fn test_has_conflicts_single_system() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    assert!(!s.has_conflicts());
}

#[test]
fn test_has_conflicts_different_components() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(VelocityWriter);
    assert!(!s.has_conflicts());
}

#[test]
fn test_has_conflicts_both_readers() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionReader);
    s.add_system(PositionReader);
    assert!(!s.has_conflicts());
}

#[test]
fn test_has_conflicts_write_read() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    assert!(s.has_conflicts());
}

#[test]
fn test_has_conflicts_write_write() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionWriter);
    assert!(s.has_conflicts());
}

#[test]
fn test_has_conflicts_movement_and_position_reader() {
    let mut s = SystemStage::new("Test");
    s.add_system(MovementSystem);
    s.add_system(PositionReader);
    assert!(s.has_conflicts());
}

#[test]
fn test_has_conflicts_movement_and_velocity_writer() {
    let mut s = SystemStage::new("Test");
    s.add_system(MovementSystem);
    s.add_system(VelocityWriter);
    assert!(s.has_conflicts());
}

#[test]
fn test_has_conflicts_three_systems_partial() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(VelocityWriter);
    s.add_system(PositionReader);
    assert!(s.has_conflicts());
}

#[test]
fn test_has_conflicts_no_access_systems() {
    let mut s = SystemStage::new("Test");
    s.add_system(NoAccessSystem);
    s.add_system(NoAccessSystem);
    s.add_system(PositionWriter);
    assert!(!s.has_conflicts());
}

// ====================================================================
// find_conflicts
// ====================================================================

#[test]
fn test_find_conflicts_empty() {
    assert!(SystemStage::new("Test").find_conflicts().is_empty());
}

#[test]
fn test_find_conflicts_none() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(VelocityWriter);
    assert!(s.find_conflicts().is_empty());
}

#[test]
fn test_find_conflicts_one() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    let c = s.find_conflicts();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].first_system_name, "PositionWriter");
    assert_eq!(c[0].second_system_name, "PositionReader");
}

#[test]
fn test_find_conflicts_multiple() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    s.add_system(MovementSystem);
    let c = s.find_conflicts();
    assert!(c.len() >= 2);
}

#[test]
fn test_find_conflicts_for_system() {
    let mut s = SystemStage::new("Test");
    let pw_id = s.add_system(PositionWriter);
    s.add_system(PositionReader);
    s.add_system(VelocityWriter);
    let c = s.find_conflicts_for_system(pw_id);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].second_system_name, "PositionReader");
}

#[test]
fn test_find_conflicts_for_system_not_found() {
    let s = SystemStage::new("Test");
    let c = s.find_conflicts_for_system(crate::ecs::system::SystemId::new());
    assert!(c.is_empty());
}

// ====================================================================
// SystemConflict methods
// ====================================================================

#[test]
fn test_system_conflict_display() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    let c = s.find_conflicts();
    assert!(!c.is_empty());
    let display = format!("{}", c[0]);
    assert!(display.contains("PositionWriter"));
    assert!(display.contains("PositionReader"));
}

#[test]
fn test_system_conflict_accessors() {
    let mut s = SystemStage::new("Test");
    let id1 = s.add_system(PositionWriter);
    let id2 = s.add_system(PositionReader);
    let c = s.find_conflicts();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].system_ids(), (id1, id2));
    assert_eq!(c[0].system_names(), ("PositionWriter", "PositionReader"));
    assert!(c[0].total_conflict_count() > 0);
}

// ====================================================================
// Parallel groups and conflict_count
// ====================================================================

#[test]
fn test_compute_parallel_groups_empty() {
    assert!(SystemStage::new("Test")
        .compute_parallel_groups()
        .is_empty());
}

#[test]
fn test_compute_parallel_groups_no_conflicts() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(VelocityWriter);
    s.add_system(HealthWriter);
    let groups = s.compute_parallel_groups();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 3);
}

#[test]
fn test_compute_parallel_groups_with_conflicts() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    let groups = s.compute_parallel_groups();
    assert_eq!(groups.len(), 2);
}

#[test]
fn test_conflict_count() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(PositionReader);
    s.add_system(VelocityWriter);
    assert_eq!(s.conflict_count(), 1);
}

#[test]
fn test_read_only_systems() {
    let mut s = SystemStage::new("Test");
    let pr_id = s.add_system(PositionReader);
    s.add_system(PositionWriter);
    let vr_id = s.add_system(VelocityReader);
    let read_only = s.read_only_systems();
    assert_eq!(read_only.len(), 2);
    assert!(read_only.contains(&pr_id));
    assert!(read_only.contains(&vr_id));
}

#[test]
fn test_writing_systems() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionReader);
    let pw_id = s.add_system(PositionWriter);
    let writing = s.writing_systems();
    assert_eq!(writing.len(), 1);
    assert!(writing.contains(&pw_id));
}

#[test]
fn test_combined_access() {
    let mut s = SystemStage::new("Test");
    s.add_system(PositionWriter);
    s.add_system(VelocityReader);
    let combined = s.combined_access();
    assert!(combined.writes().contains(&ComponentId::of::<Position>()));
    assert!(combined
        .reads()
        .any(|c| *c == ComponentId::of::<Velocity>()));
}
