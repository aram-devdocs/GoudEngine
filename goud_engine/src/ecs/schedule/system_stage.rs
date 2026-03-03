//! Sequential system execution stage.

use std::collections::HashMap;
use std::fmt;

use crate::ecs::system::{BoxedSystem, IntoSystem, SystemId};
use crate::ecs::World;

use super::core_stage::CoreStage;
use super::stage::Stage;
use super::stage_label::StageLabel;
use super::system_ordering::SystemOrdering;
use super::system_set::ChainedSystems;
use super::topological_sort::TopologicalSorter;

/// A container that holds and runs systems sequentially.
///
/// `SystemStage` is the primary implementation of [`Stage`]. It stores systems
/// in a vector and runs them in the order they were added (or as reordered
/// by ordering constraints).
pub struct SystemStage {
    /// Human-readable name of this stage.
    name: String,
    /// Systems to run in this stage, in order.
    pub(super) systems: Vec<BoxedSystem>,
    /// Map from system ID to index for fast lookup.
    pub(super) system_indices: HashMap<SystemId, usize>,
    /// Whether the stage has been initialized.
    initialized: bool,
    /// Ordering constraints between systems.
    orderings: Vec<SystemOrdering>,
    /// Whether the system order needs to be rebuilt.
    order_dirty: bool,
}

