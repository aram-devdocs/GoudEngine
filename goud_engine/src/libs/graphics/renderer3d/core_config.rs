//! Environment configuration and debug draw methods for [`Renderer3D`].

use super::core::Renderer3D;
use super::mesh::create_grid_mesh;
use super::types::{FogConfig, GridConfig, SkyboxConfig};
use crate::core::providers::types::DebugShape3D;
use crate::libs::graphics::backend::{BufferHandle, BufferType, BufferUsage};

use super::debug_draw::build_debug_draw_vertices;

#[allow(missing_docs)]
impl Renderer3D {
    pub fn configure_grid(&mut self, config: GridConfig) {
        if config.size != self.grid_config.size || config.divisions != self.grid_config.divisions {
            self.backend.destroy_buffer(self.grid_buffer);
            if let Ok((buf, count)) =
                create_grid_mesh(self.backend.as_mut(), config.size, config.divisions)
            {
                self.grid_buffer = buf;
                self.grid_vertex_count = count;
            }
        }
        self.grid_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.grid = config;
            }
        }
    }

    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_config.enabled = enabled;
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.grid.enabled = enabled;
            }
        }
    }

    pub fn set_grid_alpha(&mut self, alpha: f32) {
        self.grid_config.alpha = alpha;
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.grid.alpha = alpha;
            }
        }
    }

    pub fn grid_alpha(&self) -> f32 {
        self.grid_config.alpha
    }

    pub fn configure_skybox(&mut self, config: SkyboxConfig) {
        self.skybox_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.skybox = config;
            }
        }
    }

    pub fn configure_fog(&mut self, config: FogConfig) {
        self.fog_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.fog = config;
            }
        }
    }

    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_config.enabled = enabled;
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.fog.enabled = enabled;
            }
        }
    }

    pub fn set_debug_draw_shapes(&mut self, shapes: &[DebugShape3D]) {
        let vertices = build_debug_draw_vertices(shapes);
        if vertices.is_empty() {
            self.debug_draw_vertex_count = 0;
            return;
        }

        let vertex_bytes = bytemuck::cast_slice(vertices.as_slice());
        let required_bytes = vertex_bytes.len();
        let needs_new_buffer = !self.backend.is_buffer_valid(self.debug_draw_buffer)
            || self.debug_draw_buffer_capacity_bytes < required_bytes;

        if needs_new_buffer {
            if self.backend.is_buffer_valid(self.debug_draw_buffer) {
                self.backend.destroy_buffer(self.debug_draw_buffer);
                self.debug_draw_buffer = BufferHandle::INVALID;
                self.debug_draw_buffer_capacity_bytes = 0;
            }

            match self
                .backend
                .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, vertex_bytes)
            {
                Ok(buffer) => {
                    self.debug_draw_buffer = buffer;
                    self.debug_draw_buffer_capacity_bytes = required_bytes;
                }
                Err(e) => {
                    log::error!("Failed to create debug draw buffer: {e}");
                    self.debug_draw_vertex_count = 0;
                    return;
                }
            }
        } else if let Err(e) = self
            .backend
            .update_buffer(self.debug_draw_buffer, 0, vertex_bytes)
        {
            log::error!("Failed to update debug draw buffer: {e}");
            self.debug_draw_vertex_count = 0;
            return;
        }

        self.debug_draw_vertex_count = (vertices.len() / 6) as i32;
    }
}
