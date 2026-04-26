//! Sparse uniform-grid spatial index for fast frustum culling of 3D scene objects.
//!
//! Each object is keyed by its world-space AABB. The grid is a sparse
//! `FxHashMap<(i32,i32,i32), Vec<u32>>` so empty space costs nothing. Inserts,
//! removes, and updates are O(cells touched) per object — for typical scene
//! objects whose bounding sphere fits inside one cell, that is O(1) work.
//!
//! `query_aabb` enumerates the cells overlapping the queried AABB and yields
//! every object ID found at least once, deduplicated via a per-query stamp so
//! callers do not have to maintain their own seen-set.
//!
//! The index does not perform the per-object frustum-sphere test itself — that
//! still lives in [`super::frustum::Frustum`]. Its job is to shrink the
//! candidate set from the full scene-object registry (which can grow to ~55k
//! sub-meshes on large maps) to only the objects whose grid cell touches the
//! frustum AABB.
//!
//! See `goud_engine/tests/integration/scene_culling.rs` for the
//! parity check against the linear-scan baseline and `benches/scene_culling.rs`
//! for the 1k/10k/50k scaling benchmark.
use cgmath::Vector3;
use rustc_hash::FxHashMap;

/// Default cell size in world units. Sized to cover typical small props
/// (1–8 units) without forcing one object to span more than a 2×2×2 footprint
/// while keeping cell count manageable for 200×200 maps.
pub(in crate::libs::graphics::renderer3d) const DEFAULT_CELL_SIZE: f32 = 32.0;

/// Cached cell-coord range for a single object so removals and updates do not
/// have to re-derive the cell footprint from a stale AABB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellRange {
    min: (i32, i32, i32),
    max: (i32, i32, i32),
}

/// Sparse uniform-grid spatial index over `u32` object IDs.
#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct SpatialIndex {
    cell_size: f32,
    inv_cell_size: f32,
    cells: FxHashMap<(i32, i32, i32), Vec<u32>>,
    object_ranges: FxHashMap<u32, CellRange>,
    /// Per-object visit stamp for query dedup. Indexed by object ID; grown
    /// lazily on first sight of an ID.
    query_stamp: Vec<u64>,
    next_stamp: u64,
    last_query_visited_cells: u32,
    last_query_visited_candidates: u32,
}

// Diagnostic accessors and `clear`/`update` are exposed for tests and
// integration callers. `update` is a thin alias around `insert` because
// `insert` already overwrites stale cell entries when an ID is re-inserted.
#[allow(dead_code)]
impl SpatialIndex {
    /// Construct a new empty index with the given cell size in world units.
    /// `cell_size` is clamped to at least `0.5` to keep the grid coordinates
    /// finite when callers accidentally pass `0.0`.
    pub(in crate::libs::graphics::renderer3d) fn new(cell_size: f32) -> Self {
        let cs = cell_size.max(0.5);
        Self {
            cell_size: cs,
            inv_cell_size: 1.0 / cs,
            cells: FxHashMap::default(),
            object_ranges: FxHashMap::default(),
            query_stamp: Vec::new(),
            next_stamp: 0,
            last_query_visited_cells: 0,
            last_query_visited_candidates: 0,
        }
    }

    pub(in crate::libs::graphics::renderer3d) fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub(in crate::libs::graphics::renderer3d) fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub(in crate::libs::graphics::renderer3d) fn object_count(&self) -> usize {
        self.object_ranges.len()
    }

    /// Cell count visited by the most recent `query_aabb` call. Useful for
    /// stats / instrumentation; resets at the start of every query.
    pub(in crate::libs::graphics::renderer3d) fn last_query_visited_cells(&self) -> u32 {
        self.last_query_visited_cells
    }

    /// Candidate object count visited by the most recent `query_aabb` call,
    /// **after** dedup. This is the size of the set the caller iterates over.
    pub(in crate::libs::graphics::renderer3d) fn last_query_visited_candidates(&self) -> u32 {
        self.last_query_visited_candidates
    }

    /// Drop every entry. Equivalent to recreating the index with the same
    /// cell size.
    pub(in crate::libs::graphics::renderer3d) fn clear(&mut self) {
        self.cells.clear();
        self.object_ranges.clear();
        // query_stamp can stay — it is keyed by ID and will be overwritten
        // on the next stamped query. Resizing here would just churn allocs.
        self.last_query_visited_cells = 0;
        self.last_query_visited_candidates = 0;
    }

