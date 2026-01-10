//! Broad phase collision detection using spatial hashing.
//!
//! The broad phase is the first step of collision detection. It efficiently identifies
//! pairs of objects that *might* be colliding, filtering out objects that are too far
//! apart to possibly collide. The narrow phase then performs precise collision tests
//! on these candidate pairs.
//!
//! # Spatial Hash
//!
//! A spatial hash divides space into a uniform grid of cells. Each object is assigned
//! to one or more cells based on its AABB. Objects in the same cell are potential
//! collision pairs.
//!
//! Benefits:
//! - O(1) insertion and removal
//! - O(n) query for nearby objects (where n = objects per cell)
//! - Simple implementation
//! - Good cache locality
//! - Predictable performance
//!
//! Trade-offs:
//! - Struggles with objects of vastly different sizes
//! - Uniform grid doesn't adapt to object distribution
//! - Memory usage proportional to covered area
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::broad_phase::SpatialHash;
//! use goud_engine::ecs::Entity;
//! use goud_engine::core::math::Rect;
//!
//! // Create spatial hash with 64-pixel cells
//! let mut hash = SpatialHash::new(64.0);
//!
//! // Insert entities with their AABBs
//! let entity = Entity::new(0, 0);
//! let aabb = Rect::new(0.0, 0.0, 32.0, 32.0);
//! hash.insert(entity, aabb);
//!
//! // Query for potential collisions
//! let pairs = hash.query_pairs();
//! for (a, b) in pairs {
//!     // Perform narrow phase collision test on (a, b)
//! }
//! ```

use crate::core::math::{Rect, Vec2};
use crate::ecs::Entity;
use std::collections::{HashMap, HashSet};

// =============================================================================
// CellCoord - Grid Cell Coordinate
// =============================================================================

/// A coordinate in the spatial hash grid.
///
/// Cells are indexed by integer (x, y) coordinates. The hash function maps
/// these coordinates to hash buckets for efficient storage.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct CellCoord {
    x: i32,
    y: i32,
}

impl CellCoord {
    /// Creates a new cell coordinate.
    #[inline]
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts a world position to a cell coordinate.
    #[inline]
    fn from_world(pos: Vec2, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
        }
    }
}

// =============================================================================
// SpatialHash - Broad Phase Collision Detection
// =============================================================================

/// Spatial hash for broad phase collision detection.
///
/// Divides 2D space into a uniform grid of cells. Entities are mapped to cells
/// based on their AABBs. This enables O(1) insertion/removal and efficient
/// querying of potential collision pairs.
///
/// # Performance
///
/// - Insertion: O(k) where k = number of cells occupied by AABB
/// - Removal: O(k)
/// - Query pairs: O(n^2) where n = average entities per cell
/// - Total query: O(m * n^2) where m = number of occupied cells
///
/// # Cell Size Selection
///
/// The cell size should be roughly equal to the average object size:
/// - Too small: Objects span many cells, increasing overhead
/// - Too large: Too many objects per cell, increasing pair checks
///
/// For games with mixed object sizes, use the size of the most common objects.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::broad_phase::SpatialHash;
/// use goud_engine::ecs::Entity;
/// use goud_engine::core::math::Rect;
///
/// let mut hash = SpatialHash::new(64.0);
///
/// // Insert entities
/// let player = Entity::new(0, 0);
/// let enemy = Entity::new(1, 0);
///
/// hash.insert(player, Rect::new(0.0, 0.0, 32.0, 32.0));
/// hash.insert(enemy, Rect::new(50.0, 0.0, 32.0, 32.0));
///
/// // Query potential collisions
/// let pairs = hash.query_pairs();
/// assert_eq!(pairs.len(), 1);  // Player and enemy are close
/// ```
#[derive(Clone, Debug)]
pub struct SpatialHash {
    /// Size of each grid cell (world units).
    cell_size: f32,

    /// Inverse of cell size for faster division.
    #[allow(dead_code)]
    cell_size_inv: f32,

    /// Grid cells: each cell contains a list of entities.
    /// Key is the cell coordinate, value is the set of entities in that cell.
    grid: HashMap<CellCoord, HashSet<Entity>>,

    /// Entity to AABB mapping for updates.
    /// Stores the AABB for each entity to know which cells it occupies.
    entity_bounds: HashMap<Entity, Rect>,

