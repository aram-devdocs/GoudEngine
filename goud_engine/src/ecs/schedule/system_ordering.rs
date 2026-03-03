//! System ordering constraint types.

use std::fmt;

use crate::ecs::system::SystemId;

/// Specifies an ordering constraint between two systems.
///
/// Ordering constraints are used to ensure systems run in a specific order,
/// regardless of the order they were added to the stage.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::SystemOrdering;
/// use goud_engine::ecs::system::SystemId;
///
/// let ordering = SystemOrdering::Before {
///     system: SystemId::from_raw(1),
///     before: SystemId::from_raw(2),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemOrdering {
    /// The `system` must run before `before`.
    Before {
        /// The system that must run first.
        system: SystemId,
        /// The system that must run after.
        before: SystemId,
    },
    /// The `system` must run after `after`.
    After {
        /// The system that must run second.
        system: SystemId,
        /// The system that must run first.
        after: SystemId,
    },
}

impl SystemOrdering {
    /// Creates a constraint where `system` runs before `other`.
    #[inline]
    pub fn before(system: SystemId, other: SystemId) -> Self {
        SystemOrdering::Before {
            system,
            before: other,
        }
    }

    /// Creates a constraint where `system` runs after `other`.
    #[inline]
    pub fn after(system: SystemId, other: SystemId) -> Self {
        SystemOrdering::After {
            system,
            after: other,
        }
    }

    /// Returns the system that must run first according to this constraint.
    #[inline]
    pub fn first(&self) -> SystemId {
        match self {
            SystemOrdering::Before { system, .. } => *system,
            SystemOrdering::After { after, .. } => *after,
        }
    }

    /// Returns the system that must run second according to this constraint.
    #[inline]
    pub fn second(&self) -> SystemId {
        match self {
            SystemOrdering::Before { before, .. } => *before,
            SystemOrdering::After { system, .. } => *system,
        }
    }

    /// Returns the edge (from, to) for the dependency graph.
    ///
    /// The edge represents "from must run before to".
    #[inline]
    pub fn as_edge(&self) -> (SystemId, SystemId) {
        (self.first(), self.second())
    }

    /// Returns true if this ordering involves the given system.
    #[inline]
    pub fn involves(&self, id: SystemId) -> bool {
        self.first() == id || self.second() == id
    }
}

impl fmt::Display for SystemOrdering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemOrdering::Before { system, before } => {
                write!(f, "System {} before {}", system.raw(), before.raw())
            }
            SystemOrdering::After { system, after } => {
                write!(f, "System {} after {}", system.raw(), after.raw())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_ordering_before() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let ordering = SystemOrdering::before(a, b);
        assert_eq!(ordering.first(), a);
        assert_eq!(ordering.second(), b);
        assert_eq!(ordering.as_edge(), (a, b));
    }

    #[test]
    fn test_ordering_after() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let ordering = SystemOrdering::after(a, b);
        assert_eq!(ordering.first(), b);
        assert_eq!(ordering.second(), a);
        assert_eq!(ordering.as_edge(), (b, a));
    }

    #[test]
    fn test_ordering_involves() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        let ordering = SystemOrdering::before(a, b);
        assert!(ordering.involves(a));
        assert!(ordering.involves(b));
        assert!(!ordering.involves(c));
    }

    #[test]
    fn test_ordering_display() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let before = SystemOrdering::before(a, b);
        let after = SystemOrdering::after(a, b);
        let before_str = format!("{}", before);
        let after_str = format!("{}", after);
        assert!(before_str.contains("before"));
        assert!(after_str.contains("after"));
    }

    #[test]
    fn test_ordering_equality() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let o1 = SystemOrdering::before(a, b);
        let o2 = SystemOrdering::before(a, b);
        let o3 = SystemOrdering::before(b, a);
        assert_eq!(o1, o2);
        assert_ne!(o1, o3);
    }

    #[test]
    fn test_ordering_hash() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let mut set = HashSet::new();
        set.insert(SystemOrdering::before(a, b));
        set.insert(SystemOrdering::before(a, b));
        set.insert(SystemOrdering::before(b, a));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_ordering_clone() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let o1 = SystemOrdering::before(a, b);
        let o2 = o1;
        assert_eq!(o1, o2);
    }
}
