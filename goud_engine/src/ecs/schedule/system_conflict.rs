//! System conflict detection types.

use std::fmt;

use crate::ecs::system::SystemId;

/// Information about a conflict between two systems.
///
/// This struct provides detailed information about which systems conflict
/// and what specific accesses cause the conflict.
#[derive(Debug, Clone)]
pub struct SystemConflict {
    /// ID of the first conflicting system.
    pub first_system_id: SystemId,
    /// Name of the first conflicting system.
    pub first_system_name: &'static str,
    /// ID of the second conflicting system.
    pub second_system_id: SystemId,
    /// Name of the second conflicting system.
    pub second_system_name: &'static str,
    /// Detailed access conflict information.
    pub conflict: crate::ecs::query::AccessConflict,
}

impl SystemConflict {
    /// Returns the pair of system IDs involved in this conflict.
    #[inline]
    pub fn system_ids(&self) -> (SystemId, SystemId) {
        (self.first_system_id, self.second_system_id)
    }

    /// Returns the pair of system names involved in this conflict.
    #[inline]
    pub fn system_names(&self) -> (&'static str, &'static str) {
        (self.first_system_name, self.second_system_name)
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.conflict.has_write_write()
    }

    /// Returns the number of conflicting components.
    #[inline]
    pub fn component_conflict_count(&self) -> usize {
        self.conflict.component_count()
    }

    /// Returns the number of conflicting resources.
    #[inline]
    pub fn resource_conflict_count(&self) -> usize {
        self.conflict.resource_count()
    }

    /// Returns the total number of conflicts.
    #[inline]
    pub fn total_conflict_count(&self) -> usize {
        self.conflict.total_count()
    }
}

impl fmt::Display for SystemConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Conflict: '{}' vs '{}' - {}",
            self.first_system_name, self.second_system_name, self.conflict
        )
    }
}
