//! # SDK Rendering API
//!
//! Provides methods on [`GoudGame`] for 2D rendering operations
//! including frame management, immediate-mode sprite/quad drawing, text,
//! and render state control.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop windowed rendering).

pub(crate) mod immediate;

use super::GoudGame;
use crate::assets::{
    loaders::{MaterialAsset, ShaderAsset},
    AssetHandle,
};
use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::render_backend::RenderTargetOps;
use crate::libs::graphics::backend::render_backend::TextureOps;
use crate::libs::graphics::backend::types::{
    RenderTargetDesc as BackendRenderTargetDesc, RenderTargetHandle as BackendRenderTargetHandle,
    TextureFormat, TextureHandle as BackendTextureHandle,
};

pub use immediate::ImmediateRenderState;

// Re-export rendering types for SDK users
pub use crate::rendering::sprite_batch::SpriteBatchConfig;
pub use crate::rendering::{RenderViewport, ViewportScaleMode};

// Re-export 3D types (they live in rendering_3d but users expect them here)
pub use crate::libs::graphics::renderer3d::{
    FogConfig, GridConfig, GridRenderMode, Light, LightType, PrimitiveCreateInfo, PrimitiveType,
    SkyboxConfig,
};

/// Native 2D renderer facade for sprite batching, immediate drawing, and text.
pub struct Renderer2D<'a> {
    game: &'a mut GoudGame,
}

impl<'a> Renderer2D<'a> {
    fn new(game: &'a mut GoudGame) -> Self {
        Self { game }
    }

    /// Begins a 2D sprite batch pass.
    pub fn begin(&mut self) -> GoudResult<()> {
        self.game.begin_2d_render()
    }

    /// Ends a 2D sprite batch pass.
    pub fn end(&mut self) -> GoudResult<()> {
        self.game.end_2d_render()
    }

    /// Draws ECS-managed sprites through the active 2D renderer.
    pub fn draw_sprites(&mut self) -> GoudResult<()> {
        self.game.draw_sprites()
    }

    /// Returns 2D batch statistics.
    pub fn stats(&self) -> (usize, usize, f32) {
        self.game.render_2d_stats()
    }

    /// Returns `true` when a native 2D renderer is available.
    pub fn is_available(&self) -> bool {
        self.game.has_2d_renderer()
    }

    /// Sets how the logical 2D viewport maps to the framebuffer.
    pub fn set_viewport_scale_mode(&mut self, mode: ViewportScaleMode) {
        self.game.set_viewport_scale_mode(mode);
    }

    /// Sets the logical design resolution used by the 2D viewport.
    pub fn set_design_resolution(&mut self, width: u32, height: u32) {
        self.game.set_design_resolution(width, height);
    }

    /// Returns the current resolved render viewport.
    pub fn viewport(&self) -> RenderViewport {
        self.game.render_viewport()
    }

    /// Creates an offscreen render target with an RGBA8 color attachment.
    pub fn create_render_target(
        &mut self,
        width: u32,
        height: u32,
        has_depth: bool,
    ) -> GoudResult<u64> {
        self.game.create_render_target(width, height, has_depth)
    }

    /// Binds an offscreen render target for subsequent draws.
    pub fn bind_render_target(&mut self, handle: u64) -> GoudResult<()> {
        self.game.bind_render_target(handle)
    }

    /// Restores drawing to the default framebuffer.
    pub fn bind_default_render_target(&mut self) -> GoudResult<()> {
        self.game.bind_default_render_target()
    }

    /// Destroys a render target and its owned attachments.
    pub fn destroy_render_target(&mut self, handle: u64) -> bool {
        self.game.destroy_render_target(handle)
    }

    /// Returns the color texture for a render target, packed as a texture handle.
    pub fn render_target_texture(&self, handle: u64) -> Option<u64> {
        self.game.render_target_texture(handle)
    }

    /// Sets the shader asset used by the native sprite batch.
    pub fn set_shader_asset(&mut self, shader: AssetHandle<ShaderAsset>) -> GoudResult<()> {
        self.game.set_2d_shader_asset(shader)
    }

    /// Sets the material asset used by the native sprite batch.
    pub fn set_material_asset(&mut self, material: AssetHandle<MaterialAsset>) -> GoudResult<()> {
        self.game.set_2d_material_asset(material)
    }

    /// Draws a textured quad immediately.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_sprite(
        &mut self,
        texture: u64,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        self.game
            .draw_sprite(texture, x, y, width, height, rotation, r, g, b, a)
    }

    /// Draws a solid-color quad immediately.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_quad(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        self.game.draw_quad(x, y, width, height, r, g, b, a)
    }

    /// Draws UTF-8 text immediately.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &mut self,
        font_path: &str,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        max_width: f32,
        line_spacing: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        self.game.draw_text(
            font_path,
            text,
            x,
            y,
            font_size,
            max_width,
            line_spacing,
            r,
            g,
            b,
            a,
        )
    }
}

