//! Rapier3D physics provider implementation.
//!
//! Wraps the `rapier3d` crate behind the `PhysicsProvider3D` trait, providing
//! 3D rigid-body simulation, collision detection, raycasting, and joints.

mod conversions;
mod queries;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crossbeam_channel::Receiver;
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::diagnostics::Physics3DDiagnosticsV1;
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::CharacterControllerHandle;
use crate::core::providers::types::ColliderHandle as EngineColliderHandle;
use crate::core::providers::types::CollisionEvent as EngineCollisionEvent;
use crate::core::providers::types::{
    BodyDesc3D, CharacterControllerDesc3D, CharacterMoveResult3D, ColliderDesc3D,
    CollisionEventKind, ContactPair3D, DebugShape3D, JointDesc3D, PhysicsCapabilities3D,
    RaycastHit3D,
};
use crate::core::providers::types::{BodyHandle, JointHandle};
use crate::core::providers::{Provider, ProviderLifecycle};

use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::ColliderHandle as RapierColliderHandle;

use conversions::{body_type_from_u32, joint_from_desc, shape_from_desc};

/// Internal data for a character controller instance.
struct CharacterControllerData {
    /// The Rapier kinematic character controller.
    controller: KinematicCharacterController,
    /// Handle to the kinematic rigid body.
    body_handle: RigidBodyHandle,
    /// Handle to the capsule collider.
    collider_handle: RapierColliderHandle,
    /// The capsule shape used for move_shape queries.
    shape: SharedShape,
    /// Whether the character is currently touching the ground.
    grounded: bool,
    /// Accumulated vertical velocity for gravity.
    vertical_velocity: f32,
    /// Gravity magnitude (m/s^2, positive = downward).
    gravity: f32,
}

/// A 3D physics provider backed by Rapier3D.
pub struct Rapier3DPhysicsProvider {
    capabilities: PhysicsCapabilities3D,
    gravity: Vector<f32>,
    integration_params: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,

    // Handle mapping: engine u64 <-> rapier handles
    next_body_id: u64,
    next_collider_id: u64,
    next_joint_id: u64,
    body_map: HashMap<u64, RigidBodyHandle>,
    body_reverse: HashMap<RigidBodyHandle, u64>,
    collider_map: HashMap<u64, RapierColliderHandle>,
    collider_to_body: HashMap<RapierColliderHandle, RigidBodyHandle>,
    joint_map: HashMap<u64, ImpulseJointHandle>,

    // Character controllers
    next_controller_id: u64,
    controllers: HashMap<u64, CharacterControllerData>,

    // Collision event handling
    collision_events: Vec<EngineCollisionEvent>,
    collision_recv: Receiver<rapier3d::geometry::CollisionEvent>,
    // Stored to keep the channel alive; contact force events are drained
    // to prevent unbounded growth but processed via narrow_phase directly.
    _contact_recv: Receiver<ContactForceEvent>,
    event_handler: ChannelEventCollector,
}

impl Rapier3DPhysicsProvider {
    /// Create a new Rapier3D physics provider with default gravity [0, -9.81, 0].
    pub fn new() -> Self {
        let (collision_send, collision_recv) = crossbeam_channel::unbounded();
        let (contact_send, contact_recv) = crossbeam_channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_send);

        Self {
            capabilities: PhysicsCapabilities3D {
                supports_continuous_collision: true,
                supports_joints: true,
                max_bodies: u32::MAX,
            },
            gravity: vector![0.0, -9.81, 0.0],
            integration_params: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),

            next_body_id: 1,
            next_collider_id: 1,
            next_joint_id: 1,
            body_map: HashMap::new(),
            body_reverse: HashMap::new(),
            collider_map: HashMap::new(),
            collider_to_body: HashMap::new(),
            joint_map: HashMap::new(),

            next_controller_id: 1,
            controllers: HashMap::new(),

            collision_events: Vec::new(),
            collision_recv,
            _contact_recv: contact_recv,
            event_handler,
        }
    }

    /// Look up the rapier body handle for an engine handle, returning an error
    /// if not found.
    fn resolve_body(&self, handle: BodyHandle) -> GoudResult<RigidBodyHandle> {
        self.body_map
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }

    /// Look up the rapier collider handle, returning an error if not found.
    fn resolve_collider(&self, handle: EngineColliderHandle) -> GoudResult<RapierColliderHandle> {
        self.collider_map
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }
}

