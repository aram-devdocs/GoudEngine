//! Conflict detection and access analysis for ParallelSystemStage.

use crate::ecs::system::SystemId;

use super::parallel::ParallelSystemStage;
use super::system_conflict::SystemConflict;

impl ParallelSystemStage {
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
                let ai = self.systems[i].component_access();
                let aj = self.systems[j].component_access();
                if let Some(c) = ai.get_conflicts(&aj) {
                    conflicts.push(SystemConflict {
                        first_system_id: self.systems[i].id(),
                        first_system_name: self.systems[i].name(),
                        second_system_id: self.systems[j].id(),
                        second_system_name: self.systems[j].name(),
                        conflict: c,
                    });
                }
            }
        }
        conflicts
    }

    /// Returns all systems that are read-only.
    pub fn read_only_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Returns all systems that write.
    pub fn writing_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| !s.is_read_only())
            .map(|s| s.id())
            .collect()
    }
}
