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

    fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = Vector3::new(x, y, z);
    }

    fn set_position_xy(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_rotation(&mut self, _pitch: f32, _yaw: f32, _roll: f32) {
        // 2D cameras don't use rotation
    }

    fn get_rotation(&self) -> Vector3<f32> {
        // 2D cameras don't use rotation
        Vector3::new(0.0, 0.0, 0.0)
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    fn get_zoom(&self) -> f32 {
        self.zoom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera2d_new() {
        let camera = Camera2D::new();
        assert_eq!(camera.position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera2d_set_position() {
        let mut camera = Camera2D::new();
        camera.set_position(1.0, 2.0, 3.0);
        assert_eq!(camera.position, Vector3::new(1.0, 2.0, 3.0));
    }
    #[test]
    fn test_camera2d_set_position_xy() {
        let mut camera = Camera2D::new();
        camera.set_position_xy(1.0, 2.0);
        assert_eq!(camera.position.x, 1.0);
        assert_eq!(camera.position.y, 2.0);
        assert_eq!(camera.position.z, 0.0);
    }

    #[test]
    fn test_camera2d_get_position() {
        let mut camera = Camera2D::new();
        camera.set_position(1.0, 2.0, 3.0);
        assert_eq!(camera.get_position(), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_camera2d_set_rotation() {
        let mut camera = Camera2D::new();
        camera.set_rotation(1.0, 2.0, 3.0);
        // Rotation should remain unchanged as 2D cameras don't use rotation
        assert_eq!(camera.get_rotation(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_camera2d_get_rotation() {
        let camera = Camera2D::new();
        assert_eq!(camera.get_rotation(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_camera2d_set_zoom() {
        let mut camera = Camera2D::new();
        camera.set_zoom(2.0);
        assert_eq!(camera.zoom, 2.0);
    }

    #[test]
    fn test_camera2d_get_zoom() {
        let mut camera = Camera2D::new();
        camera.set_zoom(2.0);
        assert_eq!(camera.get_zoom(), 2.0);
    }
}
