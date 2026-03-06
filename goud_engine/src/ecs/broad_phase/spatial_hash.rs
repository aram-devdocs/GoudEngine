//! Spatial hash data structure and mutation operations.

use crate::core::math::Rect;
use crate::ecs::Entity;
use std::collections::{HashMap, HashSet};

use super::grid::CellCoord;
use super::stats::SpatialHashStats;

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
    pub(super) cell_size: f32,

    /// Inverse of cell size for faster division (reserved for future use).
    pub(super) _cell_size_inv: f32,

    /// Grid cells: each cell contains a list of entities.
    /// Key is the cell coordinate, value is the set of entities in that cell.
    pub(super) grid: HashMap<CellCoord, HashSet<Entity>>,

    /// Entity to AABB mapping for updates.
    /// Stores the AABB for each entity to know which cells it occupies.
    pub(super) entity_bounds: HashMap<Entity, Rect>,

    /// Entity to cell list mapping.
    /// Stores which cells each entity occupies for efficient removal.
    pub(super) entity_cells: HashMap<Entity, Vec<CellCoord>>,

    /// Statistics for debugging and profiling.
    pub(super) stats: SpatialHashStats,
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
            _cell_size_inv: 1.0 / cell_size,
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
            _cell_size_inv: 1.0 / cell_size,
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
    // Internal Helpers (shared with queries module)
    // =========================================================================

    /// Gets all cell coordinates that an AABB overlaps.
    pub(super) fn get_cells_for_aabb(&self, aabb: Rect) -> Vec<CellCoord> {
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
    pub(super) fn update_stats(&mut self) {
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