    fn aabb_to_cell_range(&self, world_min: Vector3<f32>, world_max: Vector3<f32>) -> CellRange {
        let min = (
            (world_min.x * self.inv_cell_size).floor() as i32,
            (world_min.y * self.inv_cell_size).floor() as i32,
            (world_min.z * self.inv_cell_size).floor() as i32,
        );
        let max = (
            (world_max.x * self.inv_cell_size).floor() as i32,
            (world_max.y * self.inv_cell_size).floor() as i32,
            (world_max.z * self.inv_cell_size).floor() as i32,
        );
        CellRange { min, max }
    }

    fn add_to_cells(&mut self, id: u32, range: CellRange) {
        for x in range.min.0..=range.max.0 {
            for y in range.min.1..=range.max.1 {
                for z in range.min.2..=range.max.2 {
                    self.cells.entry((x, y, z)).or_default().push(id);
                }
            }
        }
    }

    fn remove_from_cells(&mut self, id: u32, range: CellRange) {
        for x in range.min.0..=range.max.0 {
            for y in range.min.1..=range.max.1 {
                for z in range.min.2..=range.max.2 {
                    let key = (x, y, z);
                    if let Some(bucket) = self.cells.get_mut(&key) {
                        if let Some(pos) = bucket.iter().position(|&i| i == id) {
                            bucket.swap_remove(pos);
                        }
                        if bucket.is_empty() {
                            self.cells.remove(&key);
                        }
                    }
                }
            }
        }
    }

    /// Insert (or re-insert) `id` with the given world-space AABB. If the
    /// object was already present, its old entries are removed first.
    pub(in crate::libs::graphics::renderer3d) fn insert(
        &mut self,
        id: u32,
        world_min: Vector3<f32>,
        world_max: Vector3<f32>,
    ) {
        let new_range = self.aabb_to_cell_range(world_min, world_max);
        if let Some(old_range) = self.object_ranges.insert(id, new_range) {
            if old_range == new_range {
                // Already in the right cells — bail out before we double-add.
                return;
            }
            self.remove_from_cells(id, old_range);
        }
        self.add_to_cells(id, new_range);
    }

    /// Update `id` to a new world-space AABB. Falls back to `insert` when the
    /// object is not yet tracked.
    pub(in crate::libs::graphics::renderer3d) fn update(
        &mut self,
        id: u32,
        world_min: Vector3<f32>,
        world_max: Vector3<f32>,
    ) {
        self.insert(id, world_min, world_max);
    }

    /// Remove `id` and all of its cell entries. Returns `true` when the entry
    /// existed.
    pub(in crate::libs::graphics::renderer3d) fn remove(&mut self, id: u32) -> bool {
        if let Some(range) = self.object_ranges.remove(&id) {
            self.remove_from_cells(id, range);
            true
        } else {
            false
        }
    }