impl Default for Rapier3DPhysicsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for Rapier3DPhysicsProvider {
    fn name(&self) -> &str {
        "rapier3d"
    }

    fn version(&self) -> &str {
        "0.22"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for Rapier3DPhysicsProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.step(delta)
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider3D for Rapier3DPhysicsProvider {
    fn physics_capabilities(&self) -> &PhysicsCapabilities3D {
        &self.capabilities
    }

    fn step(&mut self, delta: f32) -> GoudResult<()> {
        self.integration_params.dt = delta;

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &self.event_handler,
        );

        // Drain collision events from the channel and convert to engine events
        while let Ok(event) = self.collision_recv.try_recv() {
            let (h1, h2, kind) = match event {
                rapier3d::geometry::CollisionEvent::Started(a, b, _) => {
                    (a, b, CollisionEventKind::Enter)
                }
                rapier3d::geometry::CollisionEvent::Stopped(a, b, _) => {
                    (a, b, CollisionEventKind::Exit)
                }
            };

            let body_a = self
                .collider_to_body
                .get(&h1)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied();
            let body_b = self
                .collider_to_body
                .get(&h2)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied();
            if let (Some(a), Some(b)) = (body_a, body_b) {
                self.collision_events.push(EngineCollisionEvent {
                    body_a: BodyHandle(a),
                    body_b: BodyHandle(b),
                    kind,
                });
            }
        }

        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 3]) {
        self.gravity = vector![gravity[0], gravity[1], gravity[2]];
    }

    fn gravity(&self) -> [f32; 3] {
        [self.gravity.x, self.gravity.y, self.gravity.z]
    }

    fn set_timestep(&mut self, dt: f32) {
        self.integration_params.dt = dt;
    }

    fn timestep(&self) -> f32 {
        self.integration_params.dt
    }

