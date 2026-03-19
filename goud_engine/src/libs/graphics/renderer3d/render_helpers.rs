//! Render helper methods: post-processing, FXAA, shadow textures, and fullscreen blitting.

use super::core::Renderer3D;
use super::postprocess::apply_fxaa_like_filter;
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};
use crate::libs::graphics::backend::PrimitiveTopology;

impl Renderer3D {
    pub(super) fn update_shadow_texture(&mut self, rgba8: &[u8], width: u32, height: u32) {
        let recreate = self.shadow_texture.is_none();
        if recreate {
            self.shadow_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    rgba8,
                )
                .ok();
            return;
        }
        if let Some(texture) = self.shadow_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, rgba8);
        }
    }

    pub(super) fn apply_fxaa_pass(&mut self) {
        let width = self.viewport.2.max(1);
        let height = self.viewport.3.max(1);
        let Ok(frame) = self.backend.read_default_framebuffer_rgba8(width, height) else {
            return;
        };
        let filtered = apply_fxaa_like_filter(width, height, &frame);
        if self.postprocess_texture.is_none() || self.postprocess_texture_size != (width, height) {
            if let Some(texture) = self.postprocess_texture.take() {
                self.backend.destroy_texture(texture);
            }
            self.postprocess_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    &filtered,
                )
                .ok();
            self.postprocess_texture_size = (width, height);
        } else if let Some(texture) = self.postprocess_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, &filtered);
        }
        let Some(texture) = self.postprocess_texture else {
            return;
        };
        self.blit_fullscreen_texture(texture);
    }

    pub(super) fn apply_postprocess_pipeline(&mut self) {
        let width = self.viewport.2.max(1);
        let height = self.viewport.3.max(1);
        let Ok(mut frame) = self.backend.read_default_framebuffer_rgba8(width, height) else {
            return;
        };
        self.postprocess_pipeline.run(width, height, &mut frame);
        if self.postprocess_texture.is_none() || self.postprocess_texture_size != (width, height) {
            if let Some(texture) = self.postprocess_texture.take() {
                self.backend.destroy_texture(texture);
            }
            self.postprocess_texture = self
                .backend
                .create_texture(
                    width,
                    height,
                    TextureFormat::RGBA8,
                    TextureFilter::Linear,
                    TextureWrap::ClampToEdge,
                    &frame,
                )
                .ok();
            self.postprocess_texture_size = (width, height);
        } else if let Some(texture) = self.postprocess_texture {
            let _ = self
                .backend
                .update_texture(texture, 0, 0, width, height, &frame);
        }
        let Some(texture) = self.postprocess_texture else {
            return;
        };
        self.blit_fullscreen_texture(texture);
    }

    pub(super) fn blit_fullscreen_texture(&mut self, texture: TextureHandle) {
        self.backend.disable_depth_test();
        self.backend.disable_culling();
        self.backend.set_depth_mask(false);
        let _ = self.backend.bind_shader(self.postprocess_shader_handle);
        let _ = self.backend.bind_texture(texture, 0);
        if let Some(location) = self
            .backend
            .get_uniform_location(self.postprocess_shader_handle, "screenTexture")
        {
            self.backend.set_uniform_int(location, 0);
        }
        let _ = self.backend.bind_buffer(self.postprocess_quad_buffer);
        self.backend.set_vertex_attributes(&self.postprocess_layout);
        let _ = self.backend.draw_arrays(PrimitiveTopology::Triangles, 0, 6);
        self.backend.unbind_shader();
        self.backend.set_depth_mask(true);
        self.backend.enable_depth_test();
    }
}
