//! Access conflict reporting types for the query system.
//!
//! These types describe which components and resources cause conflicts between
//! two [`Access`] descriptors.

use std::fmt;

use crate::ecs::component::ComponentId;
use crate::ecs::resource::{NonSendResourceId, ResourceId};

use super::types::AccessType;

// =============================================================================
// ConflictInfo
// =============================================================================

/// Information about a single component access conflict.
///
/// Describes which component conflicts and what type of access each side has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictInfo {
    /// The conflicting component ID.
    pub component_id: ComponentId,
    /// How the first access pattern accesses this component.
    pub first_access: AccessType,
    /// How the second access pattern accesses this component.
    pub second_access: AccessType,
}

impl ConflictInfo {
    /// Creates a new conflict info.
    #[inline]
    pub fn new(
        component_id: ComponentId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            component_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }

    /// Returns true if this is a read-write conflict.
    #[inline]
    pub fn is_read_write(&self) -> bool {
        (self.first_access == AccessType::Read && self.second_access == AccessType::Write)
            || (self.first_access == AccessType::Write && self.second_access == AccessType::Read)
    }
}

impl fmt::Display for ConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Component {:?}: {:?} vs {:?}",
            self.component_id, self.first_access, self.second_access
        )
    }
}

// =============================================================================
// ResourceConflictInfo
// =============================================================================

/// Information about a single resource access conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceConflictInfo {
    /// The conflicting resource ID.
    pub resource_id: ResourceId,
    /// How the first access pattern accesses this resource.
    pub first_access: AccessType,
    /// How the second access pattern accesses this resource.
    pub second_access: AccessType,
}

impl ResourceConflictInfo {
    /// Creates a new resource conflict info.
    #[inline]
    pub fn new(
        resource_id: ResourceId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            resource_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }
}

impl fmt::Display for ResourceConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resource {:?}: {:?} vs {:?}",
            self.resource_id, self.first_access, self.second_access
        )
    }
}

// =============================================================================
// NonSendConflictInfo
// =============================================================================

/// Information about a single non-send resource access conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonSendConflictInfo {
    /// The conflicting non-send resource ID.
    pub resource_id: NonSendResourceId,
    /// How the first access pattern accesses this resource.
    pub first_access: AccessType,
    /// How the second access pattern accesses this resource.
    pub second_access: AccessType,
}

impl NonSendConflictInfo {
    /// Creates a new non-send resource conflict info.
    #[inline]
    pub fn new(
        resource_id: NonSendResourceId,
        first_access: AccessType,
        second_access: AccessType,
    ) -> Self {
        Self {
            resource_id,
            first_access,
            second_access,
        }
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.first_access == AccessType::Write && self.second_access == AccessType::Write
    }
}

impl fmt::Display for NonSendConflictInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NonSendResource {:?}: {:?} vs {:?}",
            self.resource_id, self.first_access, self.second_access
        )
    }
}

// =============================================================================
// AccessConflict
// =============================================================================

/// Detailed information about all conflicts between two access patterns.
///
/// This struct is returned by
/// [`Access::get_conflicts`](super::types::Access::get_conflicts) when there are
/// conflicting accesses. It contains separate lists of component, resource,
/// and non-send resource conflicts.
///
/// # Usage
///
/// ```
/// use goud_engine::ecs::query::{Access, AccessConflict};
/// use goud_engine::ecs::component::ComponentId;
/// use goud_engine::ecs::Component;
///
/// #[derive(Clone, Copy)]
/// struct Health(f32);
/// impl Component for Health {}
///
/// let mut system_a = Access::new();
/// system_a.add_write(ComponentId::of::<Health>());
///
/// let mut system_b = Access::new();
/// system_b.add_read(ComponentId::of::<Health>());
///
/// if let Some(conflict) = system_a.get_conflicts(&system_b) {
///     println!("Systems conflict on {} components", conflict.component_count());
///     for info in conflict.component_conflicts() {
///         println!("  - {}", info);
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessConflict {
    /// Component access conflicts.
    component_conflicts: Vec<ConflictInfo>,
    /// Resource access conflicts.
    resource_conflicts: Vec<ResourceConflictInfo>,
    /// Non-send resource access conflicts.
    non_send_conflicts: Vec<NonSendConflictInfo>,
}

