//! Query operations for the spatial grid.

use crate::core::math::Vec2;
use crate::ecs::Entity;

use super::core::SpatialGrid;
use super::grid::CellCoord;

impl SpatialGrid {
    /// Queries for entities within a radius of a center point.
    ///
    /// First identifies candidate cells that overlap the query circle's
    /// bounding square, then filters by exact Euclidean distance.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the query circle
    /// * `radius` - The query radius
    ///
    /// # Returns
    ///
    /// A vector of entities within the specified radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::spatial_grid::SpatialGrid;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut grid = SpatialGrid::new(32.0);
    /// let e1 = Entity::new(0, 0);
    /// let e2 = Entity::new(1, 0);
    ///
    /// grid.insert(e1, Vec2::new(10.0, 10.0));
    /// grid.insert(e2, Vec2::new(100.0, 100.0));
    ///
    /// let nearby = grid.query_radius(Vec2::new(10.0, 10.0), 20.0);
    /// assert_eq!(nearby.len(), 1);
    /// assert_eq!(nearby[0], e1);
    /// ```
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        debug_assert!(radius >= 0.0, "query_radius called with negative radius");
        let radius_sq = radius * radius;
        let cell_size = self.cell_size();

        // Compute bounding cell range
        let min = Vec2::new(center.x - radius, center.y - radius);
        let max = Vec2::new(center.x + radius, center.y + radius);
        let min_cell = CellCoord::from_world(min, cell_size);
        let max_cell = CellCoord::from_world(max, cell_size);

        let mut result = Vec::new();
        let grid = self.grid();
        let positions = self.entity_positions();

        // Scan candidate cells
        for cy in min_cell.y..=max_cell.y {
            for cx in min_cell.x..=max_cell.x {
                let cell = CellCoord::new(cx, cy);
                if let Some(cell_entities) = grid.get(&cell) {
                    for &entity in cell_entities {
                        // Exact distance filter
                        if let Some(&pos) = positions.get(&entity) {
                            let dx = pos.x - center.x;
                            let dy = pos.y - center.y;
                            if dx * dx + dy * dy <= radius_sq {
                                result.push(entity);
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Queries for entities in the same cell as a point.
    ///
    /// Returns all entities whose position maps to the same grid cell
    /// as the query point. This is an O(1) cell lookup.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::spatial_grid::SpatialGrid;
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut grid = SpatialGrid::new(64.0);
    /// let entity = Entity::new(0, 0);
    ///
    /// grid.insert(entity, Vec2::new(10.0, 10.0));
    ///
    /// let nearby = grid.query_point(Vec2::new(20.0, 20.0));
    /// assert_eq!(nearby.len(), 1); // Same cell
    /// ```
    pub fn query_point(&self, point: Vec2) -> Vec<Entity> {
        let cell = CellCoord::from_world(point, self.cell_size());

        self.grid()
            .get(&cell)
            .map(|entities| entities.iter().copied().collect())
            .unwrap_or_default()
    }
}
