use super::renderer2d::Renderer2D;
use super::renderer3d::Renderer3D;
use crate::types::{SpriteMap, TextureManager};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Renderer2D = 0,
    Renderer3D = 1,
}

/// Internal enum that owns the actual renderer.
/// This replaces the raw pointer approach for better safety.
enum RendererInner {
    Renderer2D(Box<Renderer2D>),
    Renderer3D(Box<Renderer3D>),
}

/// Wrapper type that provides a safe interface to either 2D or 3D renderer.
pub struct RendererType {
    pub kind: RendererKind,
    inner: RendererInner,
}

impl RendererType {
    pub fn new_2d(renderer: Renderer2D) -> Self {
        RendererType {
            kind: RendererKind::Renderer2D,
            inner: RendererInner::Renderer2D(Box::new(renderer)),
        }
    }

    pub fn new_3d(renderer: Renderer3D) -> Self {
        RendererType {
            kind: RendererKind::Renderer3D,
            inner: RendererInner::Renderer3D(Box::new(renderer)),
        }
    }

    /// Returns a mutable reference to the 2D renderer if this is a 2D renderer.
    #[allow(dead_code)]
    pub fn as_2d_mut(&mut self) -> Option<&mut Renderer2D> {
        match &mut self.inner {
            RendererInner::Renderer2D(r) => Some(r.as_mut()),
            _ => None,
        }
    }

    /// Returns a mutable reference to the 3D renderer if this is a 3D renderer.
    pub fn as_3d_mut(&mut self) -> Option<&mut Renderer3D> {
        match &mut self.inner {
            RendererInner::Renderer3D(r) => Some(r.as_mut()),
            _ => None,
        }
    }

    /// Returns an immutable reference to the 2D renderer if this is a 2D renderer.
    #[allow(dead_code)]
    pub fn as_2d(&self) -> Option<&Renderer2D> {
        match &self.inner {
            RendererInner::Renderer2D(r) => Some(r.as_ref()),
            _ => None,
        }
    }

    /// Returns an immutable reference to the 3D renderer if this is a 3D renderer.
    #[allow(dead_code)]
    pub fn as_3d(&self) -> Option<&Renderer3D> {
        match &self.inner {
            RendererInner::Renderer3D(r) => Some(r.as_ref()),
            _ => None,
        }
    }

    pub fn render(&mut self, sprites: &SpriteMap, texture_manager: &TextureManager) {
        match &mut self.inner {
            RendererInner::Renderer2D(r) => r.render(sprites, texture_manager),
            RendererInner::Renderer3D(r) => r.render(sprites, texture_manager),
        }
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32) {
        match &mut self.inner {
            RendererInner::Renderer2D(r) => r.set_camera_position(x, y),
            RendererInner::Renderer3D(r) => r.set_camera_position(x, y),
        }
    }

    pub fn set_camera_zoom(&mut self, zoom: f32) {
        match &mut self.inner {
            RendererInner::Renderer2D(r) => r.set_camera_zoom(zoom),
            RendererInner::Renderer3D(r) => r.set_camera_zoom(zoom),
        }
    }

    pub fn set_camera_position_3d(&mut self, x: f32, y: f32, z: f32) {
        match &mut self.inner {
            RendererInner::Renderer2D(r) => r.set_camera_position(x, y),
            RendererInner::Renderer3D(r) => r.set_camera_position_3d(x, y, z),
        }
    }

    pub fn get_camera_position(&self) -> cgmath::Vector3<f32> {
        match &self.inner {
            RendererInner::Renderer2D(r) => r.get_camera_position(),
            RendererInner::Renderer3D(r) => r.get_camera_position(),
        }
    }

    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        match &mut self.inner {
            RendererInner::Renderer2D(_) => {
                // 2D renderer ignores rotation
            }
            RendererInner::Renderer3D(r) => r.set_camera_rotation(pitch, yaw, roll),
        }
    }

    pub fn get_camera_rotation(&self) -> cgmath::Vector3<f32> {
        match &self.inner {
            RendererInner::Renderer2D(_) => {
                // 2D renderer doesn't have rotation
                cgmath::Vector3::new(0.0, 0.0, 0.0)
            }
            RendererInner::Renderer3D(r) => r.get_camera_rotation(),
        }
    }

    pub fn get_camera_zoom(&self) -> f32 {
        match &self.inner {
            RendererInner::Renderer2D(r) => r.get_camera_zoom(),
            RendererInner::Renderer3D(r) => r.get_camera_zoom(),
        }
    }

    pub fn terminate(&self) {
        match &self.inner {
            RendererInner::Renderer2D(r) => r.terminate(),
            RendererInner::Renderer3D(r) => r.terminate(),
        }
    }
}

// No need for Drop impl - Box handles cleanup automatically

pub trait Renderer {
    /// Renders the scene.
    // TODO: We need to abstract this so it works better for 3d
    fn render(&mut self, sprites: &SpriteMap, texture_manager: &TextureManager);

    // fn set_camera_position(&mut self, x: f32, y: f32);
    // fn set_camera_zoom(&mut self, zoom: f32);

    /// Terminates the renderer.
    fn terminate(&self);
}
