//! Simple 2D physics provider with AABB collision and gravity.

use std::collections::{HashMap, HashSet};

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::diagnostics::PhysicsDiagnosticsV1;
use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::{
    BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, ContactPair, DebugShape,
    JointDesc, JointHandle, PhysicsCapabilities, RaycastHit,
};
use crate::core::providers::{Provider, ProviderLifecycle};

mod contacts;
mod geometry;
#[cfg(test)]
mod tests;

use geometry::{circle_overlaps_aabb, collider_half_extents, raycast_aabb};

#[derive(Debug, Clone)]
struct SimpleBody {
    position: [f32; 2],
    velocity: [f32; 2],
    force: [f32; 2],
    body_type: u32,
    gravity_scale: f32,
}

#[derive(Debug, Clone)]
struct SimpleCollider {
    body: BodyHandle,
    desc: ColliderDesc,
}

#[derive(Debug, Clone, Copy)]
struct Aabb {
    min: [f32; 2],
    max: [f32; 2],
}

impl Aabb {
    fn center(self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }

    fn half_extents(self) -> [f32; 2] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
        ]
    }
}

/// A lightweight physics provider for simple 2D games.
///
/// Intended for small scenes. Broad-phase collision detection uses a pairwise
/// O(n^2) scan. Restitution is stored on colliders, but this simple solver
/// does not apply it.
pub struct SimplePhysicsProvider {
    capabilities: PhysicsCapabilities,
    gravity: [f32; 2],
    timestep: f32,
    next_body_id: u64,
    next_collider_id: u64,
    bodies: HashMap<u64, SimpleBody>,
    colliders: HashMap<u64, SimpleCollider>,
    previous_overlaps: HashSet<(u64, u64)>,
    collision_events: Vec<CollisionEvent>,
    contacts: Vec<ContactPair>,
}

impl SimplePhysicsProvider {
    /// Creates a simple 2D physics provider with the given gravity vector.
    pub fn new(gravity: [f32; 2]) -> Self {
        Self {
            capabilities: PhysicsCapabilities {
                supports_continuous_collision: false,
                supports_joints: false,
                max_bodies: u32::MAX,
            },
            gravity,
            timestep: 1.0 / 60.0,
            next_body_id: 1,
            next_collider_id: 1,
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            previous_overlaps: HashSet::new(),
            collision_events: Vec::new(),
            contacts: Vec::new(),
        }
    }

    fn require_body(&self, handle: BodyHandle) -> GoudResult<&SimpleBody> {
        self.bodies.get(&handle.0).ok_or(GoudError::InvalidHandle)
    }

    fn require_body_mut(&mut self, handle: BodyHandle) -> GoudResult<&mut SimpleBody> {
        self.bodies
            .get_mut(&handle.0)
            .ok_or(GoudError::InvalidHandle)
    }

    fn require_collider(&self, handle: ColliderHandle) -> GoudResult<&SimpleCollider> {
        self.colliders
            .get(&handle.0)
            .ok_or(GoudError::InvalidHandle)
    }

    fn body_aabb(&self, collider: &SimpleCollider) -> GoudResult<Aabb> {
        let body = self.require_body(collider.body)?;
        let half_extents = collider_half_extents(&collider.desc);
        Ok(Aabb {
            min: [
                body.position[0] - half_extents[0],
                body.position[1] - half_extents[1],
            ],
            max: [
                body.position[0] + half_extents[0],
                body.position[1] + half_extents[1],
            ],
        })
    }

    fn rebuild_contacts(&mut self) {
        contacts::rebuild_contacts(self);
    }
}

impl Default for SimplePhysicsProvider {
    fn default() -> Self {
        Self::new([0.0, -9.81])
    }
}

