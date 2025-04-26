use super::Camera;
use crate::types::Camera3D;
use cgmath::{Matrix4, Point3, Vector3};

impl Camera3D {
    pub fn new() -> Self {
        Camera3D {
            position: Vector3::new(0.0, 0.0, 3.0),
            target: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            zoom: 1.0,
        }
    }

    // pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
    //     self.target = Vector3::new(x, y, z);
    // }

    // pub fn get_target(&self) -> Vector3<f32> {
    //     self.target
    // }

    // pub fn set_up(&mut self, x: f32, y: f32, z: f32) {
    //     self.up = Vector3::new(x, y, z);
    // }

    // pub fn get_up(&self) -> Vector3<f32> {
    //     self.up
    // }
}

impl Camera for Camera3D {
    fn get_view_matrix(&self) -> Matrix4<f32> {
        // The original implementation treats camera_zoom as the z-coordinate
        // and uses look_at_rh to place the camera at (x, y, zoom) looking at (0,0,0)
        let eye_position = Point3::new(
            self.position.x,
            self.position.y,
            self.zoom, // Direct z coordinate, not a scaling factor
        );

        Matrix4::look_at_rh(
            eye_position,
            Point3::new(self.target.x, self.target.y, self.target.z),
            self.up,
        )
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
        // In the original code, zoom was actually used as the z-coordinate
        self.zoom = zoom;
    }

    // fn get_zoom(&self) -> f32 {
    //     self.zoom
    // }
}
