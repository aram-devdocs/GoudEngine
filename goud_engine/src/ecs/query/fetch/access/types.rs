//! `AccessType` enum and `Access` struct for component/resource access tracking.

use std::collections::BTreeSet;

use crate::ecs::component::ComponentId;
use crate::ecs::resource::{NonSendResourceId, ResourceId};

use super::conflict::{AccessConflict, ConflictInfo, NonSendConflictInfo, ResourceConflictInfo};

// =============================================================================
// AccessType
// =============================================================================

/// Represents the type of access a query has to a component.
///
/// Used for detecting conflicts between queries. Two queries conflict if:
/// - One has `Write` access and the other has `Read` or `Write` to the same component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessType {
    /// Read-only access (`&T`)
    Read,
    /// Mutable access (`&mut T`)
    Write,
}

// =============================================================================
// Access
// =============================================================================

/// Describes the component access pattern for a query.
///
/// Used by the scheduler to determine which systems can run in parallel.
#[derive(Debug, Clone, Default)]
pub struct Access {
    /// Components accessed for reading only
    reads: BTreeSet<ComponentId>,
    /// Components accessed for writing (also counts as read)
    writes: BTreeSet<ComponentId>,
    /// Resources accessed for reading only
    resource_reads: BTreeSet<ResourceId>,
    /// Resources accessed for writing (also counts as read)
    resource_writes: BTreeSet<ResourceId>,
    /// Non-send resources accessed for reading only
    non_send_reads: BTreeSet<NonSendResourceId>,
    /// Non-send resources accessed for writing (also counts as read)
    non_send_writes: BTreeSet<NonSendResourceId>,
}

impl Access {
    /// Creates a new empty access descriptor.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a read access for the given component.
    #[inline]
    pub fn add_read(&mut self, id: ComponentId) {
        self.reads.insert(id);
    }

