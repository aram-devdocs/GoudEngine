//! Mutation, direction, point-transform, and interpolation operations for [`Transform`].

use crate::core::math::Vec3;
use crate::ecs::components::transform::core::Transform;
use crate::ecs::components::transform::quat::Quat;

impl Transform {
    // =========================================================================
    // Position Methods
    // =========================================================================

    /// Translates the transform by the given offset.
    #[inline]
    pub fn translate(&mut self, offset: Vec3) {
        self.position = self.position + offset;
    }

    /// Translates the transform in local space.
    ///
    /// The offset is rotated by the transform's rotation before being applied.
    #[inline]
    pub fn translate_local(&mut self, offset: Vec3) {
        let world_offset = self.rotation.rotate_vector(offset);
        self.position = self.position + world_offset;
    }

    /// Sets the position of the transform.
    #[inline]
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    // =========================================================================
    // Rotation Methods
    // =========================================================================

    /// Rotates the transform by the given quaternion.
    ///
    /// The rotation is applied in world space (rotation is applied after the current rotation).
    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = (self.rotation * rotation).normalize();
    }

    /// Rotates around the X axis by the given angle in radians.
    #[inline]
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_x(), angle));
    }

    /// Rotates around the Y axis by the given angle in radians.
    #[inline]
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_y(), angle));
    }

    /// Rotates around the Z axis by the given angle in radians.
    #[inline]
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_z(), angle));
    }

    /// Rotates around an arbitrary axis by the given angle in radians.
    #[inline]
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis.normalize(), angle));
    }

    /// Rotates in local space around the X axis.
    #[inline]
    pub fn rotate_local_x(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_x(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Rotates in local space around the Y axis.
    #[inline]
    pub fn rotate_local_y(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_y(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Rotates in local space around the Z axis.
    #[inline]
    pub fn rotate_local_z(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_z(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Sets the rotation from Euler angles (in radians).
    #[inline]
    pub fn set_rotation_euler(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation = Quat::from_euler(pitch, yaw, roll);
    }

    /// Sets the rotation.
    #[inline]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation.normalize();
    }

    /// Makes the transform look at a target position.
    #[inline]
    pub fn look_at_target(&mut self, target: Vec3, up: Vec3) {
        let looking = Transform::look_at(self.position, target, up);
        self.rotation = looking.rotation;
    }

    // =========================================================================
    // Scale Methods
    // =========================================================================

    /// Sets the scale of the transform.
    #[inline]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// Sets uniform scale on all axes.
    #[inline]
    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.scale = Vec3::new(scale, scale, scale);
    }

    /// Multiplies the current scale by the given factors.
    #[inline]
    pub fn scale_by(&mut self, factors: Vec3) {
        self.scale = Vec3::new(
            self.scale.x * factors.x,
            self.scale.y * factors.y,
            self.scale.z * factors.z,
        );
    }

    // =========================================================================
    // Direction Vectors
    // =========================================================================

    /// Returns the forward direction vector (negative Z in local space).
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation.forward()
    }

    /// Returns the right direction vector (positive X in local space).
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.right()
    }

    /// Returns the up direction vector (positive Y in local space).
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.up()
    }

    /// Returns the back direction vector (positive Z in local space).
    #[inline]
    pub fn back(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(0.0, 0.0, 1.0))
    }

    /// Returns the left direction vector (negative X in local space).
    #[inline]
    pub fn left(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(-1.0, 0.0, 0.0))
    }

    /// Returns the down direction vector (negative Y in local space).
    #[inline]
    pub fn down(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(0.0, -1.0, 0.0))
    }

    // =========================================================================
    // Point Transformation
    // =========================================================================

    /// Transforms a point from local space to world space.
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let scaled = Vec3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );
        let rotated = self.rotation.rotate_vector(scaled);
        rotated + self.position
    }

    /// Transforms a direction from local space to world space.
    ///
    /// Unlike points, directions are not affected by translation.
    #[inline]
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation.rotate_vector(direction)
    }

    /// Transforms a point from world space to local space.
    #[inline]
    pub fn inverse_transform_point(&self, point: Vec3) -> Vec3 {
        let translated = point - self.position;
        let inv_rotation = self.rotation.inverse();
        let rotated = inv_rotation.rotate_vector(translated);
        Vec3::new(
            rotated.x / self.scale.x,
            rotated.y / self.scale.y,
            rotated.z / self.scale.z,
        )
    }

    /// Transforms a direction from world space to local space.
    #[inline]
    pub fn inverse_transform_direction(&self, direction: Vec3) -> Vec3 {
        let inv_rotation = self.rotation.inverse();
        inv_rotation.rotate_vector(direction)
    }

    // =========================================================================
    // Interpolation
    // =========================================================================

    /// Linearly interpolates between two transforms.
    ///
    /// Position and scale are linearly interpolated, rotation uses slerp.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}
