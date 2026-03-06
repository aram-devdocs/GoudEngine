//! Physics provider trait definition.
//!
//! The `PhysicsProvider` trait abstracts the physics engine, enabling
//! runtime selection between Rapier2D, a simple AABB engine, or null (no-op).

use super::types::{
    BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, ContactPair, DebugShape,
    JointDesc, JointHandle, PhysicsCapabilities, RaycastHit,
};
use super::{Provider, ProviderLifecycle};
use crate::libs::error::GoudResult;

/// Trait for physics backends.
///
/// Uses `[f32; 2]` for Vec2 parameters to avoid depending on external math
/// types. Concrete implementations convert to/from their internal vector types.
///
/// The trait is object-safe and stored as `Box<dyn PhysicsProvider>`.
pub trait PhysicsProvider: Provider + ProviderLifecycle {
    /// Returns the typed physics capabilities for this provider.
    fn physics_capabilities(&self) -> &PhysicsCapabilities;

    // -------------------------------------------------------------------------
    // Simulation
    // -------------------------------------------------------------------------

    /// Advance the physics simulation by `delta` seconds.
    fn step(&mut self, delta: f32) -> GoudResult<()>;

    /// Set the global gravity vector as [x, y].
    fn set_gravity(&mut self, gravity: [f32; 2]);

    /// Get the current gravity vector as [x, y].
    fn gravity(&self) -> [f32; 2];

    // -------------------------------------------------------------------------
    // Body Management
    // -------------------------------------------------------------------------

    /// Create a physics body from a descriptor.
    fn create_body(&mut self, desc: &BodyDesc) -> GoudResult<BodyHandle>;

    /// Destroy a previously created body.
    fn destroy_body(&mut self, handle: BodyHandle);

    /// Get the position of a body as [x, y].
    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 2]>;

    /// Set the position of a body.
    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 2]) -> GoudResult<()>;

    /// Get the velocity of a body as [x, y].
    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 2]>;

    /// Set the velocity of a body.
    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 2]) -> GoudResult<()>;

    /// Apply a force to a body (accumulated over the frame).
    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 2]) -> GoudResult<()>;

    /// Apply an instantaneous impulse to a body.
    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 2]) -> GoudResult<()>;

    // -------------------------------------------------------------------------
    // Collider Management
    // -------------------------------------------------------------------------

    /// Attach a collider to a body.
    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc,
    ) -> GoudResult<ColliderHandle>;

    /// Destroy a previously created collider.
    fn destroy_collider(&mut self, handle: ColliderHandle);

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Cast a ray and return the first hit, if any.
    ///
    /// `origin` and `dir` are [x, y] arrays. `max_dist` is the maximum
    /// ray length.
    fn raycast(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
    ) -> Option<RaycastHit>;

    /// Find all bodies whose colliders overlap the given circle.
    fn overlap_circle(&self, center: [f32; 2], radius: f32) -> Vec<BodyHandle>;

    // -------------------------------------------------------------------------
    // Collision Events
    // -------------------------------------------------------------------------

    /// Drain and return all collision events since the last call.
    ///
    /// Returns owned `Vec` rather than a slice to avoid lifetime coupling
    /// between the event buffer and the provider borrow.
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent>;

    /// Return all current contact pairs.
    fn contact_pairs(&self) -> Vec<ContactPair>;

    // -------------------------------------------------------------------------
    // Joints
    // -------------------------------------------------------------------------

    /// Create a joint connecting two bodies.
    fn create_joint(&mut self, desc: &JointDesc) -> GoudResult<JointHandle>;

    /// Destroy a previously created joint.
    fn destroy_joint(&mut self, handle: JointHandle);

    // -------------------------------------------------------------------------
    // Debug
    // -------------------------------------------------------------------------

    /// Return debug visualization shapes for rendering physics debug overlays.
    fn debug_shapes(&self) -> Vec<DebugShape>;
}
