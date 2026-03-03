//! Shared conflict detection utilities for system stages.

use crate::ecs::system::{BoxedSystem, SystemId};

use super::system_conflict::SystemConflict;

/// Checks if any systems in the slice have conflicting access patterns.
pub(super) fn has_conflicts(systems: &[BoxedSystem]) -> bool {
    for i in 0..systems.len() {
        for j in (i + 1)..systems.len() {
            if systems[i].conflicts_with(&systems[j]) {
                return true;
            }
        }
    }
    false
}

/// Finds all conflicting system pairs.
pub(super) fn find_conflicts(systems: &[BoxedSystem]) -> Vec<SystemConflict> {
    let mut conflicts = Vec::new();
    for i in 0..systems.len() {
        for j in (i + 1)..systems.len() {
            let access_i = systems[i].component_access();
            let access_j = systems[j].component_access();
            if let Some(conflict) = access_i.get_conflicts(&access_j) {
                conflicts.push(SystemConflict {
                    first_system_id: systems[i].id(),
                    first_system_name: systems[i].name(),
                    second_system_id: systems[j].id(),
                    second_system_name: systems[j].name(),
                    conflict,
                });
            }
        }
    }
    conflicts
}

/// Returns IDs of all read-only systems.
pub(super) fn read_only_systems(systems: &[BoxedSystem]) -> Vec<SystemId> {
    systems
        .iter()
        .filter(|s| s.is_read_only())
        .map(|s| s.id())
        .collect()
}

/// Returns IDs of all systems with write access.
pub(super) fn writing_systems(systems: &[BoxedSystem]) -> Vec<SystemId> {
    systems
        .iter()
        .filter(|s| !s.is_read_only())
        .map(|s| s.id())
        .collect()
}