impl AccessConflict {
    /// Creates a new empty access conflict.
    #[inline]
    pub fn new() -> Self {
        Self {
            component_conflicts: Vec::new(),
            resource_conflicts: Vec::new(),
            non_send_conflicts: Vec::new(),
        }
    }

    /// Creates an `AccessConflict` from pre-built conflict vectors.
    #[inline]
    pub(super) fn from_parts(
        component_conflicts: Vec<ConflictInfo>,
        resource_conflicts: Vec<ResourceConflictInfo>,
        non_send_conflicts: Vec<NonSendConflictInfo>,
    ) -> Self {
        Self {
            component_conflicts,
            resource_conflicts,
            non_send_conflicts,
        }
    }

    /// Returns the component conflicts.
    #[inline]
    pub fn component_conflicts(&self) -> &[ConflictInfo] {
        &self.component_conflicts
    }

    /// Returns the resource conflicts.
    #[inline]
    pub fn resource_conflicts(&self) -> &[ResourceConflictInfo] {
        &self.resource_conflicts
    }

    /// Returns the non-send resource conflicts.
    #[inline]
    pub fn non_send_conflicts(&self) -> &[NonSendConflictInfo] {
        &self.non_send_conflicts
    }

    /// Returns the total number of component conflicts.
    #[inline]
    pub fn component_count(&self) -> usize {
        self.component_conflicts.len()
    }

    /// Returns the total number of resource conflicts.
    #[inline]
    pub fn resource_count(&self) -> usize {
        self.resource_conflicts.len()
    }

    /// Returns the total number of non-send resource conflicts.
    #[inline]
    pub fn non_send_count(&self) -> usize {
        self.non_send_conflicts.len()
    }

    /// Returns the total number of conflicts across all categories.
    #[inline]
    pub fn total_count(&self) -> usize {
        self.component_conflicts.len()
            + self.resource_conflicts.len()
            + self.non_send_conflicts.len()
    }

    /// Returns true if there are no conflicts.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.component_conflicts.is_empty()
            && self.resource_conflicts.is_empty()
            && self.non_send_conflicts.is_empty()
    }

    /// Returns true if any conflict is a write-write conflict.
    #[inline]
    pub fn has_write_write(&self) -> bool {
        self.component_conflicts.iter().any(|c| c.is_write_write())
            || self.resource_conflicts.iter().any(|c| c.is_write_write())
            || self.non_send_conflicts.iter().any(|c| c.is_write_write())
    }

    /// Returns an iterator over all conflicting component IDs.
    #[inline]
    pub fn conflicting_components(&self) -> impl Iterator<Item = ComponentId> + '_ {
        self.component_conflicts.iter().map(|c| c.component_id)
    }

    /// Returns an iterator over all conflicting resource IDs.
    #[inline]
    pub fn conflicting_resources(&self) -> impl Iterator<Item = ResourceId> + '_ {
        self.resource_conflicts.iter().map(|c| c.resource_id)
    }

    /// Returns an iterator over all conflicting non-send resource IDs.
    #[inline]
    pub fn conflicting_non_send_resources(&self) -> impl Iterator<Item = NonSendResourceId> + '_ {
        self.non_send_conflicts.iter().map(|c| c.resource_id)
    }
}

impl Default for AccessConflict {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AccessConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccessConflict(")?;
        let mut first = true;

        for conflict in &self.component_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        for conflict in &self.resource_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        for conflict in &self.non_send_conflicts {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", conflict)?;
            first = false;
        }

        write!(f, ")")
    }
}
