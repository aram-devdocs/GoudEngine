//! Grid cell coordinate types for the spatial grid.

use crate::core::math::Vec2;

/// A coordinate in the spatial grid.
///
/// Cells are indexed by integer (x, y) coordinates derived from world
/// positions by floor-dividing by the cell size.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct CellCoord {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl CellCoord {
    /// Creates a new cell coordinate.
    #[inline]
    pub(super) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts a world position to a cell coordinate.
    #[inline]
    pub(super) fn from_world(pos: Vec2, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
        }
    }
}
