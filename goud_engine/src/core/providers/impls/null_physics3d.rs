//! Null 3D physics provider -- silent no-op for headless testing.

use crate::core::error::GoudResult;
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{
    BodyDesc3D, BodyHandle, ColliderDesc3D, ColliderHandle, CollisionEvent, ContactPair3D,
    DebugShape3D, JointDesc3D, JointHandle, PhysicsCapabilities3D, RaycastHit3D,
};
use crate::core::providers::{Provider, ProviderLifecycle};

/// A 3D physics provider that does nothing. Used for headless testing and as
/// a default when no 3D physics backend is needed.
pub struct NullPhysicsProvider3D {
    capabilities: PhysicsCapabilities3D,
    gravity: [f32; 3],
    timestep: f32,
}

impl NullPhysicsProvider3D {
    /// Create a new null 3D physics provider.
    pub fn new() -> Self {
        Self {
            capabilities: PhysicsCapabilities3D {
                supports_continuous_collision: false,
                supports_joints: false,
                max_bodies: 0,
            },
            gravity: [0.0, 0.0, 0.0],
            timestep: 1.0 / 60.0,
        }
    }
}

impl Default for NullPhysicsProvider3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullPhysicsProvider3D {
    fn name(&self) -> &str {
        "null"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for NullPhysicsProvider3D {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider3D for NullPhysicsProvider3D {
    fn physics_capabilities(&self) -> &PhysicsCapabilities3D {
        &self.capabilities
    }

    fn step(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 3]) {
        self.gravity = gravity;
    }

    fn gravity(&self) -> [f32; 3] {
        self.gravity
    }

    fn set_timestep(&mut self, dt: f32) {
        self.timestep = dt;
    }

    fn timestep(&self) -> f32 {
        self.timestep
    }

    fn create_body(&mut self, _desc: &BodyDesc3D) -> GoudResult<BodyHandle> {
        Ok(BodyHandle(0))
    }

    fn destroy_body(&mut self, _handle: BodyHandle) {}

    fn body_position(&self, _handle: BodyHandle) -> GoudResult<[f32; 3]> {
        Ok([0.0, 0.0, 0.0])
    }

    fn set_body_position(&mut self, _handle: BodyHandle, _pos: [f32; 3]) -> GoudResult<()> {
        Ok(())
    }

    fn body_rotation(&self, _handle: BodyHandle) -> GoudResult<[f32; 4]> {
        Ok([0.0, 0.0, 0.0, 1.0])
    }

    fn set_body_rotation(&mut self, _handle: BodyHandle, _rot: [f32; 4]) -> GoudResult<()> {
        Ok(())
    }

    fn body_velocity(&self, _handle: BodyHandle) -> GoudResult<[f32; 3]> {
        Ok([0.0, 0.0, 0.0])
    }

    fn set_body_velocity(&mut self, _handle: BodyHandle, _vel: [f32; 3]) -> GoudResult<()> {
        Ok(())
    }

    fn apply_force(&mut self, _handle: BodyHandle, _force: [f32; 3]) -> GoudResult<()> {
        Ok(())
    }

    fn apply_impulse(&mut self, _handle: BodyHandle, _impulse: [f32; 3]) -> GoudResult<()> {
        Ok(())
    }

    fn body_gravity_scale(&self, _handle: BodyHandle) -> GoudResult<f32> {
        Ok(1.0)
    }

    fn set_body_gravity_scale(&mut self, _handle: BodyHandle, _scale: f32) -> GoudResult<()> {
        Ok(())
    }

    fn create_collider(
        &mut self,
        _body: BodyHandle,
        _desc: &ColliderDesc3D,
    ) -> GoudResult<ColliderHandle> {
        Ok(ColliderHandle(0))
    }

    fn destroy_collider(&mut self, _handle: ColliderHandle) {}

    fn collider_friction(&self, _handle: ColliderHandle) -> GoudResult<f32> {
        Ok(0.5)
    }

    fn set_collider_friction(&mut self, _handle: ColliderHandle, _friction: f32) -> GoudResult<()> {
        Ok(())
    }

    fn collider_restitution(&self, _handle: ColliderHandle) -> GoudResult<f32> {
        Ok(0.0)
    }

    fn set_collider_restitution(
        &mut self,
        _handle: ColliderHandle,
        _restitution: f32,
    ) -> GoudResult<()> {
        Ok(())
    }

    fn raycast(&self, _origin: [f32; 3], _dir: [f32; 3], _max_dist: f32) -> Option<RaycastHit3D> {
        None
    }

    fn overlap_sphere(&self, _center: [f32; 3], _radius: f32) -> Vec<BodyHandle> {
        Vec::new()
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        Vec::new()
    }

    fn contact_pairs(&self) -> Vec<ContactPair3D> {
        Vec::new()
    }

    fn create_joint(&mut self, _desc: &JointDesc3D) -> GoudResult<JointHandle> {
        Ok(JointHandle(0))
    }

    fn destroy_joint(&mut self, _handle: JointHandle) {}

    fn debug_shapes(&self) -> Vec<DebugShape3D> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_physics3d_construction() {
        let provider = NullPhysicsProvider3D::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_physics3d_default() {
        let provider = NullPhysicsProvider3D::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_physics3d_init_shutdown() {
        let mut provider = NullPhysicsProvider3D::new();
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_physics3d_capabilities() {
        let provider = NullPhysicsProvider3D::new();
        let caps = provider.physics_capabilities();
        assert!(!caps.supports_continuous_collision);
        assert!(!caps.supports_joints);
        assert_eq!(caps.max_bodies, 0);
    }

    #[test]
    fn test_null_physics3d_gravity() {
        let mut provider = NullPhysicsProvider3D::new();
        assert_eq!(provider.gravity(), [0.0, 0.0, 0.0]);
        provider.set_gravity([0.0, -9.81, 0.0]);
        assert_eq!(provider.gravity(), [0.0, -9.81, 0.0]);
    }

    #[test]
    fn test_null_physics3d_step() {
        let mut provider = NullPhysicsProvider3D::new();
        assert!(provider.step(0.016).is_ok());
    }

    #[test]
    fn test_null_physics3d_body_operations() {
        let mut provider = NullPhysicsProvider3D::new();
        let body = provider.create_body(&BodyDesc3D::default()).unwrap();
        assert_eq!(body, BodyHandle(0));

        assert_eq!(provider.body_position(body).unwrap(), [0.0, 0.0, 0.0]);
        assert_eq!(provider.body_rotation(body).unwrap(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(provider.body_velocity(body).unwrap(), [0.0, 0.0, 0.0]);
        assert!(provider.set_body_position(body, [1.0, 2.0, 3.0]).is_ok());
        assert!(provider
            .set_body_rotation(body, [0.0, 0.707, 0.0, 0.707])
            .is_ok());
        assert!(provider.set_body_velocity(body, [3.0, 4.0, 5.0]).is_ok());
        assert!(provider.apply_force(body, [1.0, 0.0, 0.0]).is_ok());
        assert!(provider.apply_impulse(body, [0.0, 1.0, 0.0]).is_ok());

        provider.destroy_body(body);
    }

    #[test]
    fn test_null_physics3d_collider_operations() {
        let mut provider = NullPhysicsProvider3D::new();
        let body = provider.create_body(&BodyDesc3D::default()).unwrap();
        let collider = provider
            .create_collider(body, &ColliderDesc3D::default())
            .unwrap();
        assert_eq!(collider, ColliderHandle(0));
        provider.destroy_collider(collider);
    }

    #[test]
    fn test_null_physics3d_queries() {
        let provider = NullPhysicsProvider3D::new();
        assert!(provider
            .raycast([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 100.0)
            .is_none());
        assert!(provider.overlap_sphere([0.0, 0.0, 0.0], 10.0).is_empty());
    }

    #[test]
    fn test_null_physics3d_events() {
        let mut provider = NullPhysicsProvider3D::new();
        assert!(provider.drain_collision_events().is_empty());
        assert!(provider.contact_pairs().is_empty());
    }

    #[test]
    fn test_null_physics3d_joints() {
        let mut provider = NullPhysicsProvider3D::new();
        let joint = provider.create_joint(&JointDesc3D::default()).unwrap();
        assert_eq!(joint, JointHandle(0));
        provider.destroy_joint(joint);
    }

    #[test]
    fn test_null_physics3d_debug_shapes() {
        let provider = NullPhysicsProvider3D::new();
        assert!(provider.debug_shapes().is_empty());
    }
}
