//! 3D physics provider trait definition.
//!
//! The `PhysicsProvider3D` trait abstracts the 3D physics engine, enabling
//! runtime selection between Rapier3D or null (no-op).

use super::types::{
    BodyDesc3D, BodyHandle, ColliderDesc3D, ColliderHandle, CollisionEvent, ContactPair3D,
    DebugShape3D, JointDesc3D, JointHandle, PhysicsCapabilities3D, RaycastHit3D,
};
use super::{Provider, ProviderLifecycle};
use crate::core::error::GoudResult;

/// Trait for 3D physics backends.
///
/// Uses `[f32; 3]` for Vec3 and `[f32; 4]` for quaternion parameters to avoid
/// depending on external math types. Concrete implementations convert to/from
/// their internal vector types.
///
/// The trait is object-safe and stored as `Box<dyn PhysicsProvider3D>`.
pub trait PhysicsProvider3D: Provider + ProviderLifecycle {
    /// Returns the typed physics capabilities for this provider.
    fn physics_capabilities(&self) -> &PhysicsCapabilities3D;

    // -------------------------------------------------------------------------
    // Simulation
    // -------------------------------------------------------------------------

    /// Advance the physics simulation by `delta` seconds.
    fn step(&mut self, delta: f32) -> GoudResult<()>;

    /// Set the global gravity vector as [x, y, z].
    fn set_gravity(&mut self, gravity: [f32; 3]);

    /// Get the current gravity vector as [x, y, z].
    fn gravity(&self) -> [f32; 3];

    // -------------------------------------------------------------------------
    // Body Management
    // -------------------------------------------------------------------------

    /// Create a physics body from a descriptor.
    fn create_body(&mut self, desc: &BodyDesc3D) -> GoudResult<BodyHandle>;

    /// Destroy a previously created body.
    fn destroy_body(&mut self, handle: BodyHandle);

    /// Get the position of a body as [x, y, z].
    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 3]>;

    /// Set the position of a body.
    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 3]) -> GoudResult<()>;

    /// Get the rotation of a body as quaternion [x, y, z, w].
    fn body_rotation(&self, handle: BodyHandle) -> GoudResult<[f32; 4]>;

    /// Set the rotation of a body as quaternion [x, y, z, w].
    fn set_body_rotation(&mut self, handle: BodyHandle, rot: [f32; 4]) -> GoudResult<()>;

    /// Get the velocity of a body as [x, y, z].
    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 3]>;

    /// Set the velocity of a body.
    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 3]) -> GoudResult<()>;

    /// Apply a force to a body (accumulated over the frame).
    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 3]) -> GoudResult<()>;

    /// Apply an instantaneous impulse to a body.
    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 3]) -> GoudResult<()>;

    /// Get the gravity scale of a body (default 1.0).
    fn body_gravity_scale(&self, handle: BodyHandle) -> GoudResult<f32>;

    /// Set the gravity scale of a body.
    fn set_body_gravity_scale(&mut self, handle: BodyHandle, scale: f32) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Collider Management
    // -------------------------------------------------------------------------

    /// Attach a collider to a body.
    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc3D,
    ) -> GoudResult<ColliderHandle>;

    /// Destroy a previously created collider.
    fn destroy_collider(&mut self, handle: ColliderHandle);

    /// Get the friction coefficient of a collider.
    fn collider_friction(&self, handle: ColliderHandle) -> GoudResult<f32>;

    /// Set the friction coefficient of a collider.
    fn set_collider_friction(&mut self, handle: ColliderHandle, friction: f32) -> GoudResult<()>;

    /// Get the restitution (bounciness) of a collider.
    fn collider_restitution(&self, handle: ColliderHandle) -> GoudResult<f32>;

    /// Set the restitution (bounciness) of a collider.
    fn set_collider_restitution(
        &mut self,
        handle: ColliderHandle,
        restitution: f32,
    ) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Cast a ray and return the first hit, if any.
    ///
    /// `origin` and `dir` are [x, y, z] arrays. `max_dist` is the maximum
    /// ray length.
    fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<RaycastHit3D>;

    /// Find all bodies whose colliders overlap the given sphere.
    fn overlap_sphere(&self, center: [f32; 3], radius: f32) -> Vec<BodyHandle>;

    // -------------------------------------------------------------------------
    // Collision Events
    // -------------------------------------------------------------------------

    /// Drain and return all collision events since the last call.
    ///
    /// Returns owned `Vec` rather than a slice to avoid lifetime coupling
    /// between the event buffer and the provider borrow.
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent>;

    /// Return all current contact pairs.
    fn contact_pairs(&self) -> Vec<ContactPair3D>;

    // -------------------------------------------------------------------------
    // Joints
    // -------------------------------------------------------------------------

    /// Create a joint connecting two bodies.
    fn create_joint(&mut self, desc: &JointDesc3D) -> GoudResult<JointHandle>;

    /// Destroy a previously created joint.
    fn destroy_joint(&mut self, handle: JointHandle);

    // -------------------------------------------------------------------------
    // Debug
    // -------------------------------------------------------------------------

    /// Return debug visualization shapes for rendering physics debug overlays.
    fn debug_shapes(&self) -> Vec<DebugShape3D>;
}
