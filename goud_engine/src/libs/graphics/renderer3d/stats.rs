//! Per-frame renderer statistics.
//!
//! Split out of [`super::types`] so the file-size budget for `types.rs`
//! stays under the repo's per-file line limit. Re-exported from `types.rs`
//! for backward compatibility with the existing public API path.

/// Last-frame renderer statistics exposed for tests and debugging.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Renderer3DStats {
    /// Total draw calls recorded by the renderer this frame.
    pub draw_calls: u32,
    /// Instanced draw calls recorded this frame.
    pub instanced_draw_calls: u32,
    /// Particle instanced draw calls recorded this frame.
    pub particle_draw_calls: u32,
    /// Number of instance records submitted this frame.
    pub active_instances: u32,
    /// Number of live particles submitted this frame.
    pub active_particles: u32,
    /// Total objects in the scene (before culling).
    pub total_objects: u32,
    /// Objects that passed frustum culling and were drawn.
    pub visible_objects: u32,
    /// Objects culled by frustum test.
    pub culled_objects: u32,
    /// Number of material/shader state switches this frame.
    pub material_switches: u32,
    /// Number of texture bind operations this frame.
    pub texture_binds: u32,
    /// Number of skinned mesh instances rendered this frame.
    pub skinned_instances: u32,
    /// Number of bone matrix uploads this frame.
    pub bone_matrix_uploads: u32,
    /// Number of animation evaluations this frame.
    pub animation_evaluations: u32,
    /// Number of animation evaluations saved (cache hits / LOD skips) this frame.
    pub animation_evaluations_saved: u32,
    /// Number of objects that survived the spatial-index pre-filter and were
    /// fed into the frustum sphere test this frame. Equals `total_objects`
    /// when the spatial index is disabled.
    pub spatial_index_candidates: u32,
    /// Number of grid cells visited by the spatial index this frame.
    /// Useful for tuning [`crate::libs::graphics::renderer3d::Render3DConfig`]
    /// `spatial_index.cell_size` against the active scene.
    pub spatial_index_cells_visited: u32,
}
