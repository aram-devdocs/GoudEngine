//! Conflict detection methods for SystemStage.
//!
//! Extracted to keep `system_stage.rs` under 500 lines.

use crate::ecs::system::SystemId;

use super::system_conflict::SystemConflict;
use super::system_stage::SystemStage;

impl SystemStage {
    /// Checks if any systems have conflicting access patterns.
    pub fn has_conflicts(&self) -> bool {
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                if self.systems[i].conflicts_with(&self.systems[j]) {
                    return true;
                }
            }
        }
        false
    }

    /// Finds all conflicting system pairs.
    pub fn find_conflicts(&self) -> Vec<SystemConflict> {
        let mut conflicts = Vec::new();
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                let access_i = self.systems[i].component_access();
                let access_j = self.systems[j].component_access();
                if let Some(access_conflict) = access_i.get_conflicts(&access_j) {
                    conflicts.push(SystemConflict {
                        first_system_id: self.systems[i].id(),
                        first_system_name: self.systems[i].name(),
                        second_system_id: self.systems[j].id(),
                        second_system_name: self.systems[j].name(),
                        conflict: access_conflict,
                    });
                }
            }
        }
        conflicts
    }

    /// Finds conflicts for a specific system.
    pub fn find_conflicts_for_system(&self, id: SystemId) -> Vec<SystemConflict> {
        let index = match self.system_indices.get(&id) {
            Some(&i) => i,
            None => return Vec::new(),
        };
        let mut conflicts = Vec::new();
        let access_target = self.systems[index].component_access();
        for (i, system) in self.systems.iter().enumerate() {
            if i == index {
                continue;
            }
            let access = system.component_access();
            if let Some(access_conflict) = access_target.get_conflicts(&access) {
                conflicts.push(SystemConflict {
                    first_system_id: self.systems[index].id(),
                    first_system_name: self.systems[index].name(),
                    second_system_id: system.id(),
                    second_system_name: system.name(),
                    conflict: access_conflict,
                });
            }
        }
        conflicts
    }

    /// Returns all read-only systems.
    pub fn read_only_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Returns all systems with write access.
    pub fn writing_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| !s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Groups systems by conflict-free parallel groups.
    pub fn compute_parallel_groups(&self) -> Vec<Vec<SystemId>> {
        if self.systems.is_empty() {
            return Vec::new();
        }
        let n = self.systems.len();
        let mut groups: Vec<Vec<SystemId>> = Vec::new();
        let mut assigned = vec![None::<usize>; n];
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            let mut found_group = false;
            for (group_idx, group) in groups.iter().enumerate() {
                let conflicts_with_group = group.iter().any(|&other_id| {
                    let other_idx = self.system_indices[&other_id];
                    self.systems[i].conflicts_with(&self.systems[other_idx])
                });
                if !conflicts_with_group {
                    assigned[i] = Some(group_idx);
                    found_group = true;
                    break;
                }
            }
            if !found_group {
                assigned[i] = Some(groups.len());
                groups.push(Vec::new());
            }
            let group_idx = assigned[i].unwrap();
            groups[group_idx].push(self.systems[i].id());
        }
        groups
    }

    /// Returns the number of conflicts.
    pub fn conflict_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                if self.systems[i].conflicts_with(&self.systems[j]) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Returns combined access pattern for all systems.
    pub fn combined_access(&self) -> crate::ecs::query::Access {
        let mut combined = crate::ecs::query::Access::new();
        for system in &self.systems {
            combined.extend(&system.component_access());
        }
        combined
    }
}
