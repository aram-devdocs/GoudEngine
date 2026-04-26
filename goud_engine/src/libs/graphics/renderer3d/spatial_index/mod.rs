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
    ///
    /// Memory note: this `Vec` is keyed by raw `u32` object ID, so its size
    /// tracks the renderer's monotonically increasing `next_object_id`. For
    /// long-running sessions that churn through many millions of object
    /// allocations the vec grows unboundedly. In practice the renderer skips
    /// `0` and wraps at `u32::MAX`, so a worst-case session would need to
    /// burn through ~2^32 object IDs before this becomes a real RAM hog.
    /// Trimming on object removal is intentionally not done — clearing a
    /// slot would require either re-stamping every live entry or tracking
    /// an explicit free list, neither of which is worth the complexity.
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
mod tests;
