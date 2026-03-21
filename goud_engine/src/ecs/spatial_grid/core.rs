//! Spatial grid data structure and mutation operations.

use crate::core::math::Vec2;
use crate::ecs::Entity;
use std::collections::{HashMap, HashSet};

use super::grid::CellCoord;

/// A point-based spatial grid for efficient proximity queries.
///
/// Unlike [`SpatialHash`](crate::ecs::broad_phase::SpatialHash) which uses
/// AABBs and is designed for collision detection, `SpatialGrid` stores entities
/// as points. Each entity occupies exactly one cell, making insert, remove,
/// and update operations O(1).
///
/// Designed for game-level spatial queries: AI neighbor lookups, combat range
/// checks, resource gathering, etc. Independent of the physics system.
///
/// # Performance
///
/// - Insert: O(1)
/// - Remove: O(1)
/// - Update: O(1) (same cell) or O(1) amortized (cell change)
/// - Query radius: O(k) where k = entities in covered cells
///
/// # Cell Size Selection
///
/// Choose a cell size close to the typical query radius for best performance.
/// - Too small: radius queries scan many cells
/// - Too large: too many entities per cell, wasting filter time
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::spatial_grid::SpatialGrid;
/// use goud_engine::ecs::Entity;
/// use goud_engine::core::math::Vec2;
///
/// let mut grid = SpatialGrid::new(32.0);
///
/// let player = Entity::new(0, 0);
/// let enemy = Entity::new(1, 0);
///
/// grid.insert(player, Vec2::new(100.0, 100.0));
/// grid.insert(enemy, Vec2::new(120.0, 110.0));
///
/// // Find entities within 50 units of the player
/// let nearby = grid.query_radius(Vec2::new(100.0, 100.0), 50.0);
/// assert_eq!(nearby.len(), 2); // Both entities are within range
/// ```
#[derive(Clone, Debug)]
pub struct SpatialGrid {
    /// Size of each grid cell (world units).
    cell_size: f32,

    /// Entity to cell mapping. Each entity is in exactly one cell.
    entity_cells: HashMap<Entity, CellCoord>,

    /// Entity to position mapping for exact distance filtering.
    entity_positions: HashMap<Entity, Vec2>,

    /// Grid cells: each cell contains a set of entities.
    grid: HashMap<CellCoord, HashSet<Entity>>,
}

impl SpatialGrid {
    /// Creates a new spatial grid with the specified cell size.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - Size of each grid cell in world units
    ///
    /// # Panics
    ///
    /// Panics if `cell_size` is not positive and finite.
    pub fn new(cell_size: f32) -> Self {
        assert!(
            cell_size > 0.0 && cell_size.is_finite(),
            "Cell size must be positive and finite"
        );

        Self {
            cell_size,
            entity_cells: HashMap::new(),
            entity_positions: HashMap::new(),
            grid: HashMap::new(),
        }
    }

    /// Creates a spatial grid with pre-allocated capacity.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - Size of each grid cell in world units
    /// * `capacity` - Expected number of entities
    ///
    /// # Panics
    ///
    /// Panics if `cell_size` is not positive and finite.
    pub fn with_capacity(cell_size: f32, capacity: usize) -> Self {
        assert!(
            cell_size > 0.0 && cell_size.is_finite(),
            "Cell size must be positive and finite"
        );

        Self {
            cell_size,
            entity_cells: HashMap::with_capacity(capacity),
            entity_positions: HashMap::with_capacity(capacity),
            grid: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts an entity at a position into the spatial grid.
    ///
    /// If the entity already exists, it is removed first and re-inserted
    /// at the new position.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to insert
    /// * `position` - The entity's world position
    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        // Remove if already present
        if self.entity_cells.contains_key(&entity) {
            self.remove(entity);
        }

        let cell = CellCoord::from_world(position, self.cell_size);
        self.grid.entry(cell).or_default().insert(entity);
        self.entity_cells.insert(entity, cell);
        self.entity_positions.insert(entity, position);
    }

    /// Removes an entity from the spatial grid.
    ///
    /// # Returns
    ///
    /// `true` if the entity was present and removed, `false` otherwise.
    pub fn remove(&mut self, entity: Entity) -> bool {
        let cell = match self.entity_cells.remove(&entity) {
            Some(c) => c,
            None => return false,
        };

        if let Some(cell_entities) = self.grid.get_mut(&cell) {
            cell_entities.remove(&entity);
            if cell_entities.is_empty() {
                self.grid.remove(&cell);
            }
        }

        self.entity_positions.remove(&entity);
        true
    }

    /// Updates an entity's position in the spatial grid.
    ///
    /// Optimized for the common case where the entity stays in the same cell.
    ///
    /// # Returns
    ///
    /// `true` if the entity was present and updated, `false` otherwise.
    pub fn update(&mut self, entity: Entity, new_position: Vec2) -> bool {
        let old_cell = match self.entity_cells.get(&entity) {
            Some(c) => *c,
            None => return false,
        };

        let new_cell = CellCoord::from_world(new_position, self.cell_size);

        // Always update the stored position
        self.entity_positions.insert(entity, new_position);

        // If cell hasn't changed, we're done
        if old_cell == new_cell {
            return true;
        }

        // Remove from old cell
        if let Some(cell_entities) = self.grid.get_mut(&old_cell) {
            cell_entities.remove(&entity);
            if cell_entities.is_empty() {
                self.grid.remove(&old_cell);
            }
        }

        // Insert into new cell
        self.grid.entry(new_cell).or_default().insert(entity);
        self.entity_cells.insert(entity, new_cell);

        true
    }

    /// Clears all entities from the spatial grid.
    pub fn clear(&mut self) {
        self.grid.clear();
        self.entity_cells.clear();
        self.entity_positions.clear();
    }

    /// Returns the cell size.
    #[inline]
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Returns the number of entities in the grid.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entity_cells.len()
    }

    /// Returns whether the grid is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_cells.is_empty()
    }

    /// Returns whether an entity is in the grid.
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_cells.contains_key(&entity)
    }

    /// Returns the position of an entity, if present.
    #[inline]
    pub fn get_position(&self, entity: Entity) -> Option<Vec2> {
        self.entity_positions.get(&entity).copied()
    }

    /// Returns a reference to the grid's cell map (for query operations).
    #[inline]
    pub(super) fn grid(&self) -> &HashMap<CellCoord, HashSet<Entity>> {
        &self.grid
    }

    /// Returns a reference to the entity positions map.
    #[inline]
    pub(super) fn entity_positions(&self) -> &HashMap<Entity, Vec2> {
        &self.entity_positions
    }
}

impl std::fmt::Display for SpatialGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpatialGrid(cell_size: {:.1}, entities: {}, cells: {})",
            self.cell_size,
            self.entity_count(),
            self.grid.len()
        )
    }
}