    /// Entity to cell list mapping.
    /// Stores which cells each entity occupies for efficient removal.
    entity_cells: HashMap<Entity, Vec<CellCoord>>,

    /// Statistics for debugging and profiling.
    stats: SpatialHashStats,
}

/// Statistics for spatial hash performance analysis.
#[derive(Clone, Debug, Default)]
pub struct SpatialHashStats {
    /// Total number of entities in the hash.
    pub entity_count: usize,

    /// Number of occupied cells.
    pub cell_count: usize,

    /// Total number of entity-cell mappings.
    pub total_cell_entries: usize,

    /// Maximum entities in a single cell.
    pub max_entities_per_cell: usize,

    /// Average entities per occupied cell.
    pub avg_entities_per_cell: f32,

    /// Number of potential collision pairs found in last query.
    pub last_query_pairs: usize,
}

impl SpatialHash {
    // =========================================================================
    // Construction
    // =========================================================================

    /// Creates a new spatial hash with the specified cell size.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - Size of each grid cell in world units (e.g., pixels)
    ///
    /// # Panics
    ///
    /// Panics if `cell_size` is not positive and finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    ///
    /// // 64-pixel cells (good for medium-sized objects)
    /// let hash = SpatialHash::new(64.0);
    ///
    /// // 128-pixel cells (larger objects)
    /// let hash = SpatialHash::new(128.0);
    /// ```
    pub fn new(cell_size: f32) -> Self {
        assert!(
            cell_size > 0.0 && cell_size.is_finite(),
            "Cell size must be positive and finite"
        );

        Self {
            cell_size,
            cell_size_inv: 1.0 / cell_size,
            grid: HashMap::new(),
            entity_bounds: HashMap::new(),
            entity_cells: HashMap::new(),
            stats: SpatialHashStats::default(),
        }
    }

    /// Creates a spatial hash with the specified cell size and initial capacity.
    ///
    /// Pre-allocates memory for the specified number of entities, reducing
    /// allocations during insertion.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - Size of each grid cell
    /// * `capacity` - Expected number of entities
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    ///
    /// // Pre-allocate for 1000 entities
    /// let hash = SpatialHash::with_capacity(64.0, 1000);
    /// ```
    pub fn with_capacity(cell_size: f32, capacity: usize) -> Self {
        assert!(
            cell_size > 0.0 && cell_size.is_finite(),
            "Cell size must be positive and finite"
        );

        Self {
            cell_size,
            cell_size_inv: 1.0 / cell_size,
            grid: HashMap::with_capacity(capacity),
            entity_bounds: HashMap::with_capacity(capacity),
            entity_cells: HashMap::with_capacity(capacity),
            stats: SpatialHashStats::default(),
        }
    }

    // =========================================================================
    // Insertion and Removal
    // =========================================================================

    /// Inserts an entity with its AABB into the spatial hash.
    ///
    /// The entity is added to all cells that its AABB overlaps. If the entity
    /// was already in the hash, it is first removed before re-insertion.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to insert
    /// * `aabb` - The entity's axis-aligned bounding box
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    /// let aabb = Rect::new(0.0, 0.0, 32.0, 32.0);
    ///
    /// hash.insert(entity, aabb);
    /// assert!(hash.contains(entity));
    /// ```
    pub fn insert(&mut self, entity: Entity, aabb: Rect) {
        // Remove if already present
        if self.entity_bounds.contains_key(&entity) {
            self.remove(entity);
        }

        // Calculate which cells this AABB overlaps
        let cells = self.get_cells_for_aabb(aabb);

        // Insert entity into each cell
        for cell in &cells {
            self.grid.entry(*cell).or_default().insert(entity);
        }

        // Store mappings
        self.entity_bounds.insert(entity, aabb);
        self.entity_cells.insert(entity, cells);

        // Update stats
        self.update_stats();
    }