// =============================================================================
// 2D Rendering -- ECS-based SpriteBatch (not FFI-generated)
// =============================================================================

impl GoudGame {
    /// Creates an offscreen render target with an RGBA8 color attachment.
    pub fn create_render_target(
        &mut self,
        width: u32,
        height: u32,
        has_depth: bool,
    ) -> GoudResult<u64> {
        let backend = self
            .render_backend
            .as_mut()
            .ok_or(GoudError::NotInitialized)?;
        let handle = backend.create_render_target(&BackendRenderTargetDesc {
            width: width.max(1),
            height: height.max(1),
            format: TextureFormat::RGBA8,
            has_depth,
        })?;
        if let Some(texture) = backend.render_target_texture(handle) {
            self.render_target_attachment_owners
                .insert(texture.to_u64(), handle.to_u64());
        }
        Ok(handle.to_u64())
    }

    /// Binds an offscreen render target for subsequent draws.
    pub fn bind_render_target(&mut self, handle: u64) -> GoudResult<()> {
        let backend_handle = BackendRenderTargetHandle::from_u64(handle);
        let backend = self
            .render_backend
            .as_mut()
            .ok_or(GoudError::NotInitialized)?;
        let texture = backend
            .render_target_texture(backend_handle)
            .ok_or(GoudError::InvalidHandle)?;
        let (width, height) = backend
            .texture_size(texture)
            .ok_or(GoudError::InvalidHandle)?;
        backend.bind_render_target(Some(backend_handle))?;
        self.bound_render_target_viewport =
            Some((handle, RenderViewport::fullscreen((width, height))));
        #[cfg(feature = "native")]
        self.apply_render_viewport();
        Ok(())
    }

    /// Restores drawing to the default framebuffer.
    pub fn bind_default_render_target(&mut self) -> GoudResult<()> {
        let backend = self
            .render_backend
            .as_mut()
            .ok_or(GoudError::NotInitialized)?;
        backend.bind_render_target(None)?;
        self.bound_render_target_viewport = None;
        #[cfg(feature = "native")]
        self.apply_render_viewport();
        Ok(())
    }

    /// Destroys a render target and its owned attachments.
    pub fn destroy_render_target(&mut self, handle: u64) -> bool {
        match self.render_backend.as_mut() {
            Some(backend) => {
                let destroyed =
                    backend.destroy_render_target(BackendRenderTargetHandle::from_u64(handle));
                if destroyed
                    && self
                        .bound_render_target_viewport
                        .map(|(bound_handle, _)| bound_handle)
                        == Some(handle)
                {
                    self.bound_render_target_viewport = None;
                    #[cfg(feature = "native")]
                    self.apply_render_viewport();
                }
                self.render_target_attachment_owners
                    .retain(|_, owner| *owner != handle);
                destroyed
            }
            None => false,
        }
    }

    /// Returns the color texture for a render target, packed as a texture handle.
    pub fn render_target_texture(&self, handle: u64) -> Option<u64> {
        self.render_backend.as_ref().and_then(|backend| {
            backend
                .render_target_texture(BackendRenderTargetHandle::from_u64(handle))
                .map(|texture| BackendTextureHandle::to_u64(&texture))
        })
    }

    /// Returns the current resolved render viewport.
    #[inline]
    pub fn render_viewport(&self) -> RenderViewport {
        #[cfg(feature = "native")]
        {
            self.bound_render_target_viewport
                .map(|(_, viewport)| viewport)
                .unwrap_or(self.render_viewport)
        }
        #[cfg(not(feature = "native"))]
        {
            RenderViewport::fullscreen(self.context.window_size())
        }
    }

    /// Sets the viewport scaling policy used for native rendering.
    pub fn set_viewport_scale_mode(&mut self, mode: ViewportScaleMode) {
        #[cfg(feature = "native")]
        {
            self.viewport_scale_mode = mode;
            let logical = self.get_window_size();
            let framebuffer = self.get_framebuffer_size();
            self.sync_render_viewport(logical, framebuffer);
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = mode;
        }
    }

    /// Returns the active viewport scaling policy.
    #[inline]
    pub fn viewport_scale_mode(&self) -> ViewportScaleMode {
        #[cfg(feature = "native")]
        {
            self.viewport_scale_mode
        }
        #[cfg(not(feature = "native"))]
        {
            ViewportScaleMode::Stretch
        }
    }

