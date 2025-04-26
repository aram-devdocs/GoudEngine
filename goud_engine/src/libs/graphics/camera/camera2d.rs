use super::Camera;
use crate::types::Camera2D;
use cgmath::{Matrix4, Vector3};

impl Camera2D {
    pub fn new() -> Self {
        Camera2D {
            position: Vector3::new(0.0, 0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Camera for Camera2D {
    fn get_view_matrix(&self) -> Matrix4<f32> {
        // For 2D, we simply translate by the negative position and scale by zoom
        Matrix4::from_translation(-self.position) * Matrix4::from_scale(self.zoom)
    }

    // fn set_position(&mut self, x: f32, y: f32, z: f32) {
    //     self.position = Vector3::new(x, y, z);
    // }

    fn set_position_xy(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    // fn get_zoom(&self) -> f32 {
    //     self.zoom
    // }
}
