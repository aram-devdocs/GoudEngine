//! GPU resource management for the sprite batch renderer.

use super::batch::SpriteBatch;
use super::types::{SpriteVertex, TextureCacheEntry};
mod asset_support;
#[cfg(test)]
mod tests;

use crate::assets::loaders::{MaterialAsset, ShaderAsset, ShaderStage, TextureAsset};
use crate::assets::{AssetHandle, AssetServer};
use crate::core::error::{GoudError, GoudResult};
use crate::core::math::Vec2;
use crate::libs::graphics::backend::types::{
    BufferType, BufferUsage, TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};
use crate::libs::graphics::backend::RenderBackend;
pub(crate) use asset_support::{ensure_default_sprite_shader_loaded, ensure_sprite_asset_loaders};
use asset_support::{shader_signature, texture_signature};

impl<B: RenderBackend> SpriteBatch<B> {
    /// Ensures GPU resources (buffers, shader) are created.
    pub(super) fn ensure_resources(&mut self, asset_server: &mut AssetServer) -> GoudResult<()> {
        // Create vertex buffer if needed
        if self.vertex_buffer.is_none() || self.vertices.len() > self.vertex_capacity * 4 {
            self.create_vertex_buffer()?;
        }

        // Create index buffer if needed
        if self.index_buffer.is_none() {
            self.create_index_buffer()?;
        }

        // Create or refresh shader if needed
        if self.shader_needs_refresh(asset_server)? {
            self.create_shader(asset_server)?;
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
    pub(super) fn create_shader(&mut self, asset_server: &mut AssetServer) -> GoudResult<()> {
        let (shader_handle, material_asset) = self.resolve_shader_inputs(asset_server)?;
        let shader_asset = asset_server.get(&shader_handle).ok_or_else(|| {
            GoudError::ResourceNotFound(format!("Sprite shader asset {:?}", shader_handle))
        })?;

        if let Some(existing_shader) = self.shader.take() {
            let _ = self.backend.destroy_shader(existing_shader);
        }

        let vertex_source = shader_asset
            .get_stage(ShaderStage::Vertex)
            .ok_or_else(|| GoudError::ShaderCompilationFailed("Missing vertex stage".to_string()))?
            .source
            .as_str();
        let fragment_source = shader_asset
            .get_stage(ShaderStage::Fragment)
            .ok_or_else(|| {
                GoudError::ShaderCompilationFailed("Missing fragment stage".to_string())
            })?
            .source
            .as_str();

        let shader = self.backend.create_shader(vertex_source, fragment_source)?;
        self.shader = Some(shader);
        self.shader_signature = Some(shader_signature(shader_asset, material_asset.as_ref()));
        Ok(())
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
        let texture_asset = asset_server.get(&asset_handle).ok_or_else(|| {
            GoudError::ResourceNotFound(format!("Texture asset {:?}", asset_handle))
        })?;
        let signature = texture_signature(texture_asset);

        if let Some(cache_entry) = self.texture_cache.get(&asset_handle).copied() {
            if cache_entry.signature == signature
                && self.backend.is_texture_valid(cache_entry.handle)
            {
                return Ok(cache_entry.handle);
            }
            if self.backend.is_texture_valid(cache_entry.handle) {
                let _ = self.backend.destroy_texture(cache_entry.handle);
            }
            self.texture_cache.remove(&asset_handle);
        }

        let gpu_handle = self.backend.create_texture(
            texture_asset.width,
            texture_asset.height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &texture_asset.data,
        )?;
        self.texture_cache.insert(
            asset_handle,
            TextureCacheEntry {
                handle: gpu_handle,
                signature,
            },
        );
        Ok(gpu_handle)
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
    pub(super) fn render_batches(&mut self, asset_server: &mut AssetServer) -> GoudResult<()> {
        if self.batches.is_empty() {
            return Ok(());
        }

        // Ensure GPU resources are created
        self.ensure_resources(asset_server)?;

        // Upload vertex data
        self.upload_vertices()?;

        // Bind shader and set uniforms
        if let Some(shader) = self.shader {
            self.backend.bind_shader(shader)?;
            if let Some(location) = self.backend.get_uniform_location(shader, "u_texture") {
                self.backend.set_uniform_int(location, 0);
            }
            if let Some(location) = self.backend.get_uniform_location(shader, "u_viewport") {
                self.backend.set_uniform_vec2(
                    location,
                    self.viewport.logical_width.max(1) as f32,
                    self.viewport.logical_height.max(1) as f32,
                );
            }
            self.apply_material_uniforms(asset_server, shader)?;
        }

        self.backend.set_viewport(
            self.viewport.x,
            self.viewport.y,
            self.viewport.width.max(1),
            self.viewport.height.max(1),
        );

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

    fn resolve_shader_inputs(
        &self,
        asset_server: &mut AssetServer,
    ) -> GoudResult<(AssetHandle<ShaderAsset>, Option<MaterialAsset>)> {
        if self.config.material_asset.is_valid() {
            let shader_path = asset_server
                .get(&self.config.material_asset)
                .ok_or_else(|| {
                    GoudError::ResourceNotFound(format!(
                        "Sprite material asset {:?}",
                        self.config.material_asset
                    ))
                })?
                .shader_path()
                .to_string();
            let shader_handle = asset_server.load::<ShaderAsset>(&shader_path);
            let material = asset_server.get(&self.config.material_asset).cloned();
            return Ok((shader_handle, material));
        }

        let shader_handle = if self.config.shader_asset.is_valid() {
            self.config.shader_asset
        } else {
            ensure_default_sprite_shader_loaded(asset_server)
        };
        Ok((shader_handle, None))
    }

    fn shader_needs_refresh(&self, asset_server: &mut AssetServer) -> GoudResult<bool> {
        if self.shader.is_none() {
            return Ok(true);
        }

        let (shader_handle, material_asset) = self.resolve_shader_inputs(asset_server)?;
        let shader_asset = asset_server.get(&shader_handle).ok_or_else(|| {
            GoudError::ResourceNotFound(format!("Sprite shader asset {:?}", shader_handle))
        })?;
        let current_signature = shader_signature(shader_asset, material_asset.as_ref());
        Ok(self.shader_signature != Some(current_signature))
    }

    fn apply_material_uniforms(
        &mut self,
        asset_server: &mut AssetServer,
        shader: crate::libs::graphics::backend::types::ShaderHandle,
    ) -> GoudResult<()> {
        if !self.config.material_asset.is_valid() {
            return Ok(());
        }

        let material = asset_server
            .get(&self.config.material_asset)
            .ok_or_else(|| {
                GoudError::ResourceNotFound(format!(
                    "Sprite material asset {:?}",
                    self.config.material_asset
                ))
            })?;

        for (name, value) in material.uniforms() {
            let Some(location) = self.backend.get_uniform_location(shader, name) else {
                continue;
            };

            match value {
                crate::assets::loaders::UniformValue::Float(value) => {
                    self.backend.set_uniform_float(location, *value);
                }
                crate::assets::loaders::UniformValue::Vec2(value) => {
                    self.backend.set_uniform_vec2(location, value[0], value[1]);
                }
                crate::assets::loaders::UniformValue::Vec3(value) => {
                    self.backend
                        .set_uniform_vec3(location, value[0], value[1], value[2]);
                }
                crate::assets::loaders::UniformValue::Vec4(value) => {
                    self.backend
                        .set_uniform_vec4(location, value[0], value[1], value[2], value[3]);
                }
                crate::assets::loaders::UniformValue::Int(value) => {
                    self.backend.set_uniform_int(location, *value);
                }
                crate::assets::loaders::UniformValue::Mat4(value) => {
                    let flattened = [
                        value[0][0],
                        value[0][1],
                        value[0][2],
                        value[0][3],
                        value[1][0],
                        value[1][1],
                        value[1][2],
                        value[1][3],
                        value[2][0],
                        value[2][1],
                        value[2][2],
                        value[2][3],
                        value[3][0],
                        value[3][1],
                        value[3][2],
                        value[3][3],
                    ];
                    self.backend.set_uniform_mat4(location, &flattened);
                }
            }
        }

        Ok(())
    }
}