    /// Visit every object whose stored AABB cell-range overlaps the queried
    /// AABB. Each object is visited at most once per call.
    ///
    /// Two iteration strategies, picked per-call:
    /// * **AABB cell sweep** when the queried cell range is small relative to
    ///   the index's occupied cell count — typical for tight queries.
    /// * **Occupied-cell scan** when the queried cell range is much larger
    ///   than the populated grid — typical when the far plane stretches well
    ///   past the actual scene extent. Walking the populated cells avoids
    ///   spending time on empty space the camera could in principle see but
    ///   that has no objects in it.
    pub(in crate::libs::graphics::renderer3d) fn query_aabb<F>(
        &mut self,
        world_min: Vector3<f32>,
        world_max: Vector3<f32>,
        mut visit: F,
    ) where
        F: FnMut(u32),
    {
        // Use a fresh stamp every query; on wrap-around just clear the stamp
        // table so old marks cannot collide with the new one.
        self.next_stamp = self.next_stamp.wrapping_add(1);
        if self.next_stamp == 0 {
            for slot in self.query_stamp.iter_mut() {
                *slot = 0;
            }
            self.next_stamp = 1;
        }
        let stamp = self.next_stamp;
        let range = self.aabb_to_cell_range(world_min, world_max);
        let mut visited_cells: u32 = 0;
        let mut visited_candidates: u32 = 0;

        // Estimate cell-sweep cost as the AABB cell-range product. Saturating
        // arithmetic keeps a giant query (e.g. far plane >> scene extent) from
        // overflowing into a misleadingly small `usize`.
        let dx = (range.max.0 as i64 - range.min.0 as i64 + 1).max(0) as u64;
        let dy = (range.max.1 as i64 - range.min.1 as i64 + 1).max(0) as u64;
        let dz = (range.max.2 as i64 - range.min.2 as i64 + 1).max(0) as u64;
        let sweep_cost = dx.saturating_mul(dy).saturating_mul(dz);
        let occupied = self.cells.len() as u64;

        if sweep_cost <= occupied.saturating_mul(2) {
            // AABB sweep: cheap when the AABB only touches a handful of cells.
            for x in range.min.0..=range.max.0 {
                for y in range.min.1..=range.max.1 {
                    for z in range.min.2..=range.max.2 {
                        let key = (x, y, z);
                        let Some(bucket) = self.cells.get(&key) else {
                            continue;
                        };
                        visited_cells = visited_cells.saturating_add(1);
                        Self::visit_bucket(
                            bucket,
                            stamp,
                            &mut self.query_stamp,
                            &mut visited_candidates,
                            &mut visit,
                        );
                    }
                }
            }
        } else {
            // Occupied-cell scan: cheaper when the AABB straddles many empty
            // cells (far plane >> scene extent).
            for (cell_key, bucket) in &self.cells {
                if cell_key.0 < range.min.0
                    || cell_key.0 > range.max.0
                    || cell_key.1 < range.min.1
                    || cell_key.1 > range.max.1
                    || cell_key.2 < range.min.2
                    || cell_key.2 > range.max.2
                {
                    continue;
                }
                visited_cells = visited_cells.saturating_add(1);
                Self::visit_bucket(
                    bucket,
                    stamp,
                    &mut self.query_stamp,
                    &mut visited_candidates,
                    &mut visit,
                );
            }
        }
        self.last_query_visited_cells = visited_cells;
        self.last_query_visited_candidates = visited_candidates;
    }

    fn visit_bucket<F>(
        bucket: &[u32],
        stamp: u64,
        query_stamp: &mut Vec<u64>,
        visited_candidates: &mut u32,
        visit: &mut F,
    ) where
        F: FnMut(u32),
    {
        for &id in bucket {
            let idx = id as usize;
            if idx >= query_stamp.len() {
                query_stamp.resize(idx + 1, 0);
            }
            if query_stamp[idx] != stamp {
                query_stamp[idx] = stamp;
                *visited_candidates = visited_candidates.saturating_add(1);
                visit(id);
            }
        }
    }
}