    /// Sets the logical design resolution used by 2D/UI rendering.
    pub fn set_design_resolution(&mut self, width: u32, height: u32) {
        #[cfg(feature = "native")]
        {
            self.design_resolution = (width.max(1), height.max(1));
            let logical = self.get_window_size();
            let framebuffer = self.get_framebuffer_size();
            self.sync_render_viewport(logical, framebuffer);
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = (width, height);
        }
    }

    /// Returns the native 2D renderer facade.
    #[inline]
    pub fn renderer_2d(&mut self) -> Renderer2D<'_> {
        Renderer2D::new(self)
    }

    /// Begins a 2D rendering pass.
    ///
    /// Call this before drawing sprites. Must be paired with
    /// [`end_2d_render`](Self::end_2d_render).
    pub fn begin_2d_render(&mut self) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => {
                batch.begin();
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Ends the 2D rendering pass and submits batched draw calls to the GPU.
    pub fn end_2d_render(&mut self) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => batch.end(),
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Draws all entities with Sprite + Transform2D components.
    pub fn draw_sprites(&mut self) -> GoudResult<()> {
        let default = self.scene_manager.default_scene();
        let world = self
            .scene_manager
            .get_scene(default)
            .ok_or(GoudError::NotInitialized)?;
        let asset_server = self
            .asset_server
            .as_mut()
            .ok_or(GoudError::NotInitialized)?;
        match &mut self.sprite_batch {
            Some(batch) => batch.draw_sprites(world, asset_server),
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Returns 2D rendering statistics: `(sprite_count, batch_count, batch_ratio)`.
    #[inline]
    pub fn render_2d_stats(&self) -> (usize, usize, f32) {
        match &self.sprite_batch {
            Some(batch) => batch.stats(),
            None => (0, 0, 0.0),
        }
    }

    /// Sets the shader asset used by the native sprite batch.
    pub fn set_2d_shader_asset(&mut self, shader: AssetHandle<ShaderAsset>) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => {
                batch.config.shader_asset = shader;
                batch.config.material_asset = AssetHandle::INVALID;
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Sets the material asset used by the native sprite batch.
    pub fn set_2d_material_asset(
        &mut self,
        material: AssetHandle<MaterialAsset>,
    ) -> GoudResult<()> {
        match &mut self.sprite_batch {
            Some(batch) => {
                batch.config.material_asset = material;
                Ok(())
            }
            None => Err(GoudError::NotInitialized),
        }
    }

    /// Returns `true` if a 2D renderer (SpriteBatch) is initialized.
    #[inline]
    pub fn has_2d_renderer(&self) -> bool {
        self.sprite_batch.is_some()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_begin_2d_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.begin_2d_render().is_err());
    }

    #[test]
    fn test_end_2d_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.end_2d_render().is_err());
    }

    #[test]
    fn test_draw_sprites_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(game.draw_sprites().is_err());
    }

    #[test]
    fn test_render_2d_stats_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.render_2d_stats(), (0, 0, 0.0));
    }

    #[test]
    fn test_has_2d_renderer_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.has_2d_renderer());
    }

    #[test]
    fn test_renderer_2d_facade_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let mut renderer = game.renderer_2d();
        assert!(!renderer.is_available());
        assert!(renderer.begin().is_err());
        assert!(renderer.create_render_target(64, 64, false).is_err());
        assert!(renderer.bind_render_target(1).is_err());
        assert!(renderer.bind_default_render_target().is_err());
        assert_eq!(renderer.render_target_texture(1), None);
        assert!(!renderer.destroy_render_target(1));
        assert_eq!(renderer.stats(), (0, 0, 0.0));
    }

    #[test]
    fn test_draw_sprite_headless_returns_false() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.draw_sprite(0, 0.0, 0.0, 10.0, 10.0, 0.0, 1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_draw_quad_headless_returns_false() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.draw_quad(0.0, 0.0, 10.0, 10.0, 1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_begin_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.begin_render());
    }

    #[test]
    fn test_end_render_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.end_render());
    }

    #[test]
    fn test_ortho_matrix_identity_like() {
        let m = immediate::ortho_matrix(0.0, 2.0, 0.0, 2.0);
        assert!((m[0] - 1.0).abs() < 0.001);
        assert!((m[5] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_model_matrix_no_rotation() {
        let m = immediate::model_matrix(10.0, 20.0, 5.0, 5.0, 0.0);
        assert!((m[12] - 10.0).abs() < 0.001);
        assert!((m[13] - 20.0).abs() < 0.001);
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_render_viewport_prefers_bound_render_target_override() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        game.bound_render_target_viewport = Some((7, RenderViewport::fullscreen((64, 32))));

        assert_eq!(game.render_viewport(), RenderViewport::fullscreen((64, 32)));
    }
}
