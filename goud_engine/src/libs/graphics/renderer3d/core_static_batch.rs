//! Static batch building for [`Renderer3D`].
//!
//! Groups static objects into a single VBO to reduce draw calls.

use super::core::Renderer3D;
use crate::libs::graphics::backend::{BufferType, BufferUsage};

/// A contiguous range of vertices in the static batch buffer sharing material and texture.
#[derive(Debug, Clone)]
pub(super) struct StaticBatchGroup {
    /// Texture ID for this group (0 = untextured).
    pub texture_id: u32,
    /// First vertex index in the batch buffer.
    pub start_vertex: u32,
    /// Number of vertices in this group.
    pub vertex_count: u32,
    /// RGBA color resolved from the material.
    pub color: [f32; 4],
}

impl Renderer3D {
    /// Rebuild the static batch VBO from all objects with `is_static == true`.
    ///
    /// Vertices are transformed (baked) by each object's model matrix so the
    /// batch can be drawn with an identity model uniform.  Objects are grouped
    /// by `(material_id, texture_id)` to minimise state changes.
    pub(super) fn rebuild_static_batch(&mut self) {
        // Destroy old buffer.
        if let Some(buf) = self.static_batch_buffer.take() {
            self.backend.destroy_buffer(buf);
        }
        self.static_batch_groups.clear();
        self.static_batch_vertex_count = 0;
        self.static_batched_ids.clear();
        self.static_batch_dirty = false;

        // Collect static object IDs with their sort key.
        let mut entries: Vec<(u32, u32, u32)> = Vec::new(); // (mat, tex, obj_id)
        for (&id, obj) in &self.objects {
            if !obj.is_static {
                continue;
            }
            let mat_id = self.object_materials.get(&id).copied().unwrap_or(0);
            entries.push((mat_id, obj.texture_id, id));
        }

        if entries.is_empty() {
            return;
        }

        entries.sort_by_key(|&(m, t, _)| (m, t));

        // Floats-per-vertex in the object layout: 3 pos + 3 normal + 2 uv = 8.
        const FPV: usize = 8;

        let max_verts = self.config.batching.max_static_batch_vertices;
        let mut all_verts: Vec<f32> = Vec::new();
        let mut current_mat: u32 = entries[0].0;
        let mut current_tex: u32 = entries[0].1;
        let mut group_start: u32 = 0;

        for &(mat_id, tex_id, obj_id) in &entries {
            let obj = match self.objects.get(&obj_id) {
                Some(o) => o,
                None => continue,
            };

            // Start a new group when material or texture changes.
            if mat_id != current_mat || tex_id != current_tex {
                let group_vertex_count = (all_verts.len() / FPV) as u32 - group_start;
                if group_vertex_count > 0 {
                    let color = self.resolve_material_color(current_mat, current_tex);
                    self.static_batch_groups.push(StaticBatchGroup {
                        texture_id: current_tex,
                        start_vertex: group_start,
                        vertex_count: group_vertex_count,
                        color,
                    });
                }
                group_start = (all_verts.len() / FPV) as u32;
                current_mat = mat_id;
                current_tex = tex_id;
            }

            // Enforce vertex budget.
            let obj_vert_count = obj.vertices.len() / FPV;
            if (all_verts.len() / FPV) + obj_vert_count > max_verts {
                log::warn!(
                    "Static batch vertex limit ({max_verts}) reached; \
                     {}/{} static objects batched, remainder uses individual draws",
                    self.static_batched_ids.len(),
                    entries.len(),
                );
                break;
            }

            self.static_batched_ids.insert(obj_id);

            // Build model matrix and bake transform into vertices.
            let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
            let normal_matrix = Self::normal_matrix_from_model(&model);

            for v in 0..obj_vert_count {
                let base = v * FPV;
                // Transform position: model * vec4(pos, 1.0)
                let px = obj.vertices[base];
                let py = obj.vertices[base + 1];
                let pz = obj.vertices[base + 2];
                let cols: &[[f32; 4]; 4] = model.as_ref();
                let tx = cols[0][0] * px + cols[1][0] * py + cols[2][0] * pz + cols[3][0];
                let ty = cols[0][1] * px + cols[1][1] * py + cols[2][1] * pz + cols[3][1];
                let tz = cols[0][2] * px + cols[1][2] * py + cols[2][2] * pz + cols[3][2];
                all_verts.push(tx);
                all_verts.push(ty);
                all_verts.push(tz);

                // Transform normal: normal_matrix * normal (no translation).
                let nx = obj.vertices[base + 3];
                let ny = obj.vertices[base + 4];
                let nz = obj.vertices[base + 5];
                let tnx = normal_matrix[0] * nx + normal_matrix[3] * ny + normal_matrix[6] * nz;
                let tny = normal_matrix[1] * nx + normal_matrix[4] * ny + normal_matrix[7] * nz;
                let tnz = normal_matrix[2] * nx + normal_matrix[5] * ny + normal_matrix[8] * nz;
                // Normalize the transformed normal.
                let len = (tnx * tnx + tny * tny + tnz * tnz).sqrt().max(1e-10);
                all_verts.push(tnx / len);
                all_verts.push(tny / len);
                all_verts.push(tnz / len);

                // UV passthrough.
                all_verts.push(obj.vertices[base + 6]);
                all_verts.push(obj.vertices[base + 7]);
            }
        }

        // Close the final group.
        let group_vertex_count = (all_verts.len() / FPV) as u32 - group_start;
        if group_vertex_count > 0 {
            let color = self.resolve_material_color(current_mat, current_tex);
            self.static_batch_groups.push(StaticBatchGroup {
                texture_id: current_tex,
                start_vertex: group_start,
                vertex_count: group_vertex_count,
                color,
            });
        }

        if all_verts.is_empty() {
            return;
        }

        self.static_batch_vertex_count = (all_verts.len() / FPV) as u32;

        match self.backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&all_verts),
        ) {
            Ok(buf) => {
                self.static_batch_buffer = Some(buf);
            }
            Err(e) => {
                log::error!("Failed to create static batch buffer: {e}");
                self.static_batch_groups.clear();
                self.static_batch_vertex_count = 0;
                self.static_batched_ids.clear();
            }
        }
    }

    /// Resolve RGBA color for a material, following the same logic as the draw loop.
    pub(super) fn resolve_material_color(&self, mat_id: u32, texture_id: u32) -> [f32; 4] {
        if let Some(mat) = self.materials.get(&mat_id) {
            let c = &mat.color;
            [c.x, c.y, c.z, c.w]
        } else if texture_id > 0 {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            self.config.default_material_color
        }
    }

    /// Compute the 3x3 normal matrix (transpose of inverse of upper-left 3x3) as a flat [f32; 9].
    pub(super) fn normal_matrix_from_model(model: &cgmath::Matrix4<f32>) -> [f32; 9] {
        let cols: &[[f32; 4]; 4] = model.as_ref();
        let a = cols[0][0];
        let b = cols[1][0];
        let c = cols[2][0];
        let d = cols[0][1];
        let e = cols[1][1];
        let f = cols[2][1];
        let g = cols[0][2];
        let h = cols[1][2];
        let i = cols[2][2];
        let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        if det.abs() < 1e-10 {
            return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        }
        let inv_det = 1.0 / det;
        [
            (e * i - f * h) * inv_det,
            (f * g - d * i) * inv_det,
            (d * h - e * g) * inv_det,
            (c * h - b * i) * inv_det,
            (a * i - c * g) * inv_det,
            (b * g - a * h) * inv_det,
            (b * f - c * e) * inv_det,
            (c * d - a * f) * inv_det,
            (a * e - b * d) * inv_det,
        ]
    }
}
