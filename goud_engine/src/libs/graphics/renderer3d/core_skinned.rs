//! Skinned-mesh management methods for [`Renderer3D`].

use super::core::Renderer3D;
use super::types::PostProcessPipeline;

impl Renderer3D {
    /// Create a skinned mesh and return its ID.
    pub fn create_skinned_mesh(
        &mut self,
        vertices: Vec<f32>,
        skeleton: super::types::Skeleton3D,
    ) -> u32 {
        use super::mesh::upload_buffer;
        let buffer = match upload_buffer(self.backend.as_mut(), &vertices) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to create skinned mesh buffer: {e}");
                return 0;
            }
        };
        // Skinned vertex layout: pos(3) + normal(3) + uv(2) + bone_ids(4) + bone_weights(4) = 16 floats
        let vertex_count = (vertices.len() / 16) as i32;
        let bone_count = skeleton.bone_count();
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let bone_matrices = vec![identity; bone_count];

        let id = self.next_skinned_mesh_id;
        self.next_skinned_mesh_id += 1;
        self.skinned_meshes.insert(
            id,
            super::types::SkinnedMesh3D {
                vertices,
                skeleton,
                bone_matrices,
                buffer,
                vertex_count,
                position: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
                scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
                color: self.config.default_material_color,
            },
        );
        id
    }

    /// Remove a skinned mesh by ID. Returns `true` if it existed.
    pub fn remove_skinned_mesh(&mut self, id: u32) -> bool {
        if let Some(mesh) = self.skinned_meshes.remove(&id) {
            self.backend.destroy_buffer(mesh.buffer);
            true
        } else {
            false
        }
    }

    /// Set the position of a skinned mesh. Returns `true` if found.
    pub fn set_skinned_mesh_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(mesh) = self.skinned_meshes.get_mut(&id) {
            mesh.position = cgmath::Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set the rotation of a skinned mesh. Returns `true` if found.
    pub fn set_skinned_mesh_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(mesh) = self.skinned_meshes.get_mut(&id) {
            mesh.rotation = cgmath::Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set the scale of a skinned mesh. Returns `true` if found.
    pub fn set_skinned_mesh_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(mesh) = self.skinned_meshes.get_mut(&id) {
            mesh.scale = cgmath::Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Update bone matrices for a skinned mesh. Returns `true` if found.
    pub fn set_skinned_mesh_bone_matrices(&mut self, id: u32, matrices: Vec<[f32; 16]>) -> bool {
        if let Some(mesh) = self.skinned_meshes.get_mut(&id) {
            mesh.bone_matrices = matrices;
            true
        } else {
            false
        }
    }

    /// Get a reference to the post-processing pipeline.
    pub fn postprocess_pipeline(&self) -> &PostProcessPipeline {
        &self.postprocess_pipeline
    }

    /// Get a mutable reference to the post-processing pipeline.
    pub fn postprocess_pipeline_mut(&mut self) -> &mut PostProcessPipeline {
        &mut self.postprocess_pipeline
    }
}