impl Provider for SimplePhysicsProvider {
    fn name(&self) -> &str {
        "simple"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for SimplePhysicsProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.step(delta)
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider for SimplePhysicsProvider {
    fn physics_capabilities(&self) -> &PhysicsCapabilities {
        &self.capabilities
    }

    fn step(&mut self, delta: f32) -> GoudResult<()> {
        for body in self.bodies.values_mut() {
            match body.body_type {
                1 => {
                    body.velocity[0] +=
                        (body.force[0] + self.gravity[0] * body.gravity_scale) * delta;
                    body.velocity[1] +=
                        (body.force[1] + self.gravity[1] * body.gravity_scale) * delta;
                    body.position[0] += body.velocity[0] * delta;
                    body.position[1] += body.velocity[1] * delta;
                }
                2 => {
                    body.position[0] += body.velocity[0] * delta;
                    body.position[1] += body.velocity[1] * delta;
                }
                _ => {}
            }
            body.force = [0.0, 0.0];
        }
        self.rebuild_contacts();
        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 2]) {
        self.gravity = gravity;
    }

    fn gravity(&self) -> [f32; 2] {
        self.gravity
    }

    fn set_timestep(&mut self, dt: f32) {
        self.timestep = dt;
    }

    fn timestep(&self) -> f32 {
        self.timestep
    }

    fn create_body(&mut self, desc: &BodyDesc) -> GoudResult<BodyHandle> {
        let id = self.next_body_id;
        self.next_body_id += 1;
        self.bodies.insert(
            id,
            SimpleBody {
                position: desc.position,
                velocity: [0.0, 0.0],
                force: [0.0, 0.0],
                body_type: desc.body_type,
                gravity_scale: desc.gravity_scale,
            },
        );
        Ok(BodyHandle(id))
    }

    fn destroy_body(&mut self, handle: BodyHandle) {
        self.bodies.remove(&handle.0);
        self.colliders.retain(|_, collider| collider.body != handle);
        self.previous_overlaps
            .retain(|(left, right)| *left != handle.0 && *right != handle.0);
    }

    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
        Ok(self.require_body(handle)?.position)
    }

    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 2]) -> GoudResult<()> {
        self.require_body_mut(handle)?.position = pos;
        Ok(())
    }

    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
        Ok(self.require_body(handle)?.velocity)
    }

    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 2]) -> GoudResult<()> {
        self.require_body_mut(handle)?.velocity = vel;
        Ok(())
    }

    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 2]) -> GoudResult<()> {
        let body = self.require_body_mut(handle)?;
        body.force[0] += force[0];
        body.force[1] += force[1];
        Ok(())
    }

    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 2]) -> GoudResult<()> {
        let body = self.require_body_mut(handle)?;
        body.velocity[0] += impulse[0];
        body.velocity[1] += impulse[1];
        Ok(())
    }

    fn body_gravity_scale(&self, handle: BodyHandle) -> GoudResult<f32> {
        Ok(self.require_body(handle)?.gravity_scale)
    }

    fn set_body_gravity_scale(&mut self, handle: BodyHandle, scale: f32) -> GoudResult<()> {
        self.require_body_mut(handle)?.gravity_scale = scale;
        Ok(())
    }

    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc,
    ) -> GoudResult<ColliderHandle> {
        self.require_body(body)?;
        let id = self.next_collider_id;
        self.next_collider_id += 1;
        self.colliders.insert(
            id,
            SimpleCollider {
                body,
                desc: desc.clone(),
            },
        );
        Ok(ColliderHandle(id))
    }

    fn destroy_collider(&mut self, handle: ColliderHandle) {
        self.colliders.remove(&handle.0);
    }

    fn collider_friction(&self, handle: ColliderHandle) -> GoudResult<f32> {
        Ok(self.require_collider(handle)?.desc.friction)
    }

    fn set_collider_friction(&mut self, handle: ColliderHandle, friction: f32) -> GoudResult<()> {
        self.colliders
            .get_mut(&handle.0)
            .ok_or(GoudError::InvalidHandle)?
            .desc
            .friction = friction;
        Ok(())
    }

    fn collider_restitution(&self, handle: ColliderHandle) -> GoudResult<f32> {
        Ok(self.require_collider(handle)?.desc.restitution)
    }

    fn set_collider_restitution(
        &mut self,
        handle: ColliderHandle,
        restitution: f32,
    ) -> GoudResult<()> {
        self.colliders
            .get_mut(&handle.0)
            .ok_or(GoudError::InvalidHandle)?
            .desc
            .restitution = restitution;
        Ok(())
    }

    fn raycast(&self, origin: [f32; 2], dir: [f32; 2], max_dist: f32) -> Option<RaycastHit> {
        self.raycast_with_mask(origin, dir, max_dist, u32::MAX)
    }

    fn raycast_with_mask(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
        layer_mask: u32,
    ) -> Option<RaycastHit> {
        let mut best_hit: Option<RaycastHit> = None;
        let mut best_distance = max_dist;
        for (collider_id, collider) in &self.colliders {
            if collider.desc.layer & layer_mask == 0 {
                continue;
            }
            let Ok(aabb) = self.body_aabb(collider) else {
                continue;
            };
            let Some((distance, normal)) = raycast_aabb(origin, dir, max_dist, aabb) else {
                continue;
            };
            if distance < best_distance {
                best_distance = distance;
                best_hit = Some(RaycastHit {
                    body: collider.body,
                    collider: ColliderHandle(*collider_id),
                    point: [origin[0] + dir[0] * distance, origin[1] + dir[1] * distance],
                    normal,
                    distance,
                });
            }
        }
        best_hit
    }

    fn overlap_circle(&self, center: [f32; 2], radius: f32) -> Vec<BodyHandle> {
        self.colliders
            .values()
            .filter_map(|collider| {
                let aabb = self.body_aabb(collider).ok()?;
                circle_overlaps_aabb(center, radius, aabb).then_some(collider.body)
            })
            .collect()
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        std::mem::take(&mut self.collision_events)
    }

    fn contact_pairs(&self) -> Vec<ContactPair> {
        self.contacts.clone()
    }

    fn create_joint(&mut self, _desc: &JointDesc) -> GoudResult<JointHandle> {
        Err(GoudError::ProviderError {
            subsystem: "physics",
            message: "simple physics provider does not support joints".to_string(),
        })
    }

    fn destroy_joint(&mut self, _handle: JointHandle) {}

    fn debug_shapes(&self) -> Vec<DebugShape> {
        self.colliders
            .values()
            .filter_map(|collider| {
                let aabb = self.body_aabb(collider).ok()?;
                let center = aabb.center();
                let size = aabb.half_extents();
                let color = if collider.desc.is_sensor {
                    [1.0, 1.0, 0.0, 0.5]
                } else if self.bodies.get(&collider.body.0)?.body_type == 0 {
                    [0.0, 1.0, 0.0, 0.5]
                } else {
                    [0.0, 0.0, 1.0, 0.5]
                };
                let debug_size = if collider.desc.shape == 0 {
                    [collider.desc.radius.max(0.5), collider.desc.radius.max(0.5)]
                } else {
                    [size[0] * 2.0, size[1] * 2.0]
                };
                Some(DebugShape {
                    shape_type: if collider.desc.shape == 0 { 0 } else { 1 },
                    position: center,
                    size: debug_size,
                    rotation: 0.0,
                    color,
                })
            })
            .collect()
    }

    fn physics_diagnostics(&self) -> PhysicsDiagnosticsV1 {
        PhysicsDiagnosticsV1 {
            body_count: self.bodies.len() as u32,
            collider_count: self.colliders.len() as u32,
            joint_count: 0,
            contact_pair_count: self.contacts.len() as u32,
            gravity: self.gravity,
            timestep: self.timestep,
        }
    }
}
