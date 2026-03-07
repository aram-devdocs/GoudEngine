//! [`QueryArchetypeCache`] — caches matching archetype indices between frames.
//!
//! Instead of scanning all archetypes every iteration, queries can cache which
//! archetype indices matched. Because archetypes are append-only (never removed),
//! the cache only needs to evaluate newly added archetypes on update.

use crate::ecs::archetype::ArchetypeId;

/// Cached set of archetype indices that match a particular query.
///
/// The cache tracks a *generation* equal to the archetype count when it was
/// last refreshed. On update, only archetypes with indices in
/// `[generation..current_len)` are evaluated, making incremental updates O(new)
/// rather than O(all).
///
/// # Invariant
///
/// Archetypes are never removed from the graph, so cached indices remain valid
/// for the lifetime of the [`World`](crate::ecs::World).
#[derive(Debug, Clone)]
pub struct QueryArchetypeCache {
    /// Indices of archetypes that matched the query.
    archetype_indices: Vec<usize>,
    /// Number of archetypes in the graph when this cache was last built/updated.
    /// Acts as a generation counter for incremental updates.
    archetype_generation: usize,
}

impl QueryArchetypeCache {
    /// Creates a new empty cache with generation 0.
    #[inline]
    pub fn new() -> Self {
        Self {
            archetype_indices: Vec::new(),
            archetype_generation: 0,
        }
    }

    /// Returns the cached archetype indices.
    #[inline]
    pub fn archetype_indices(&self) -> &[usize] {
        &self.archetype_indices
    }

    /// Returns the generation (archetype count) when this cache was last updated.
    #[inline]
    pub fn generation(&self) -> usize {
        self.archetype_generation
    }

    /// Returns `true` if the cache is up-to-date with the given archetype count.
    #[inline]
    pub fn is_current(&self, archetype_count: usize) -> bool {
        self.archetype_generation == archetype_count
    }

    /// Updates the cache by evaluating archetypes from `generation..archetype_count`.
    ///
    /// The `matcher` closure receives an [`ArchetypeId`] and returns `true` if
    /// the archetype at that index matches the query.
    ///
    /// Only newly added archetypes (since the last update) are evaluated.
    /// If the cache is already current, this is a no-op.
    pub fn update<F>(&mut self, archetype_count: usize, matcher: F)
    where
        F: Fn(ArchetypeId) -> bool,
    {
        if self.archetype_generation >= archetype_count {
            return;
        }

        for i in self.archetype_generation..archetype_count {
            let id = ArchetypeId::new(i as u32);
            if matcher(id) {
                self.archetype_indices.push(i);
            }
        }

        self.archetype_generation = archetype_count;
    }

    /// Resets the cache, forcing a full re-evaluation on the next update.
    #[inline]
    pub fn invalidate(&mut self) {
        self.archetype_indices.clear();
        self.archetype_generation = 0;
    }
}

impl Default for QueryArchetypeCache {
    fn default() -> Self {
        Self::new()
    }
}
