//! GPU resource management for the sprite batch renderer.

use super::batch::SpriteBatch;
use super::types::SpriteVertex;
use crate::assets::{loaders::TextureAsset, AssetHandle, AssetServer};
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Vec2;
use crate::libs::graphics::backend::types::{BufferType, BufferUsage, TextureHandle};
use crate::libs::graphics::backend::RenderBackend;

impl<B: RenderBackend> SpriteBatch<B> {
    /// Ensures GPU resources (buffers, shader) are created.
    pub(super) fn ensure_resources(&mut self) -> GoudResult<()> {
        // Create vertex buffer if needed
        if self.vertex_buffer.is_none() || self.vertices.len() > self.vertex_capacity * 4 {
            self.create_vertex_buffer()?;
        }

        // Create index buffer if needed
        if self.index_buffer.is_none() {
            self.create_index_buffer()?;
        }

        // Create shader if needed
        if self.shader.is_none() {
            self.create_shader()?;
        }

        Ok(())
    }

    /// Creates or resizes the vertex buffer.
    pub(super) fn create_vertex_buffer(&mut self) -> GoudResult<()> {
        // Calculate new capacity (double if needed)
        let required_sprites = self.vertices.len().div_ceil(4);
        let new_capacity = if required_sprites > self.vertex_capacity {
            (required_sprites * 2).max(self.config.initial_capacity)
        } else {
            self.config.initial_capacity
        };

        let buffer_size = new_capacity * 4 * std::mem::size_of::<SpriteVertex>();

        // Destroy old buffer if exists
        if let Some(old_buffer) = self.vertex_buffer {
            let _ = self.backend.destroy_buffer(old_buffer);
        }

        // Create new buffer with empty data (will be updated later)
        let empty_data = vec![0u8; buffer_size];
        let buffer =
            self.backend
                .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &empty_data)?;

        self.vertex_buffer = Some(buffer);
        self.vertex_capacity = new_capacity;

        Ok(())
    }

    /// Creates the shared index buffer for quad rendering.
    pub(super) fn create_index_buffer(&mut self) -> GoudResult<()> {
        // Generate indices for max_batch_size quads
        let quad_count = self.config.max_batch_size;
        let mut indices = Vec::with_capacity(quad_count * 6);

        for i in 0..quad_count {
            let base = (i * 4) as u32;
            // Two triangles per quad (CCW winding)
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }

        let buffer_size = indices.len() * std::mem::size_of::<u32>();
        // SAFETY: `indices` is a Vec<u32> with a known length; we reinterpret the
        // memory as bytes only to pass raw data to the GPU buffer API. The slice
        // lifetime is bounded to this function, so no aliasing occurs.
        let buffer_data =
            unsafe { std::slice::from_raw_parts(indices.as_ptr() as *const u8, buffer_size) };

        let buffer =
            self.backend
                .create_buffer(BufferType::Index, BufferUsage::Static, buffer_data)?;

        self.index_buffer = Some(buffer);

        Ok(())
    }

    /// Creates the sprite shader program.
    pub(super) fn create_shader(&mut self) -> GoudResult<()> {
        // TODO: Load shader from assets or use built-in shader
        // For now, return error as shader loading isn't implemented yet
        Err(GoudError::NotImplemented(
            "Sprite shader creation".to_string(),
        ))
    }

    /// Uploads vertex data to the GPU.
    pub(super) fn upload_vertices(&mut self) -> GoudResult<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        let buffer = self
            .vertex_buffer
            .ok_or_else(|| GoudError::InvalidState("Vertex buffer not created".to_string()))?;

        let data_size = self.vertices.len() * std::mem::size_of::<SpriteVertex>();
        let data_ptr = self.vertices.as_ptr() as *const u8;
        // SAFETY: `self.vertices` is a contiguous Vec<SpriteVertex>; we reinterpret
        // the memory as bytes to submit raw data to the GPU. The slice lifetime is
        // bounded to this function call, so no aliasing or dangling references occur.
        let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };

        self.backend.update_buffer(buffer, 0, data_slice)?;

        Ok(())
    }

    /// Resolves an asset handle to a GPU texture handle.
    pub(super) fn resolve_texture(
        &mut self,
        asset_handle: AssetHandle<TextureAsset>,
        asset_server: &AssetServer,
    ) -> GoudResult<TextureHandle> {
        // Check cache first
        if let Some(&gpu_handle) = self.texture_cache.get(&asset_handle) {
            return Ok(gpu_handle);
        }

        // Load texture from asset server
        let _texture_asset = asset_server.get(&asset_handle).ok_or_else(|| {
            GoudError::ResourceNotFound(format!("Texture asset {:?}", asset_handle))
        })?;

        // TODO: Upload texture to GPU and cache handle
        // For now, return error as texture upload isn't implemented yet
        Err(GoudError::NotImplemented("Texture upload".to_string()))
    }

    /// Gets the size of a texture from the asset server.
    pub(super) fn get_texture_size(
        &self,
        asset_handle: AssetHandle<TextureAsset>,
        asset_server: &AssetServer,
    ) -> Vec2 {
        if let Some(texture) = asset_server.get(&asset_handle) {
            Vec2::new(texture.width as f32, texture.height as f32)
        } else {
            Vec2::one() // Fallback to 1x1
        }
    }

    /// Renders all batches to the GPU.
    ///
    /// Called at the end of `draw_sprites` after batches have been assembled.
    pub(super) fn render_batches(&mut self) -> GoudResult<()> {
        if self.batches.is_empty() {
            return Ok(());
        }

        // Ensure GPU resources are created
        self.ensure_resources()?;

        // Upload vertex data
        self.upload_vertices()?;

        // Bind shader and set uniforms
        if let Some(shader) = self.shader {
            self.backend.bind_shader(shader)?;
            // TODO: Set projection matrix uniform
        }

        // Bind vertex buffer and set attributes
        if let Some(vbo) = self.vertex_buffer {
            self.backend.bind_buffer(vbo)?;
            self.backend.set_vertex_attributes(&SpriteVertex::layout());
        }

        // Bind index buffer
        if let Some(ibo) = self.index_buffer {
            self.backend.bind_buffer(ibo)?;
        }

        // Draw each batch — collect data first to satisfy borrow checker,
        // since drawing requires &mut self.backend while iterating &self.batches.
        let draw_calls: Vec<(Option<TextureHandle>, usize, usize, usize)> = self
            .batches
            .iter()
            .map(|b| {
                (
                    b.gpu_texture,
                    b.vertex_count,
                    b.vertex_start,
                    b.vertex_count,
                )
            })
            .collect();

        for (gpu_texture, _, vertex_start, vertex_count) in draw_calls {
            use crate::libs::graphics::backend::types::PrimitiveTopology;

            if let Some(gpu_tex) = gpu_texture {
                self.backend.bind_texture(gpu_tex, 0)?;
            }

            let sprite_count = vertex_count / 4;
            let index_start = (vertex_start / 4) * 6;
            let index_count = sprite_count * 6;

            self.backend.draw_indexed(
                PrimitiveTopology::Triangles,
                index_count as u32,
                index_start,
            )?;
        }

        Ok(())
    }
}