    /// Removes an entity from the spatial hash.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove
    ///
    /// # Returns
    ///
    /// `true` if the entity was present and removed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    /// assert!(hash.remove(entity));
    /// assert!(!hash.contains(entity));
    /// ```
    pub fn remove(&mut self, entity: Entity) -> bool {
        // Get the cells this entity occupies
        let cells = match self.entity_cells.remove(&entity) {
            Some(cells) => cells,
            None => return false, // Entity not in hash
        };

        // Remove from each cell
        for cell in cells {
            if let Some(cell_entities) = self.grid.get_mut(&cell) {
                cell_entities.remove(&entity);

                // Remove empty cells to save memory
                if cell_entities.is_empty() {
                    self.grid.remove(&cell);
                }
            }
        }

        // Remove entity bounds
        self.entity_bounds.remove(&entity);

        // Update stats
        self.update_stats();

        true
    }

    /// Updates an entity's AABB in the spatial hash.
    ///
    /// This is more efficient than remove + insert when the entity hasn't
    /// moved much, as it can avoid cell changes.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to update
    /// * `new_aabb` - The entity's new AABB
    ///
    /// # Returns
    ///
    /// `true` if the entity was present and updated, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    /// hash.update(entity, Rect::new(10.0, 10.0, 32.0, 32.0));
    /// ```
    pub fn update(&mut self, entity: Entity, new_aabb: Rect) -> bool {
        if !self.entity_bounds.contains_key(&entity) {
            return false;
        }

        // Calculate new cells
        let new_cells = self.get_cells_for_aabb(new_aabb);
        let old_cells = self.entity_cells.get(&entity).unwrap();

        // Optimization: if cells haven't changed, just update the AABB
        if new_cells.len() == old_cells.len() && new_cells.iter().all(|c| old_cells.contains(c)) {
            self.entity_bounds.insert(entity, new_aabb);
            return true;
        }

        // Cells changed, do full update
        self.remove(entity);
        self.insert(entity, new_aabb);

        true
    }