impl SystemStage {
    /// Creates a new empty stage with the given name.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_indices: HashMap::new(),
            initialized: false,
            orderings: Vec::new(),
            order_dirty: false,
        }
    }

    /// Creates a new stage with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_indices: HashMap::with_capacity(capacity),
            initialized: false,
            orderings: Vec::with_capacity(capacity),
            order_dirty: false,
        }
    }

    /// Creates a new stage from a `CoreStage` variant.
    #[inline]
    pub fn from_core(core_stage: CoreStage) -> Self {
        Self::new(core_stage.label_name())
    }

    /// Adds a system to this stage. Returns the assigned `SystemId`.
    pub fn add_system<S, Marker>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<Marker>,
    {
        let boxed = system.into_system();
        let id = boxed.id();
        let index = self.systems.len();
        self.system_indices.insert(id, index);
        self.systems.push(boxed);
        id
    }

    /// Removes a system from this stage by its ID.
    pub fn remove_system(&mut self, id: SystemId) -> bool {
        if let Some(index) = self.system_indices.remove(&id) {
            self.systems.remove(index);
            self.system_indices.clear();
            for (i, system) in self.systems.iter().enumerate() {
                self.system_indices.insert(system.id(), i);
            }
            true
        } else {
            false
        }
    }

    /// Returns a reference to a system by its ID.
    #[inline]
    pub fn get_system(&self, id: SystemId) -> Option<&BoxedSystem> {
        self.system_indices.get(&id).map(|&i| &self.systems[i])
    }

    /// Returns a mutable reference to a system by its ID.
    #[inline]
    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut BoxedSystem> {
        self.system_indices
            .get(&id)
            .copied()
            .map(|i| &mut self.systems[i])
    }

    /// Returns whether the stage contains a system with the given ID.
    #[inline]
    pub fn contains_system(&self, id: SystemId) -> bool {
        self.system_indices.contains_key(&id)
    }

    /// Returns an iterator over all system IDs.
    #[inline]
    pub fn system_ids(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().map(|s| s.id())
    }

    /// Returns an iterator over all systems.
    #[inline]
    pub fn systems(&self) -> impl Iterator<Item = &BoxedSystem> {
        self.systems.iter()
    }

    /// Returns a mutable iterator over all systems.
    #[inline]
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut BoxedSystem> {
        self.systems.iter_mut()
    }

    /// Returns whether this stage has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Marks the stage as uninitialized, forcing re-initialization on next run.
    #[inline]
    pub fn reset_initialized(&mut self) {
        self.initialized = false;
    }

    /// Clears all systems from this stage.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_indices.clear();
        self.orderings.clear();
        self.initialized = false;
        self.order_dirty = false;
    }

    /// Returns system names for debugging.
    pub fn system_names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

    /// Runs a single system by ID on the given world.
    pub fn run_system(&mut self, id: SystemId, world: &mut World) -> Option<bool> {
        if let Some(&index) = self.system_indices.get(&id) {
            let system = &mut self.systems[index];
            if system.should_run(world) {
                system.run(world);
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }

    // =====================================================================
    // Ordering API
    // =====================================================================

    /// Adds an ordering constraint: `first` must run before `second`.
    pub fn add_ordering(&mut self, first: SystemId, second: SystemId) -> bool {
        if !self.contains_system(first) || !self.contains_system(second) {
            return false;
        }
        if first == second {
            return false;
        }
        let ordering = SystemOrdering::before(first, second);
        if self.orderings.contains(&ordering) {
            return true;
        }
        self.orderings.push(ordering);
        self.order_dirty = true;
        true
    }

    /// Adds a constraint that `system` must run before `other`.
    #[inline]
    pub fn set_before(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(system, other)
    }

    /// Adds a constraint that `system` must run after `other`.
    #[inline]
    pub fn set_after(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(other, system)
    }

    /// Removes all ordering constraints involving the given system.
    pub fn remove_orderings_for(&mut self, system: SystemId) -> usize {
        let before_len = self.orderings.len();
        self.orderings.retain(|o| !o.involves(system));
        let removed = before_len - self.orderings.len();
        if removed > 0 {
            self.order_dirty = true;
        }
        removed
    }

    /// Clears all ordering constraints.
    pub fn clear_orderings(&mut self) {
        if !self.orderings.is_empty() {
            self.orderings.clear();
            self.order_dirty = true;
        }
    }

    /// Chains multiple systems for strict sequential execution.
    pub fn chain_systems<I>(&mut self, systems: I) -> usize
    where
        I: IntoIterator<Item = SystemId>,
    {
        let orderings = super::system_set::chain(systems);
        let count = orderings.len();
        for ordering in orderings {
            let (first, second) = ordering.as_edge();
            self.add_ordering(first, second);
        }
        count
    }

    /// Adds all orderings from a ChainedSystems.
    pub fn add_chain(&mut self, chained: &ChainedSystems) -> usize {
        let orderings = chained.to_orderings();
        let count = orderings.len();
        for ordering in orderings {
            let (first, second) = ordering.as_edge();
            self.add_ordering(first, second);
        }
        count
    }

    /// Returns an iterator over all ordering constraints.
    #[inline]
    pub fn orderings(&self) -> impl Iterator<Item = &SystemOrdering> {
        self.orderings.iter()
    }

    /// Returns the number of ordering constraints.
    #[inline]
    pub fn ordering_count(&self) -> usize {
        self.orderings.len()
    }

    /// Returns whether the system order needs to be rebuilt.
    #[inline]
    pub fn is_order_dirty(&self) -> bool {
        self.order_dirty
    }

    /// Rebuilds the system execution order based on ordering constraints.
    pub fn rebuild_order(&mut self) -> Result<(), super::topological_sort::OrderingCycleError> {
        if self.orderings.is_empty() {
            self.order_dirty = false;
            return Ok(());
        }
        let mut sorter = TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len());
        for system in &self.systems {
            sorter.add_system(system.id(), system.name());
        }
        for ordering in &self.orderings {
            sorter.add_system_ordering(*ordering);
        }
        let sorted_ids = sorter.sort()?;
        let mut new_systems = Vec::with_capacity(self.systems.len());
        let mut new_indices = HashMap::with_capacity(self.systems.len());
        let mut system_map: HashMap<SystemId, BoxedSystem> =
            self.systems.drain(..).map(|s| (s.id(), s)).collect();
        for id in sorted_ids {
            if let Some(system) = system_map.remove(&id) {
                new_indices.insert(id, new_systems.len());
                new_systems.push(system);
            }
        }
        for (id, system) in system_map {
            new_indices.insert(id, new_systems.len());
            new_systems.push(system);
        }
        self.systems = new_systems;
        self.system_indices = new_indices;
        self.order_dirty = false;
        Ok(())
    }

    /// Checks if adding an ordering would create a cycle.
    pub fn would_ordering_cycle(&self, first: SystemId, second: SystemId) -> bool {
        let mut sorter =
            TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len() + 1);
        for system in &self.systems {
            sorter.add_system(system.id(), system.name());
        }
        for ordering in &self.orderings {
            sorter.add_system_ordering(*ordering);
        }
        sorter.add_ordering(first, second);
        sorter.would_cycle()
    }

    /// Returns the orderings involving a specific system.
    pub fn orderings_for(&self, system: SystemId) -> Vec<&SystemOrdering> {
        self.orderings
            .iter()
            .filter(|o| o.involves(system))
            .collect()
    }
}

impl Stage for SystemStage {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, world: &mut World) {
        if self.order_dirty {
            let _ = self.rebuild_order();
        }
        if !self.initialized {
            self.initialize(world);
        }
        for system in &mut self.systems {
            if system.should_run(world) {
                system.run(world);
            }
        }
    }

    fn initialize(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.initialize(world);
        }
        self.initialized = true;
    }

    #[inline]
    fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl Default for SystemStage {
    fn default() -> Self {
        Self::new("DefaultStage")
    }
}

impl fmt::Debug for SystemStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemStage")
            .field("name", &self.name)
            .field("system_count", &self.systems.len())
            .field("initialized", &self.initialized)
            .field("ordering_count", &self.orderings.len())
            .field("order_dirty", &self.order_dirty)
            .field("systems", &self.system_names())
            .finish()
    }
}

// SAFETY: BoxedSystem contains Box<dyn System> where System: Send.
// The stage requires &mut self to run, so there's no concurrent access.
unsafe impl Send for SystemStage {}
unsafe impl Sync for SystemStage {}

#[cfg(test)]
#[path = "tests/system_stage_tests.rs"]
mod tests;
