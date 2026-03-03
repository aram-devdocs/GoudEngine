//! Conflict detection and access analysis for ParallelSystemStage.

use crate::ecs::system::SystemId;

use super::parallel::ParallelSystemStage;
use super::system_conflict::SystemConflict;

impl ParallelSystemStage {
    /// Checks if any systems have conflicting access patterns.
    pub fn has_conflicts(&self) -> bool {
        super::conflict_utils::has_conflicts(&self.systems)
    }

    /// Finds all conflicting system pairs.
    pub fn find_conflicts(&self) -> Vec<SystemConflict> {
        super::conflict_utils::find_conflicts(&self.systems)
    }

    /// Returns all systems that are read-only.
    pub fn read_only_systems(&self) -> Vec<SystemId> {
        super::conflict_utils::read_only_systems(&self.systems)
    }

    /// Returns all systems that write.
    pub fn writing_systems(&self) -> Vec<SystemId> {
        super::conflict_utils::writing_systems(&self.systems)
    }
}