    /// Adds a write access for the given component.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_write(&mut self, id: ComponentId) {
        self.writes.insert(id);
    }

    /// Returns all components that are read (including those also written).
    #[inline]
    pub fn reads(&self) -> impl Iterator<Item = &ComponentId> {
        self.reads.iter().chain(self.writes.iter())
    }

    /// Returns all components that are written.
    #[inline]
    pub fn writes(&self) -> &BTreeSet<ComponentId> {
        &self.writes
    }

    /// Returns the set of read-only components (read but not written).
    #[inline]
    pub fn reads_only(&self) -> impl Iterator<Item = &ComponentId> {
        self.reads.iter().filter(|id| !self.writes.contains(id))
    }

    /// Checks if this access conflicts with another.
    ///
    /// Two accesses conflict if:
    /// - One writes to a component that the other reads or writes
    /// - One writes to a resource that the other reads or writes
    #[inline]
    pub fn conflicts_with(&self, other: &Access) -> bool {
        // Check if our writes conflict with their reads or writes
        for write in &self.writes {
            if other.reads.contains(write) || other.writes.contains(write) {
                return true;
            }
        }

        // Check if their writes conflict with our reads
        for write in &other.writes {
            if self.reads.contains(write) {
                return true;
            }
        }

        // Check resource conflicts
        if self.resource_conflicts_with(other) {
            return true;
        }

        // Check non-send resource conflicts
        self.non_send_conflicts_with(other)
    }

    /// Returns true if this access pattern is read-only.
    ///
    /// This checks component, resource, and non-send resource access.
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.writes.is_empty() && self.resource_writes.is_empty() && self.non_send_writes.is_empty()
    }

    /// Merges another access into this one.
    #[inline]
    pub fn extend(&mut self, other: &Access) {
        self.reads.extend(other.reads.iter().copied());
        self.writes.extend(other.writes.iter().copied());
        self.resource_reads
            .extend(other.resource_reads.iter().copied());
        self.resource_writes
            .extend(other.resource_writes.iter().copied());
        self.non_send_reads
            .extend(other.non_send_reads.iter().copied());
        self.non_send_writes
            .extend(other.non_send_writes.iter().copied());
    }

    // =========================================================================
    // Resource Access
    // =========================================================================

    /// Adds a read access for the given resource.
    #[inline]
    pub fn add_resource_read(&mut self, id: ResourceId) {
        self.resource_reads.insert(id);
    }

    /// Adds a write access for the given resource.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_resource_write(&mut self, id: ResourceId) {
        self.resource_writes.insert(id);
    }

    /// Returns all resources that are read (including those also written).
    #[inline]
    pub fn resource_reads(&self) -> impl Iterator<Item = &ResourceId> {
        self.resource_reads
            .iter()
            .chain(self.resource_writes.iter())
    }

    /// Returns all resources that are written.
    #[inline]
    pub fn resource_writes(&self) -> &BTreeSet<ResourceId> {
        &self.resource_writes
    }

    /// Returns the set of read-only resources (read but not written).
    #[inline]
    pub fn resource_reads_only(&self) -> impl Iterator<Item = &ResourceId> {
        self.resource_reads
            .iter()
            .filter(|id| !self.resource_writes.contains(id))
    }

    /// Checks if resource access conflicts with another.
    ///
    /// Two accesses conflict if one writes to a resource that the other
    /// reads or writes.
    #[inline]
    pub fn resource_conflicts_with(&self, other: &Access) -> bool {
        for write in &self.resource_writes {
            if other.resource_reads.contains(write) || other.resource_writes.contains(write) {
                return true;
            }
        }
        for write in &other.resource_writes {
            if self.resource_reads.contains(write) {
                return true;
            }
        }
        false
    }

    /// Checks if this access has any resource access.
    #[inline]
    pub fn has_resource_access(&self) -> bool {
        !self.resource_reads.is_empty() || !self.resource_writes.is_empty()
    }

    /// Checks if this access has any component access.
    #[inline]
    pub fn has_component_access(&self) -> bool {
        !self.reads.is_empty() || !self.writes.is_empty()
    }

    // =========================================================================
    // Non-Send Resource Access
    // =========================================================================

    /// Adds a read access for the given non-send resource.
    #[inline]
    pub fn add_non_send_read(&mut self, id: NonSendResourceId) {
        self.non_send_reads.insert(id);
    }

    /// Adds a write access for the given non-send resource.
    ///
    /// Write access implies read access.
    #[inline]
    pub fn add_non_send_write(&mut self, id: NonSendResourceId) {
        self.non_send_writes.insert(id);
    }

    /// Returns all non-send resources that are read (including those also written).
    #[inline]
    pub fn non_send_reads(&self) -> impl Iterator<Item = &NonSendResourceId> {
        self.non_send_reads
            .iter()
            .chain(self.non_send_writes.iter())
    }

    /// Returns all non-send resources that are written.
    #[inline]
    pub fn non_send_writes(&self) -> &BTreeSet<NonSendResourceId> {
        &self.non_send_writes
    }

    /// Returns the set of read-only non-send resources (read but not written).
    #[inline]
    pub fn non_send_reads_only(&self) -> impl Iterator<Item = &NonSendResourceId> {
        self.non_send_reads
            .iter()
            .filter(|id| !self.non_send_writes.contains(id))
    }

    /// Checks if non-send resource access conflicts with another.
    ///
    /// Two accesses conflict if one writes to a non-send resource that the other
    /// reads or writes.
    #[inline]
    pub fn non_send_conflicts_with(&self, other: &Access) -> bool {
        for write in &self.non_send_writes {
            if other.non_send_reads.contains(write) || other.non_send_writes.contains(write) {
                return true;
            }
        }
        for write in &other.non_send_writes {
            if self.non_send_reads.contains(write) {
                return true;
            }
        }
        false
    }

    /// Checks if this access has any non-send resource access.
    #[inline]
    pub fn has_non_send_access(&self) -> bool {
        !self.non_send_reads.is_empty() || !self.non_send_writes.is_empty()
    }

    /// Returns true if this access requires execution on the main thread.
    ///
    /// This returns true if there is any non-send resource access.
    #[inline]
    pub fn requires_main_thread(&self) -> bool {
        self.has_non_send_access()
    }

    /// Returns detailed information about conflicts between this access and another.
    ///
    /// If there are no conflicts, returns `None`. Otherwise, returns an
    /// `AccessConflict` describing all the conflicting components and resources.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::query::Access;
    /// use goud_engine::ecs::component::ComponentId;
    /// use goud_engine::ecs::Component;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut access1 = Access::new();
    /// access1.add_write(ComponentId::of::<Position>());
    ///
    /// let mut access2 = Access::new();
    /// access2.add_read(ComponentId::of::<Position>());
    ///
    /// let conflict = access1.get_conflicts(&access2);
    /// assert!(conflict.is_some());
    /// let conflict = conflict.unwrap();
    /// assert_eq!(conflict.component_conflicts().len(), 1);
    /// ```
    pub fn get_conflicts(&self, other: &Access) -> Option<AccessConflict> {
        let mut component_conflicts = Vec::new();
        let mut resource_conflicts = Vec::new();
        let mut non_send_conflicts = Vec::new();

        // Check component conflicts: our writes vs their reads/writes
        for &write in &self.writes {
            if other.reads.contains(&write) {
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.writes.contains(&write) {
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check component conflicts: their writes vs our reads
        for &write in &other.writes {
            if self.reads.contains(&write) && !self.writes.contains(&write) {
                component_conflicts.push(ConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        // Check resource conflicts: our writes vs their reads/writes
        for &write in &self.resource_writes {
            if other.resource_reads.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.resource_writes.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check resource conflicts: their writes vs our reads
        for &write in &other.resource_writes {
            if self.resource_reads.contains(&write) && !self.resource_writes.contains(&write) {
                resource_conflicts.push(ResourceConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        // Check non-send resource conflicts: our writes vs their reads/writes
        for &write in &self.non_send_writes {
            if other.non_send_reads.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Read,
                ));
            } else if other.non_send_writes.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Write,
                    AccessType::Write,
                ));
            }
        }

        // Check non-send resource conflicts: their writes vs our reads
        for &write in &other.non_send_writes {
            if self.non_send_reads.contains(&write) && !self.non_send_writes.contains(&write) {
                non_send_conflicts.push(NonSendConflictInfo::new(
                    write,
                    AccessType::Read,
                    AccessType::Write,
                ));
            }
        }

        if component_conflicts.is_empty()
            && resource_conflicts.is_empty()
            && non_send_conflicts.is_empty()
        {
            None
        } else {
            Some(AccessConflict::from_parts(
                component_conflicts,
                resource_conflicts,
                non_send_conflicts,
            ))
        }
    }

    /// Returns true if this access is empty (no reads or writes).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.reads.is_empty()
            && self.writes.is_empty()
            && self.resource_reads.is_empty()
            && self.resource_writes.is_empty()
            && self.non_send_reads.is_empty()
            && self.non_send_writes.is_empty()
    }

    /// Clears all access information.
    #[inline]
    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
        self.resource_reads.clear();
        self.resource_writes.clear();
        self.non_send_reads.clear();
        self.non_send_writes.clear();
    }
}