    /// Clears all entities from the spatial hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// hash.insert(Entity::new(0, 0), Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// hash.clear();
    /// assert_eq!(hash.entity_count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.grid.clear();
        self.entity_bounds.clear();
        self.entity_cells.clear();
        self.stats = SpatialHashStats::default();
    }

    // =========================================================================
    // Queries
    // =========================================================================

    /// Queries for all potential collision pairs.
    ///
    /// Returns a list of entity pairs that share at least one cell. These are
    /// candidates for narrow phase collision testing.
    ///
    /// # Returns
    ///
    /// A vector of (Entity, Entity) pairs where the first entity has a lower
    /// index than the second (no duplicate pairs).
    ///
    /// # Performance
    ///
    /// O(m * n^2) where m = number of occupied cells, n = average entities per cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    ///
    /// let e1 = Entity::new(0, 0);
    /// let e2 = Entity::new(1, 0);
    ///
    /// hash.insert(e1, Rect::new(0.0, 0.0, 32.0, 32.0));
    /// hash.insert(e2, Rect::new(30.0, 0.0, 32.0, 32.0));
    ///
    /// let pairs = hash.query_pairs();
    /// assert_eq!(pairs.len(), 1);  // Overlapping cells
    /// ```
    pub fn query_pairs(&mut self) -> Vec<(Entity, Entity)> {
        let mut pairs = Vec::new();
        let mut seen = HashSet::new();

        // For each occupied cell
        for cell_entities in self.grid.values() {
            // Generate pairs within the cell
            let entities: Vec<_> = cell_entities.iter().copied().collect();

            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    let a = entities[i];
                    let b = entities[j];

                    // Ensure consistent ordering (smaller entity first)
                    let pair = if a.to_bits() < b.to_bits() {
                        (a, b)
                    } else {
                        (b, a)
                    };

                    // Avoid duplicate pairs (entities can share multiple cells)
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }

        self.stats.last_query_pairs = pairs.len();
        pairs
    }

    /// Queries for entities near a specific point.
    ///
    /// Returns all entities whose AABBs occupy the same cell as the point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::{Rect, Vec2};
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// let nearby = hash.query_point(Vec2::new(10.0, 10.0));
    /// assert_eq!(nearby.len(), 1);
    /// ```
    pub fn query_point(&self, point: Vec2) -> Vec<Entity> {
        let cell = CellCoord::from_world(point, self.cell_size);

        self.grid
            .get(&cell)
            .map(|entities| entities.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Queries for entities overlapping an AABB.
    ///
    /// Returns all entities whose AABBs occupy any cell overlapped by the query AABB.
    ///
    /// # Arguments
    ///
    /// * `aabb` - The query AABB
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// let nearby = hash.query_aabb(Rect::new(-10.0, -10.0, 50.0, 50.0));
    /// assert_eq!(nearby.len(), 1);
    /// ```
    pub fn query_aabb(&self, aabb: Rect) -> Vec<Entity> {
        let cells = self.get_cells_for_aabb(aabb);
        let mut result = HashSet::new();

        for cell in cells {
            if let Some(cell_entities) = self.grid.get(&cell) {
                result.extend(cell_entities.iter().copied());
            }
        }

        result.into_iter().collect()
    }

    /// Queries for entities overlapping a circle.
    ///
    /// Returns all entities in cells that overlap the circle's bounding square.
    /// This is a conservative estimate - narrow phase should verify actual overlap.
    ///
    /// # Arguments
    ///
    /// * `center` - Circle center position
    /// * `radius` - Circle radius
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::{Rect, Vec2};
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    ///
    /// let nearby = hash.query_circle(Vec2::new(16.0, 16.0), 20.0);
    /// assert_eq!(nearby.len(), 1);
    /// ```
    pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        // Use bounding square for conservative estimate
        let aabb = Rect::new(
            center.x - radius,
            center.y - radius,
            radius * 2.0,
            radius * 2.0,
        );

        self.query_aabb(aabb)
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns the cell size.
    #[inline]
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Returns the number of entities in the hash.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entity_bounds.len()
    }

    /// Returns the number of occupied cells.
    #[inline]
    pub fn cell_count(&self) -> usize {
        self.grid.len()
    }

    /// Returns whether the hash is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_bounds.is_empty()
    }

    /// Returns whether an entity is in the hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::broad_phase::SpatialHash;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Rect;
    ///
    /// let mut hash = SpatialHash::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// assert!(!hash.contains(entity));
    /// hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
    /// assert!(hash.contains(entity));
    /// ```
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_bounds.contains_key(&entity)
    }

    /// Returns the AABB for an entity, if present.
    #[inline]
    pub fn get_aabb(&self, entity: Entity) -> Option<Rect> {
        self.entity_bounds.get(&entity).copied()
    }

    /// Returns a reference to the current statistics.
    #[inline]
    pub fn stats(&self) -> &SpatialHashStats {
        &self.stats
    }

    // =========================================================================
    // Internal Helpers
    // =========================================================================

    /// Gets all cell coordinates that an AABB overlaps.
    fn get_cells_for_aabb(&self, aabb: Rect) -> Vec<CellCoord> {
        let mut cells = Vec::new();

        // Calculate cell range
        let min = aabb.min();
        let max = aabb.max();

        let min_cell = CellCoord::from_world(min, self.cell_size);
        let max_cell = CellCoord::from_world(max, self.cell_size);

        // Iterate over cell range
        for y in min_cell.y..=max_cell.y {
            for x in min_cell.x..=max_cell.x {
                cells.push(CellCoord::new(x, y));
            }
        }

        cells
    }

    /// Updates statistics after modification.
    fn update_stats(&mut self) {
        self.stats.entity_count = self.entity_bounds.len();
        self.stats.cell_count = self.grid.len();

        // Count total cell entries and find max
        let mut total = 0;
        let mut max = 0;

        for cell_entities in self.grid.values() {
            let count = cell_entities.len();
            total += count;
            max = max.max(count);
        }

        self.stats.total_cell_entries = total;
        self.stats.max_entities_per_cell = max;
        self.stats.avg_entities_per_cell = if self.stats.cell_count > 0 {
            total as f32 / self.stats.cell_count as f32
        } else {
            0.0
        };
    }
}

// =============================================================================
// Display Implementations
// =============================================================================

impl std::fmt::Display for SpatialHashStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpatialHash Stats:\n\
             - Entities: {}\n\
             - Cells: {}\n\
             - Total entries: {}\n\
             - Max per cell: {}\n\
             - Avg per cell: {:.2}\n\
             - Last query pairs: {}",
            self.entity_count,
            self.cell_count,
            self.total_cell_entries,
            self.max_entities_per_cell,
            self.avg_entities_per_cell,
            self.last_query_pairs
        )
    }
}

