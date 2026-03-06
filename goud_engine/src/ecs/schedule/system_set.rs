//! System set, configuration, chained systems, and label-based ordering.

use std::collections::HashSet;
use std::fmt;

use crate::ecs::system::SystemId;

use super::system_label::{SystemLabel, SystemLabelId};
use super::system_ordering::SystemOrdering;

/// A named set of systems for grouping and batch configuration.
///
/// System sets allow applying ordering constraints and run conditions
/// to multiple systems at once.
#[derive(Debug, Clone)]
pub struct SystemSet {
    /// Human-readable name for the set.
    name: String,
    /// Systems in this set.
    systems: Vec<SystemId>,
    /// Unique set for fast contains checks.
    system_set: HashSet<SystemId>,
}

impl SystemSet {
    /// Creates a new empty system set.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_set: HashSet::new(),
        }
    }

    /// Creates a new system set with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_set: HashSet::with_capacity(capacity),
        }
    }

    /// Returns the name of this set.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a system to this set.
    ///
    /// Returns `true` if the system was added, `false` if already present.
    pub fn add(&mut self, system: SystemId) -> bool {
        if self.system_set.insert(system) {
            self.systems.push(system);
            true
        } else {
            false
        }
    }

    /// Removes a system from this set.
    ///
    /// Returns `true` if the system was removed, `false` if not present.
    pub fn remove(&mut self, system: SystemId) -> bool {
        if self.system_set.remove(&system) {
            self.systems.retain(|&id| id != system);
            true
        } else {
            false
        }
    }

    /// Returns `true` if this set contains the given system.
    #[inline]
    pub fn contains(&self, system: SystemId) -> bool {
        self.system_set.contains(&system)
    }

    /// Returns the number of systems in this set.
    #[inline]
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Returns `true` if this set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Returns an iterator over systems in this set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().copied()
    }

    /// Clears all systems from this set.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_set.clear();
    }
}

impl Default for SystemSet {
    fn default() -> Self {
        Self::new("DefaultSet")
    }
}

// ============================================================================
// SystemSetConfig - Configuration for System Sets
// ============================================================================

/// Configuration for a system set's execution behavior.
///
/// This allows setting ordering constraints and run conditions
/// that apply to all systems in the set.
#[derive(Debug, Clone)]
pub struct SystemSetConfig {
    /// Labels that this set should run before.
    pub before_labels: Vec<SystemLabelId>,
    /// Labels that this set should run after.
    pub after_labels: Vec<SystemLabelId>,
    /// Whether the set is enabled.
    pub enabled: bool,
}

impl Default for SystemSetConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSetConfig {
    /// Creates a new default configuration.
    pub fn new() -> Self {
        Self {
            before_labels: Vec::new(),
            after_labels: Vec::new(),
            enabled: true,
        }
    }

    /// Adds a "run before" constraint.
    pub fn before<L: SystemLabel + Clone>(mut self, label: L) -> Self {
        self.before_labels.push(SystemLabelId::of(label));
        self
    }

    /// Adds a "run after" constraint.
    pub fn after<L: SystemLabel + Clone>(mut self, label: L) -> Self {
        self.after_labels.push(SystemLabelId::of(label));
        self
    }

