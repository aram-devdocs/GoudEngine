//! Query operations for the spatial hash.

use crate::core::math::{Rect, Vec2};
use crate::ecs::Entity;
use std::collections::HashSet;

use super::grid::CellCoord;
use super::spatial_hash::SpatialHash;
use super::stats::SpatialHashStats;

impl SpatialHash {
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
}
