//! Parallel system execution stage.
//!
//! `ParallelSystemStage` analyzes system access patterns to determine which
//! systems can safely run concurrently and groups them into batches.

use std::collections::HashMap;
use std::fmt;

use crate::ecs::system::{BoxedSystem, IntoSystem, SystemId};
use crate::ecs::World;

use super::core_stage::CoreStage;
use super::parallel_types::{
    ParallelBatch, ParallelExecutionConfig, ParallelExecutionStats, UnsafePtr,
};
use super::stage::Stage;
use super::stage_label::StageLabel;
use super::system_ordering::SystemOrdering;
use super::topological_sort::TopologicalSorter;

/// A stage that executes non-conflicting systems in parallel.
///
/// Analyzes system access patterns to determine which systems can safely
/// run concurrently. Groups non-conflicting systems into batches.
pub struct ParallelSystemStage {
    name: String,
    pub(super) systems: Vec<BoxedSystem>,
    pub(super) system_indices: HashMap<SystemId, usize>,
    initialized: bool,
    orderings: Vec<SystemOrdering>,
    dirty: bool,
    config: ParallelExecutionConfig,
    batches: Vec<ParallelBatch>,
    last_stats: ParallelExecutionStats,
}

impl ParallelSystemStage {
    /// Creates a new empty parallel stage.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_indices: HashMap::new(),
            initialized: false,
            orderings: Vec::new(),
            dirty: false,
            config: ParallelExecutionConfig::default(),
            batches: Vec::new(),
            last_stats: ParallelExecutionStats::default(),
        }
    }

    /// Creates a new parallel stage with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_indices: HashMap::with_capacity(capacity),
            initialized: false,
            orderings: Vec::with_capacity(capacity),
            dirty: false,
            config: ParallelExecutionConfig::default(),
            batches: Vec::new(),
            last_stats: ParallelExecutionStats::default(),
        }
    }

    /// Creates a new parallel stage from a `CoreStage` variant.
    #[inline]
    pub fn from_core(core_stage: CoreStage) -> Self {
        Self::new(core_stage.label_name())
    }

    /// Creates a new parallel stage with custom configuration.
    #[inline]
    pub fn with_config(name: impl Into<String>, config: ParallelExecutionConfig) -> Self {
        let mut stage = Self::new(name);
        stage.config = config;
        stage
    }

    /// Returns a reference to the current configuration.
    #[inline]
    pub fn config(&self) -> &ParallelExecutionConfig {
        &self.config
    }

    /// Returns a mutable reference to the configuration.
    #[inline]
    pub fn config_mut(&mut self) -> &mut ParallelExecutionConfig {
        self.dirty = true;
        &mut self.config
    }

    /// Sets the parallel execution configuration.
    #[inline]
    pub fn set_config(&mut self, config: ParallelExecutionConfig) {
        self.config = config;
        self.dirty = true;
    }

    /// Returns execution statistics from the last run.
    #[inline]
    pub fn last_stats(&self) -> &ParallelExecutionStats {
        &self.last_stats
    }

    /// Returns the pre-computed batches.
    #[inline]
    pub fn batches(&self) -> &[ParallelBatch] {
        &self.batches
    }

    /// Returns the number of batches.
    #[inline]
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Adds a system to this stage.
    pub fn add_system<S, Marker>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<Marker>,
    {
        let boxed = system.into_system();
        let id = boxed.id();
        let index = self.systems.len();
        self.system_indices.insert(id, index);
        self.systems.push(boxed);
        self.dirty = true;
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
            self.dirty = true;
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

    /// Returns an iterator over all system IDs in this stage.
    #[inline]
    pub fn system_ids(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().map(|s| s.id())
    }

    /// Returns an iterator over all systems in this stage.
    #[inline]
    pub fn systems(&self) -> impl Iterator<Item = &BoxedSystem> {
        self.systems.iter()
    }

    /// Returns a mutable iterator over all systems in this stage.
    #[inline]
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut BoxedSystem> {
        self.systems.iter_mut()
    }

    /// Returns whether this stage has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Marks the stage as uninitialized.
    #[inline]
    pub fn reset_initialized(&mut self) {
        self.initialized = false;
    }

    /// Clears all systems from this stage.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_indices.clear();
        self.orderings.clear();
        self.batches.clear();
        self.initialized = false;
        self.dirty = false;
    }

    /// Returns system names for debugging.
    pub fn system_names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

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
        self.dirty = true;
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
            self.dirty = true;
        }
        removed
    }

    /// Clears all ordering constraints.
    pub fn clear_orderings(&mut self) {
        if !self.orderings.is_empty() {
            self.orderings.clear();
            self.dirty = true;
        }
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

    /// Rebuilds the parallel execution batches.
    pub fn rebuild_batches(&mut self) -> Result<(), super::topological_sort::OrderingCycleError> {
        self.batches.clear();
        if self.systems.is_empty() {
            self.dirty = false;
            return Ok(());
        }
        let execution_order = if self.config.respect_ordering && !self.orderings.is_empty() {
            let mut sorter =
                TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len());
            for system in &self.systems {
                sorter.add_system(system.id(), system.name());
            }
            for ordering in &self.orderings {
                sorter.add_system_ordering(*ordering);
            }
            sorter.sort()?
        } else {
            self.systems.iter().map(|s| s.id()).collect()
        };
        let mut direct_predecessors: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
        if self.config.respect_ordering {
            for ordering in &self.orderings {
                direct_predecessors
                    .entry(ordering.second())
                    .or_default()
                    .push(ordering.first());
            }
        }
        let mut system_batch_index: HashMap<SystemId, usize> = HashMap::new();
        for system_id in execution_order {
            let system_idx = self.system_indices[&system_id];
            let system = &self.systems[system_idx];
            let system_access = system.component_access();
            let system_read_only = system.is_read_only();
            let min_batch_idx = if self.config.respect_ordering {
                direct_predecessors
                    .get(&system_id)
                    .map(|preds| {
                        preds
                            .iter()
                            .filter_map(|pred| system_batch_index.get(pred))
                            .max()
                            .map(|&max| max + 1)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0)
            } else {
                0
            };
            let mut assigned = false;
            for batch_idx in min_batch_idx..self.batches.len() {
                let batch = &self.batches[batch_idx];
                let has_conflict = batch.system_ids.iter().any(|&bid| {
                    let bidx = self.system_indices[&bid];
                    system_access.conflicts_with(&self.systems[bidx].component_access())
                });
                if !has_conflict {
                    self.batches[batch_idx].add(system_id, system_read_only);
                    system_batch_index.insert(system_id, batch_idx);
                    assigned = true;
                    break;
                }
            }
            if !assigned {
                let new_batch_idx = self.batches.len();
                let mut batch = ParallelBatch::with_capacity(4);
                batch.add(system_id, system_read_only);
                self.batches.push(batch);
                system_batch_index.insert(system_id, new_batch_idx);
            }
        }
        self.dirty = false;
        Ok(())
    }

    /// Forces a rebuild of parallel batches on next run.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Returns whether batches need to be rebuilt.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Runs all systems using parallel execution.
    pub fn run_parallel(&mut self, world: &mut World) {
        if (self.dirty || self.batches.is_empty()) && self.config.auto_rebuild {
            let _ = self.rebuild_batches();
        }
        if !self.initialized {
            for system in &mut self.systems {
                system.initialize(world);
            }
            self.initialized = true;
        }
        let mut stats = ParallelExecutionStats {
            batch_count: self.batches.len(),
            system_count: self.systems.len(),
            ..Default::default()
        };
        for batch in &self.batches {
            if batch.is_empty() {
                continue;
            }
            let runnable: Vec<usize> = batch
                .system_ids
                .iter()
                .filter_map(|&id| {
                    let idx = self.system_indices[&id];
                    if self.systems[idx].should_run(world) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if runnable.is_empty() {
                continue;
            }
            if runnable.len() == 1 {
                self.systems[runnable[0]].run(world);
                stats.sequential_systems += 1;
            } else {
                stats.parallel_systems += runnable.len();
                if runnable.len() > stats.max_parallelism {
                    stats.max_parallelism = runnable.len();
                }
                #[cfg(feature = "native")]
                {
                    // SAFETY: Batch computation ensures non-conflicting access.
                    let systems_ptr = UnsafePtr(self.systems.as_mut_ptr());
                    let world_ptr = UnsafePtr(world as *mut World);
                    rayon::scope(|s| {
                        for &idx in &runnable {
                            s.spawn(move |_| {
                                // SAFETY: Each system accesses disjoint data.
                                unsafe {
                                    let sys = &mut *systems_ptr.get().add(idx);
                                    let w = &mut *world_ptr.get();
                                    sys.run(w);
                                }
                            });
                        }
                    });
                }
                #[cfg(not(feature = "native"))]
                {
                    for &idx in &runnable {
                        self.systems[idx].run(world);
                    }
                }
            }
        }
        self.last_stats = stats;
    }
}

impl Stage for ParallelSystemStage {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, world: &mut World) {
        self.run_parallel(world);
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

impl Default for ParallelSystemStage {
    fn default() -> Self {
        Self::new("ParallelStage")
    }
}

impl fmt::Debug for ParallelSystemStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParallelSystemStage")
            .field("name", &self.name)
            .field("system_count", &self.systems.len())
            .field("batch_count", &self.batches.len())
            .field("initialized", &self.initialized)
            .field("dirty", &self.dirty)
            .field("config", &self.config)
            .field("systems", &self.system_names())
            .finish()
    }
}

// SAFETY: Same reasoning as SystemStage, plus parallel execution
// is safe due to non-conflicting access patterns in batches.
unsafe impl Send for ParallelSystemStage {}
unsafe impl Sync for ParallelSystemStage {}

#[cfg(test)]
#[path = "tests/parallel_tests.rs"]
mod tests;