    fn create_body(&mut self, desc: &BodyDesc3D) -> GoudResult<BodyHandle> {
        let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
            desc.rotation[3],
            desc.rotation[0],
            desc.rotation[1],
            desc.rotation[2],
        ));
        let body_type = body_type_from_u32(desc.body_type);
        let builder = match body_type {
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            _ => RigidBodyBuilder::kinematic_position_based(),
        };
        let body = builder
            .translation(vector![
                desc.position[0],
                desc.position[1],
                desc.position[2]
            ])
            .rotation(rotation.scaled_axis())
            .linear_damping(desc.linear_damping)
            .angular_damping(desc.angular_damping)
            .gravity_scale(desc.gravity_scale)
            .ccd_enabled(desc.ccd_enabled)
            .locked_axes(if desc.fixed_rotation {
                LockedAxes::ROTATION_LOCKED
            } else {
                LockedAxes::empty()
            })
            .build();

        let rb_handle = self.rigid_body_set.insert(body);
        let id = self.next_body_id;
        self.next_body_id += 1;
        self.body_map.insert(id, rb_handle);
        self.body_reverse.insert(rb_handle, id);
        Ok(BodyHandle(id))
    }

    fn destroy_body(&mut self, handle: BodyHandle) {
        if let Some(rb_handle) = self.body_map.remove(&handle.0) {
            self.body_reverse.remove(&rb_handle);
            self.rigid_body_set.remove(
                rb_handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }

    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let t = body.translation();
        Ok([t.x, t.y, t.z])
    }

    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_translation(vector![pos[0], pos[1], pos[2]], true);
        Ok(())
    }

    fn body_rotation(&self, handle: BodyHandle) -> GoudResult<[f32; 4]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let q = body.rotation();
        Ok([q.i, q.j, q.k, q.w])
    }

    fn set_body_rotation(&mut self, handle: BodyHandle, rot: [f32; 4]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let rotation =
            UnitQuaternion::from_quaternion(Quaternion::new(rot[3], rot[0], rot[1], rot[2]));
        body.set_rotation(rotation, true);
        Ok(())
    }

    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let v = body.linvel();
        Ok([v.x, v.y, v.z])
    }

    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_linvel(vector![vel[0], vel[1], vel[2]], true);
        Ok(())
    }

    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.add_force(vector![force[0], force[1], force[2]], true);
        Ok(())
    }

    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.apply_impulse(vector![impulse[0], impulse[1], impulse[2]], true);
        Ok(())
    }

    fn body_gravity_scale(&self, handle: BodyHandle) -> GoudResult<f32> {
        self.query_body_gravity_scale(handle)
    }

    fn set_body_gravity_scale(&mut self, handle: BodyHandle, scale: f32) -> GoudResult<()> {
        self.query_set_body_gravity_scale(handle, scale)
    }

    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc3D,
    ) -> GoudResult<EngineColliderHandle> {
        let rb = self.resolve_body(body)?;
        let shape = shape_from_desc(desc);
        let collider = ColliderBuilder::new(shape)
            .friction(desc.friction)
            .restitution(desc.restitution)
            .sensor(desc.is_sensor)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let rapier_handle =
            self.collider_set
                .insert_with_parent(collider, rb, &mut self.rigid_body_set);

        let id = self.next_collider_id;
        self.next_collider_id += 1;
        self.collider_map.insert(id, rapier_handle);
        self.collider_to_body.insert(rapier_handle, rb);
        Ok(EngineColliderHandle(id))
    }

    fn destroy_collider(&mut self, handle: EngineColliderHandle) {
        if let Some(rapier_handle) = self.collider_map.remove(&handle.0) {
            self.collider_to_body.remove(&rapier_handle);
            self.collider_set.remove(
                rapier_handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
        }
    }

    fn collider_friction(&self, handle: EngineColliderHandle) -> GoudResult<f32> {
        self.query_collider_friction(handle)
    }

    fn set_collider_friction(
        &mut self,
        handle: EngineColliderHandle,
        friction: f32,
    ) -> GoudResult<()> {
        self.query_set_collider_friction(handle, friction)
    }

    fn collider_restitution(&self, handle: EngineColliderHandle) -> GoudResult<f32> {
        self.query_collider_restitution(handle)
    }

    fn set_collider_restitution(
        &mut self,
        handle: EngineColliderHandle,
        restitution: f32,
    ) -> GoudResult<()> {
        self.query_set_collider_restitution(handle, restitution)
    }

    fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<RaycastHit3D> {
        self.query_raycast(origin, dir, max_dist)
    }

    fn overlap_sphere(&self, center: [f32; 3], radius: f32) -> Vec<BodyHandle> {
        self.query_overlap_sphere(center, radius)
    }

    fn drain_collision_events(&mut self) -> Vec<EngineCollisionEvent> {
        std::mem::take(&mut self.collision_events)
    }

    fn contact_pairs(&self) -> Vec<ContactPair3D> {
        self.query_contact_pairs()
    }

    fn create_joint(&mut self, desc: &JointDesc3D) -> GoudResult<JointHandle> {
        let body_a_handle = desc
            .body_a
            .ok_or(GoudError::InvalidHandle)
            .and_then(|h| self.resolve_body(h))?;
        let body_b_handle = desc
            .body_b
            .ok_or(GoudError::InvalidHandle)
            .and_then(|h| self.resolve_body(h))?;

        let joint = joint_from_desc(desc, body_a_handle, body_b_handle)?;
        let rapier_handle =
            self.impulse_joint_set
                .insert(body_a_handle, body_b_handle, joint, true);

        let id = self.next_joint_id;
        self.next_joint_id += 1;
        self.joint_map.insert(id, rapier_handle);
        Ok(JointHandle(id))
    }

    fn destroy_joint(&mut self, handle: JointHandle) {
        if let Some(rapier_handle) = self.joint_map.remove(&handle.0) {
            self.impulse_joint_set.remove(rapier_handle, true);
        }
    }

    fn debug_shapes(&self) -> Vec<DebugShape3D> {
        self.query_debug_shapes()
    }

    fn physics3d_diagnostics(&self) -> Physics3DDiagnosticsV1 {
        Physics3DDiagnosticsV1 {
            body_count: self.body_map.len() as u32,
            collider_count: self.collider_map.len() as u32,
            joint_count: self.joint_map.len() as u32,
            contact_pair_count: 0,
            gravity: [self.gravity.x, self.gravity.y, self.gravity.z],
            timestep: self.integration_params.dt,
        }
    }

    fn create_character_controller(
        &mut self,
        desc: &CharacterControllerDesc3D,
    ) -> GoudResult<CharacterControllerHandle> {
        // Create a kinematic rigid body at the requested position.
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![
                desc.position[0],
                desc.position[1],
                desc.position[2]
            ])
            .locked_axes(LockedAxes::ROTATION_LOCKED)
            .build();
        let body_handle = self.rigid_body_set.insert(body);

        // Create a capsule collider attached to the body.
        let shape = SharedShape::capsule_y(desc.half_height, desc.radius);
        let collider = ColliderBuilder::new(shape.clone())
            .friction(0.0)
            .restitution(0.0)
            .build();
        let collider_handle =
            self.collider_set
                .insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        // Configure the Rapier kinematic character controller.
        let controller = KinematicCharacterController {
            max_slope_climb_angle: desc.max_slope_angle,
            min_slope_slide_angle: desc.max_slope_angle,
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(desc.step_height),
                min_width: CharacterLength::Relative(0.5),
                include_dynamic_bodies: true,
            }),
            snap_to_ground: Some(CharacterLength::Absolute(0.1)),
            ..KinematicCharacterController::default()
        };

        let id = self.next_controller_id;
        self.next_controller_id += 1;

        self.controllers.insert(
            id,
            CharacterControllerData {
                controller,
                body_handle,
                collider_handle,
                shape,
                grounded: false,
                vertical_velocity: 0.0,
                gravity: self.gravity.y.abs(),
            },
        );

        Ok(CharacterControllerHandle(id))
    }

    fn move_character(
        &mut self,
        handle: CharacterControllerHandle,
        displacement: [f32; 3],
        dt: f32,
    ) -> GoudResult<CharacterMoveResult3D> {
        let data = self
            .controllers
            .get_mut(&handle.0)
            .ok_or(GoudError::InvalidHandle)?;

        // Apply gravity when not grounded.
        if !data.grounded {
            data.vertical_velocity -= data.gravity * dt;
        } else {
            // Reset vertical velocity when grounded (allow small downward
            // snap so ground detection stays stable).
            data.vertical_velocity = -0.1;
        }

        let desired = vector![
            displacement[0],
            displacement[1] + data.vertical_velocity * dt,
            displacement[2]
        ];

        // Get current body position for the character.
        let body = self
            .rigid_body_set
            .get(data.body_handle)
            .ok_or(GoudError::InvalidHandle)?;
        let char_pos = *body.position();

        // Exclude the character's own collider from queries.
        let exclude_collider = data.collider_handle;
        let filter = QueryFilter::default().exclude_collider(exclude_collider);

        let result = data.controller.move_shape(
            dt,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            data.shape.as_ref(),
            &char_pos,
            desired,
            filter,
            |_| {},
        );

        data.grounded = result.grounded;
        if result.grounded {
            data.vertical_velocity = -0.1;
        }

        // Apply the corrected translation to the kinematic body.
        let new_translation = char_pos.translation.vector + result.translation;
        let body = self
            .rigid_body_set
            .get_mut(data.body_handle)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_next_kinematic_translation(new_translation);

        let pos = new_translation;
        Ok(CharacterMoveResult3D {
            position: [pos.x, pos.y, pos.z],
            grounded: data.grounded,
        })
    }

    fn character_position(&self, handle: CharacterControllerHandle) -> GoudResult<[f32; 3]> {
        let data = self
            .controllers
            .get(&handle.0)
            .ok_or(GoudError::InvalidHandle)?;
        let body = self
            .rigid_body_set
            .get(data.body_handle)
            .ok_or(GoudError::InvalidHandle)?;
        let t = body.translation();
        Ok([t.x, t.y, t.z])
    }

    fn is_character_grounded(&self, handle: CharacterControllerHandle) -> GoudResult<bool> {
        let data = self
            .controllers
            .get(&handle.0)
            .ok_or(GoudError::InvalidHandle)?;
        Ok(data.grounded)
    }

    fn destroy_character_controller(&mut self, handle: CharacterControllerHandle) {
        if let Some(data) = self.controllers.remove(&handle.0) {
            self.collider_set.remove(
                data.collider_handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
            self.rigid_body_set.remove(
                data.body_handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }
}