/// Convenience: derive a tight world-space AABB from a local-space bounding
/// sphere `(center, radius)` plus a world transform `(position, scale)`.
/// Conservative: ignores rotation by extending by the scaled radius on every
/// axis, which is correct for sphere-style bounds but slightly loose for
/// non-uniform scale.
pub(in crate::libs::graphics::renderer3d) fn world_aabb_from_sphere(
    position: Vector3<f32>,
    bounds_center: Vector3<f32>,
    bounds_radius: f32,
    scale: Vector3<f32>,
) -> (Vector3<f32>, Vector3<f32>) {
    let max_scale = scale.x.abs().max(scale.y.abs()).max(scale.z.abs());
    let world_center = position + bounds_center;
    let r = bounds_radius * max_scale;
    (
        Vector3::new(world_center.x - r, world_center.y - r, world_center.z - r),
        Vector3::new(world_center.x + r, world_center.y + r, world_center.z + r),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector3;

    fn collect(idx: &mut SpatialIndex, min: Vector3<f32>, max: Vector3<f32>) -> Vec<u32> {
        let mut out = Vec::new();
        idx.query_aabb(min, max, |id| out.push(id));
        out.sort_unstable();
        out
    }

    #[test]
    fn empty_index_yields_no_candidates() {
        let mut idx = SpatialIndex::new(16.0);
        let got = collect(
            &mut idx,
            Vector3::new(-100.0, -100.0, -100.0),
            Vector3::new(100.0, 100.0, 100.0),
        );
        assert!(got.is_empty());
        assert_eq!(idx.last_query_visited_cells(), 0);
        assert_eq!(idx.last_query_visited_candidates(), 0);
    }

    #[test]
    fn insert_then_query_returns_object() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(8.0, 8.0, 8.0),
        );
        assert_eq!(got, vec![7]);
        assert_eq!(idx.last_query_visited_candidates(), 1);
    }

    #[test]
    fn query_outside_skips_object() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        let got = collect(
            &mut idx,
            Vector3::new(100.0, 100.0, 100.0),
            Vector3::new(120.0, 120.0, 120.0),
        );
        assert!(got.is_empty());
        assert_eq!(idx.last_query_visited_cells(), 0);
    }

    #[test]
    fn object_spanning_cells_dedupes_to_one_visit() {
        let mut idx = SpatialIndex::new(8.0);
        // AABB straddles four cells in the XY plane: (0,0,0), (1,0,0),
        // (0,1,0), (1,1,0).
        idx.insert(42, Vector3::new(7.0, 7.0, 0.5), Vector3::new(9.0, 9.0, 0.5));
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(20.0, 20.0, 1.0),
        );
        assert_eq!(got, vec![42]);
        assert_eq!(idx.last_query_visited_candidates(), 1);
        assert!(
            idx.last_query_visited_cells() >= 4,
            "expected at least 4 cells visited, got {}",
            idx.last_query_visited_cells()
        );
    }

    #[test]
    fn update_moves_object_between_cells() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(7, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        idx.update(
            7,
            Vector3::new(100.0, 100.0, 100.0),
            Vector3::new(101.0, 101.0, 101.0),
        );
        // Old location should yield nothing.
        assert!(collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(8.0, 8.0, 8.0)
        )
        .is_empty());
        // New location should yield the object.
        assert_eq!(
            collect(
                &mut idx,
                Vector3::new(99.0, 99.0, 99.0),
                Vector3::new(110.0, 110.0, 110.0)
            ),
            vec![7]
        );
    }

    #[test]
    fn remove_drops_object_from_all_cells() {
        let mut idx = SpatialIndex::new(8.0);
        idx.insert(42, Vector3::new(7.0, 7.0, 0.5), Vector3::new(9.0, 9.0, 0.5));
        assert!(idx.remove(42));
        assert_eq!(idx.object_count(), 0);
        assert_eq!(idx.cell_count(), 0);
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(20.0, 20.0, 1.0),
        );
        assert!(got.is_empty());
    }

    #[test]
    fn remove_missing_id_returns_false() {
        let mut idx = SpatialIndex::new(16.0);
        assert!(!idx.remove(99));
    }

    #[test]
    fn many_objects_in_one_cell_survive_partial_removal() {
        let mut idx = SpatialIndex::new(16.0);
        for id in 0..32 {
            idx.insert(id, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        }
        // Remove evens.
        for id in (0..32).step_by(2) {
            assert!(idx.remove(id));
        }
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(8.0, 8.0, 8.0),
        );
        let expected: Vec<u32> = (1..32).step_by(2).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn reinsert_with_same_aabb_is_idempotent() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(5, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        idx.insert(5, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        let got = collect(
            &mut idx,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(8.0, 8.0, 8.0),
        );
        assert_eq!(got, vec![5]);
        assert_eq!(idx.object_count(), 1);
    }

    #[test]
    fn world_aabb_from_sphere_handles_scale() {
        let (min, max) = world_aabb_from_sphere(
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            2.0,
            Vector3::new(3.0, 1.0, 1.0),
        );
        // World center is (10, 1, 0); scaled radius is 2 * max(3,1,1) = 6.
        assert!((min.x - 4.0).abs() < f32::EPSILON);
        assert!((max.x - 16.0).abs() < f32::EPSILON);
        assert!((min.y - (-5.0)).abs() < f32::EPSILON);
        assert!((max.y - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clear_drops_everything_but_keeps_cell_size() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(1, Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        idx.clear();
        assert_eq!(idx.object_count(), 0);
        assert_eq!(idx.cell_count(), 0);
        assert!((idx.cell_size() - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn negative_world_coords_route_to_negative_cells() {
        let mut idx = SpatialIndex::new(8.0);
        idx.insert(
            1,
            Vector3::new(-9.0, -9.0, -9.0),
            Vector3::new(-1.0, -1.0, -1.0),
        );
        let got = collect(
            &mut idx,
            Vector3::new(-16.0, -16.0, -16.0),
            Vector3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(got, vec![1]);
    }

    #[test]
    fn stamp_wraparound_preserves_correctness() {
        let mut idx = SpatialIndex::new(16.0);
        idx.insert(3, Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 2.0, 2.0));
        // Force the stamp counter to wrap.
        idx.next_stamp = u64::MAX - 1;
        for _ in 0..4 {
            let got = collect(
                &mut idx,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(8.0, 8.0, 8.0),
            );
            assert_eq!(got, vec![3]);
        }
    }

    #[test]
    fn cell_size_floor_clamp_protects_against_zero() {
        let idx = SpatialIndex::new(0.0);
        assert!(idx.cell_size() >= 0.5);
    }
}
