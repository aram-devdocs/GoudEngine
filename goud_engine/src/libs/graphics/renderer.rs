use super::renderer2d::Renderer2D;
use super::renderer3d::Renderer3D;
use crate::types::{SpriteMap, TextureManager};
use std::ptr;

#[repr(C)]
pub enum RendererKind {
    Renderer2D = 0,
    Renderer3D = 1,
}

#[repr(C)]
pub struct RendererType {
    pub kind: RendererKind,
    pub renderer_2d: *mut Renderer2D,
    pub renderer_3d: *mut Renderer3D,
}

impl RendererType {
    pub fn new_2d(renderer: Renderer2D) -> Self {
        let renderer_2d = Box::into_raw(Box::new(renderer));
        RendererType {
            kind: RendererKind::Renderer2D,
            renderer_2d,
            renderer_3d: ptr::null_mut(),
        }
    }

    pub fn new_3d(renderer: Renderer3D) -> Self {
        let renderer_3d = Box::into_raw(Box::new(renderer));
        RendererType {
            kind: RendererKind::Renderer3D,
            renderer_2d: ptr::null_mut(),
            renderer_3d,
        }
    }

    pub fn render(&mut self, sprites: SpriteMap, texture_manager: &TextureManager) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).render(sprites, texture_manager);
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).render(sprites, texture_manager);
                    }
                }
            }
        }
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).set_camera_position(x, y);
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).set_camera_position(x, y);
                    }
                }
            }
        }
    }

    pub fn set_camera_zoom(&mut self, zoom: f32) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).set_camera_zoom(zoom);
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).set_camera_zoom(zoom);
                    }
                }
            }
        }
    }

    pub fn set_camera_position_3d(&mut self, x: f32, y: f32, z: f32) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).set_camera_position(x, y);
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).set_camera_position_3d(x, y, z);
                    }
                }
            }
        }
    }

    pub fn get_camera_position(&self) -> cgmath::Vector3<f32> {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).get_camera_position()
                    } else {
                        cgmath::Vector3::new(0.0, 0.0, 0.0)
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).get_camera_position()
                    } else {
                        cgmath::Vector3::new(0.0, 0.0, 0.0)
                    }
                }
            }
        }
    }

    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    // 2D renderer ignores rotation
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).set_camera_rotation(pitch, yaw, roll);
                    }
                }
            }
        }
    }

    pub fn get_camera_rotation(&self) -> cgmath::Vector3<f32> {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    // 2D renderer doesn't have rotation
                    cgmath::Vector3::new(0.0, 0.0, 0.0)
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).get_camera_rotation()
                    } else {
                        cgmath::Vector3::new(0.0, 0.0, 0.0)
                    }
                }
            }
        }
    }

    pub fn get_camera_zoom(&self) -> f32 {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).get_camera_zoom()
                    } else {
                        1.0
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).get_camera_zoom()
                    } else {
                        1.0
                    }
                }
            }
        }
    }

    pub fn terminate(&self) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        (*self.renderer_2d).terminate();
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        (*self.renderer_3d).terminate();
                    }
                }
            }
        }
    }
}

impl Drop for RendererType {
    fn drop(&mut self) {
        unsafe {
            match self.kind {
                RendererKind::Renderer2D => {
                    if !self.renderer_2d.is_null() {
                        drop(Box::from_raw(self.renderer_2d));
                    }
                }
                RendererKind::Renderer3D => {
                    if !self.renderer_3d.is_null() {
                        drop(Box::from_raw(self.renderer_3d));
                    }
                }
            }
        }
    }
}

pub trait Renderer {
    /// Renders the scene.
    // TODO: We need to abstract this so it works better for 3d
    fn render(&mut self, sprites: SpriteMap, texture_manager: &TextureManager);

    // fn set_camera_position(&mut self, x: f32, y: f32);
    // fn set_camera_zoom(&mut self, zoom: f32);

    /// Terminates the renderer.
    fn terminate(&self);
}
