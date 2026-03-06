//! Null physics provider -- silent no-op for headless testing.

use crate::libs::error::GoudResult;
use crate::libs::providers::physics::PhysicsProvider;
use crate::libs::providers::types::{
    BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, ContactPair, DebugShape,
    JointDesc, JointHandle, PhysicsCapabilities, RaycastHit,
};
use crate::libs::providers::{Provider, ProviderLifecycle};

/// A physics provider that does nothing. Used for headless testing and as
/// a default when no physics backend is needed.
pub struct NullPhysicsProvider {
    capabilities: PhysicsCapabilities,
    gravity: [f32; 2],
}

impl NullPhysicsProvider {
    /// Create a new null physics provider.
    pub fn new() -> Self {
        Self {
            capabilities: PhysicsCapabilities {
                supports_continuous_collision: false,
                supports_joints: false,
                max_bodies: 0,
            },
            gravity: [0.0, 0.0],
        }
    }
}

impl Default for NullPhysicsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullPhysicsProvider {
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

impl ProviderLifecycle for NullPhysicsProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider for NullPhysicsProvider {
    fn physics_capabilities(&self) -> &PhysicsCapabilities {
        &self.capabilities
    }

    fn step(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 2]) {
        self.gravity = gravity;
    }

    fn gravity(&self) -> [f32; 2] {
        self.gravity
    }

    fn create_body(&mut self, _desc: &BodyDesc) -> GoudResult<BodyHandle> {
        Ok(BodyHandle(0))
    }

    fn destroy_body(&mut self, _handle: BodyHandle) {}

    fn body_position(&self, _handle: BodyHandle) -> GoudResult<[f32; 2]> {
        Ok([0.0, 0.0])
    }

    fn set_body_position(&mut self, _handle: BodyHandle, _pos: [f32; 2]) -> GoudResult<()> {
        Ok(())
    }

    fn body_velocity(&self, _handle: BodyHandle) -> GoudResult<[f32; 2]> {
        Ok([0.0, 0.0])
    }

    fn set_body_velocity(&mut self, _handle: BodyHandle, _vel: [f32; 2]) -> GoudResult<()> {
        Ok(())
    }

    fn apply_force(&mut self, _handle: BodyHandle, _force: [f32; 2]) -> GoudResult<()> {
        Ok(())
    }

    fn apply_impulse(&mut self, _handle: BodyHandle, _impulse: [f32; 2]) -> GoudResult<()> {
        Ok(())
    }

    fn create_collider(
        &mut self,
        _body: BodyHandle,
        _desc: &ColliderDesc,
    ) -> GoudResult<ColliderHandle> {
        Ok(ColliderHandle(0))
    }

    fn destroy_collider(&mut self, _handle: ColliderHandle) {}

    fn raycast(&self, _origin: [f32; 2], _dir: [f32; 2], _max_dist: f32) -> Option<RaycastHit> {
        None
    }

    fn overlap_circle(&self, _center: [f32; 2], _radius: f32) -> Vec<BodyHandle> {
        Vec::new()
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        Vec::new()
    }

    fn contact_pairs(&self) -> Vec<ContactPair> {
        Vec::new()
    }

    fn create_joint(&mut self, _desc: &JointDesc) -> GoudResult<JointHandle> {
        Ok(JointHandle(0))
    }

    fn destroy_joint(&mut self, _handle: JointHandle) {}

    fn debug_shapes(&self) -> Vec<DebugShape> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_physics_construction() {
        let provider = NullPhysicsProvider::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_physics_default() {
        let provider = NullPhysicsProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_physics_init_shutdown() {
        let mut provider = NullPhysicsProvider::new();
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_physics_capabilities() {
        let provider = NullPhysicsProvider::new();
        let caps = provider.physics_capabilities();
        assert!(!caps.supports_continuous_collision);
        assert!(!caps.supports_joints);
        assert_eq!(caps.max_bodies, 0);
    }

    #[test]
    fn test_null_physics_gravity() {
        let mut provider = NullPhysicsProvider::new();
        assert_eq!(provider.gravity(), [0.0, 0.0]);
        provider.set_gravity([0.0, -9.81]);
        assert_eq!(provider.gravity(), [0.0, -9.81]);
    }

    #[test]
    fn test_null_physics_step() {
        let mut provider = NullPhysicsProvider::new();
        assert!(provider.step(0.016).is_ok());
    }

    #[test]
    fn test_null_physics_body_operations() {
        let mut provider = NullPhysicsProvider::new();
        let body = provider.create_body(&BodyDesc::default()).unwrap();
        assert_eq!(body, BodyHandle(0));

        assert_eq!(provider.body_position(body).unwrap(), [0.0, 0.0]);
        assert_eq!(provider.body_velocity(body).unwrap(), [0.0, 0.0]);
        assert!(provider.set_body_position(body, [1.0, 2.0]).is_ok());
        assert!(provider.set_body_velocity(body, [3.0, 4.0]).is_ok());
        assert!(provider.apply_force(body, [1.0, 0.0]).is_ok());
        assert!(provider.apply_impulse(body, [0.0, 1.0]).is_ok());

        provider.destroy_body(body);
    }

    #[test]
    fn test_null_physics_collider_operations() {
        let mut provider = NullPhysicsProvider::new();
        let body = provider.create_body(&BodyDesc::default()).unwrap();
        let collider = provider
            .create_collider(body, &ColliderDesc::default())
            .unwrap();
        assert_eq!(collider, ColliderHandle(0));
        provider.destroy_collider(collider);
    }

    #[test]
    fn test_null_physics_queries() {
        let provider = NullPhysicsProvider::new();
        assert!(provider.raycast([0.0, 0.0], [1.0, 0.0], 100.0).is_none());
        assert!(provider.overlap_circle([0.0, 0.0], 10.0).is_empty());
    }

    #[test]
    fn test_null_physics_events() {
        let mut provider = NullPhysicsProvider::new();
        assert!(provider.drain_collision_events().is_empty());
        assert!(provider.contact_pairs().is_empty());
    }

    #[test]
    fn test_null_physics_joints() {
        let mut provider = NullPhysicsProvider::new();
        let joint = provider.create_joint(&JointDesc::default()).unwrap();
        assert_eq!(joint, JointHandle(0));
        provider.destroy_joint(joint);
    }

    #[test]
    fn test_null_physics_debug_shapes() {
        let provider = NullPhysicsProvider::new();
        assert!(provider.debug_shapes().is_empty());
    }
}