    /// Sets whether the set is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

// ============================================================================
// ChainedSystems - Strict Sequential Ordering
// ============================================================================

/// A chain of systems that must run in strict sequential order.
///
/// Chained systems are guaranteed to run one after another with no
/// interleaving. This is useful when systems have implicit dependencies
/// that cannot be expressed through component access patterns.
#[derive(Debug, Clone)]
pub struct ChainedSystems {
    /// Name of the chain for debugging.
    name: String,
    /// Systems in chain order.
    systems: Vec<SystemId>,
}

impl ChainedSystems {
    /// Creates a new empty chain.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
        }
    }

    /// Creates a chain with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
        }
    }

    /// Returns the name of this chain.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a system to the end of the chain.
    pub fn add(&mut self, system: SystemId) {
        self.systems.push(system);
    }

    /// Adds a system that must run immediately after another.
    ///
    /// If `after` is not in the chain, the system is added at the end.
    pub fn add_after(&mut self, system: SystemId, after: SystemId) {
        if let Some(pos) = self.systems.iter().position(|&id| id == after) {
            self.systems.insert(pos + 1, system);
        } else {
            self.systems.push(system);
        }
    }

    /// Returns the number of systems in the chain.
    #[inline]
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Returns `true` if the chain is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Returns an iterator over systems in chain order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().copied()
    }

    /// Converts this chain to a vector of ordering constraints.
    ///
    /// Each consecutive pair becomes a "before" constraint.
    pub fn to_orderings(&self) -> Vec<SystemOrdering> {
        self.systems
            .windows(2)
            .map(|pair| SystemOrdering::before(pair[0], pair[1]))
            .collect()
    }

    /// Clears the chain.
    pub fn clear(&mut self) {
        self.systems.clear();
    }
}

impl Default for ChainedSystems {
    fn default() -> Self {
        Self::new("Chain")
    }
}

/// Creates ordering constraints from a sequence of system IDs.
///
/// Each consecutive pair becomes a "before" constraint.
pub fn chain<I>(systems: I) -> Vec<SystemOrdering>
where
    I: IntoIterator<Item = SystemId>,
{
    let systems: Vec<_> = systems.into_iter().collect();
    systems
        .windows(2)
        .map(|pair| SystemOrdering::before(pair[0], pair[1]))
        .collect()
}

// ============================================================================
// LabeledOrderingConstraint - Label-based Ordering
// ============================================================================

/// An ordering constraint that uses labels instead of system IDs.
///
/// This allows defining ordering relationships before systems are
/// registered, using labels as placeholders.
#[derive(Debug, Clone)]
pub enum LabeledOrderingConstraint {
    /// Run before systems with this label.
    BeforeLabel(SystemLabelId),
    /// Run after systems with this label.
    AfterLabel(SystemLabelId),
    /// Run before a specific system.
    BeforeSystem(SystemId),
    /// Run after a specific system.
    AfterSystem(SystemId),
}

impl LabeledOrderingConstraint {
    /// Creates a "before label" constraint.
    pub fn before_label<L: SystemLabel + Clone>(label: L) -> Self {
        LabeledOrderingConstraint::BeforeLabel(SystemLabelId::of(label))
    }

    /// Creates an "after label" constraint.
    pub fn after_label<L: SystemLabel + Clone>(label: L) -> Self {
        LabeledOrderingConstraint::AfterLabel(SystemLabelId::of(label))
    }

    /// Creates a "before system" constraint.
    #[inline]
    pub fn before_system(id: SystemId) -> Self {
        LabeledOrderingConstraint::BeforeSystem(id)
    }

    /// Creates an "after system" constraint.
    #[inline]
    pub fn after_system(id: SystemId) -> Self {
        LabeledOrderingConstraint::AfterSystem(id)
    }

    /// Returns `true` if this is a label-based constraint.
    #[inline]
    pub fn is_label_based(&self) -> bool {
        matches!(
            self,
            LabeledOrderingConstraint::BeforeLabel(_) | LabeledOrderingConstraint::AfterLabel(_)
        )
    }

    /// Returns `true` if this is a system-based constraint.
    #[inline]
    pub fn is_system_based(&self) -> bool {
        matches!(
            self,
            LabeledOrderingConstraint::BeforeSystem(_) | LabeledOrderingConstraint::AfterSystem(_)
        )
    }
}

impl fmt::Display for LabeledOrderingConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabeledOrderingConstraint::BeforeLabel(label) => {
                write!(f, "before label '{}'", label.name())
            }
            LabeledOrderingConstraint::AfterLabel(label) => {
                write!(f, "after label '{}'", label.name())
            }
            LabeledOrderingConstraint::BeforeSystem(id) => {
                write!(f, "before system {}", id.raw())
            }
            LabeledOrderingConstraint::AfterSystem(id) => {
                write!(f, "after system {}", id.raw())
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/system_set_tests.rs"]
mod tests;