impl std::fmt::Display for SpatialHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpatialHash(cell_size: {:.1}, entities: {}, cells: {})",
            self.cell_size,
            self.entity_count(),
            self.cell_count()
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test entities
    fn entity(id: u32) -> Entity {
        Entity::new(id, 0)
    }

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_new() {
        let hash = SpatialHash::new(64.0);
        assert_eq!(hash.cell_size(), 64.0);
        assert_eq!(hash.entity_count(), 0);
        assert_eq!(hash.cell_count(), 0);
        assert!(hash.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let hash = SpatialHash::with_capacity(64.0, 100);
        assert_eq!(hash.cell_size(), 64.0);
        assert!(hash.is_empty());
    }

    #[test]
    #[should_panic(expected = "Cell size must be positive and finite")]
    fn test_new_invalid_cell_size() {
        let _ = SpatialHash::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Cell size must be positive and finite")]
    fn test_new_negative_cell_size() {
        let _ = SpatialHash::new(-10.0);
    }

    // =========================================================================
    // Insertion and Removal Tests
    // =========================================================================

    #[test]
    fn test_insert_single() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);
        let aabb = Rect::new(0.0, 0.0, 32.0, 32.0);

        hash.insert(entity, aabb);

        assert_eq!(hash.entity_count(), 1);
        assert!(hash.contains(entity));
        assert_eq!(hash.get_aabb(entity), Some(aabb));
    }

    #[test]
    fn test_insert_multiple() {
        let mut hash = SpatialHash::new(64.0);

        for i in 0..10 {
            let entity = entity(i);
            let aabb = Rect::new(i as f32 * 100.0, 0.0, 32.0, 32.0);
            hash.insert(entity, aabb);
        }

        assert_eq!(hash.entity_count(), 10);
    }

    #[test]
    fn test_insert_overwrites() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity, Rect::new(100.0, 100.0, 32.0, 32.0));

        assert_eq!(hash.entity_count(), 1);
        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(100.0, 100.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_remove_present() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.remove(entity));
        assert!(!hash.contains(entity));
        assert_eq!(hash.entity_count(), 0);
    }

    #[test]
    fn test_remove_absent() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        assert!(!hash.remove(entity));
    }

    #[test]
    fn test_remove_twice() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.remove(entity));
        assert!(!hash.remove(entity));
    }

    #[test]
    fn test_update_same_cells() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.update(entity, Rect::new(10.0, 10.0, 32.0, 32.0)));

        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(10.0, 10.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_update_different_cells() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.update(entity, Rect::new(100.0, 100.0, 32.0, 32.0)));

        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(100.0, 100.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_update_absent() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        assert!(!hash.update(entity, Rect::new(0.0, 0.0, 32.0, 32.0)));
    }

    #[test]
    fn test_clear() {
        let mut hash = SpatialHash::new(64.0);

        for i in 0..10 {
            hash.insert(entity(i), Rect::new(i as f32 * 10.0, 0.0, 32.0, 32.0));
        }

        hash.clear();
        assert_eq!(hash.entity_count(), 0);
        assert_eq!(hash.cell_count(), 0);
        assert!(hash.is_empty());
    }

    // =========================================================================
    // Query Tests
    // =========================================================================

    #[test]
    fn test_query_pairs_empty() {
        let mut hash = SpatialHash::new(64.0);
        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_query_pairs_single() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0); // No pairs with single entity
    }

    #[test]
    fn test_query_pairs_nearby() {
        let mut hash = SpatialHash::new(64.0);

        // Two entities in the same cell
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(30.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains(&(entity(0), entity(1))));
    }

    #[test]
    fn test_query_pairs_far_apart() {
        let mut hash = SpatialHash::new(64.0);

        // Two entities in different cells
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(200.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0); // Too far apart
    }

    #[test]
    fn test_query_pairs_multiple() {
        let mut hash = SpatialHash::new(64.0);

        // Three entities in same cell
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(20.0, 0.0, 32.0, 32.0));
        hash.insert(entity(2), Rect::new(40.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 3); // (0,1), (0,2), (1,2)
    }

    #[test]
    fn test_query_pairs_no_duplicates() {
        let mut hash = SpatialHash::new(64.0);

        // Entity spanning multiple cells
        hash.insert(entity(0), Rect::new(0.0, 0.0, 128.0, 32.0)); // Spans 2 cells
        hash.insert(entity(1), Rect::new(20.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 1); // Only one pair despite multiple cells
    }

    #[test]
    fn test_query_point_hit() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_point(Vec2::new(10.0, 10.0));
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    #[test]
    fn test_query_point_miss() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_point(Vec2::new(200.0, 200.0));
        assert_eq!(nearby.len(), 0);
    }

    #[test]
    fn test_query_aabb_overlapping() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(100.0, 100.0, 32.0, 32.0));

        let nearby = hash.query_aabb(Rect::new(-10.0, -10.0, 50.0, 50.0));
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    #[test]
    fn test_query_circle() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_circle(Vec2::new(16.0, 16.0), 20.0);
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    // =========================================================================
    // Statistics Tests
    // =========================================================================

    #[test]
    fn test_stats_empty() {
        let hash = SpatialHash::new(64.0);
        let stats = hash.stats();

        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.cell_count, 0);
        assert_eq!(stats.total_cell_entries, 0);
        assert_eq!(stats.max_entities_per_cell, 0);
        assert_eq!(stats.avg_entities_per_cell, 0.0);
    }

    #[test]
    fn test_stats_after_insert() {
        let mut hash = SpatialHash::new(64.0);

        // Insert 3 entities in same cell
        for i in 0..3 {
            hash.insert(entity(i), Rect::new(i as f32 * 10.0, 0.0, 32.0, 32.0));
        }

        let stats = hash.stats();
        assert_eq!(stats.entity_count, 3);
        assert!(stats.cell_count > 0);
        assert!(stats.total_cell_entries >= 3);
    }

    #[test]
    fn test_stats_display() {
        let hash = SpatialHash::new(64.0);
        let stats = hash.stats();
        let display = format!("{stats}");
        assert!(display.contains("SpatialHash Stats"));
        assert!(display.contains("Entities: 0"));
    }

    // =========================================================================
    // Large Entity Tests
    // =========================================================================

    #[test]
    fn test_large_entity_spans_multiple_cells() {
        let mut hash = SpatialHash::new(64.0);

        // Entity spanning 4 cells (2x2)
        let entity = entity(0);
        hash.insert(entity, Rect::new(0.0, 0.0, 128.0, 128.0));

        assert_eq!(hash.entity_count(), 1);
        assert!(hash.cell_count() >= 4); // Should occupy at least 4 cells
    }

    #[test]
    fn test_tiny_entity_single_cell() {
        let mut hash = SpatialHash::new(64.0);

        // Very small entity
        let entity = entity(0);
        hash.insert(entity, Rect::new(10.0, 10.0, 1.0, 1.0));

        assert_eq!(hash.entity_count(), 1);
        assert_eq!(hash.cell_count(), 1); // Should occupy exactly 1 cell
    }

    // =========================================================================
    // Stress Tests
    // =========================================================================

    #[test]
    fn test_stress_many_entities() {
        let mut hash = SpatialHash::new(64.0);

        // Insert 1000 entities
        for i in 0..1000 {
            let x = (i % 32) as f32 * 50.0;
            let y = (i / 32) as f32 * 50.0;
            hash.insert(entity(i), Rect::new(x, y, 32.0, 32.0));
        }

        assert_eq!(hash.entity_count(), 1000);

        // Query should complete
        let pairs = hash.query_pairs();
        assert!(pairs.len() > 0);
    }

    #[test]
    fn test_stress_update_cycle() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));

        // Update 100 times
        for i in 0..100 {
            let x = (i % 10) as f32 * 20.0;
            let y = (i / 10) as f32 * 20.0;
            hash.update(entity, Rect::new(x, y, 32.0, 32.0));
        }

        assert_eq!(hash.entity_count(), 1);
    }

    // =========================================================================
    // Display Tests
    // =========================================================================

    #[test]
    fn test_display() {
        let hash = SpatialHash::new(64.0);
        let display = format!("{hash}");
        assert!(display.contains("SpatialHash"));
        assert!(display.contains("64.0"));
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let mut hash1 = SpatialHash::new(64.0);
        hash1.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let hash2 = hash1.clone();
        assert_eq!(hash2.entity_count(), hash1.entity_count());
        assert_eq!(hash2.cell_size(), hash1.cell_size());
    }

    #[test]
    fn test_debug() {
        let hash = SpatialHash::new(64.0);
        let debug_str = format!("{hash:?}");
        assert!(debug_str.contains("SpatialHash"));
    }
}
