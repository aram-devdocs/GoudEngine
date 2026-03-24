//! Body and collider management for Rapier3D physics provider.

use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;

use super::conversions::{body_type_from_u32, shape_from_desc};
use super::Rapier3DPhysicsProvider;
use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::types::ColliderHandle as EngineColliderHandle;
use crate::core::providers::types::{BodyDesc3D, BodyHandle, ColliderDesc3D};

impl Rapier3DPhysicsProvider {
    pub(super) fn create_body_impl(&mut self, desc: &BodyDesc3D) -> GoudResult<BodyHandle> {
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

    pub(super) fn destroy_body_impl(&mut self, handle: BodyHandle) {
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

    pub(super) fn body_position_impl(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let t = body.translation();
        Ok([t.x, t.y, t.z])
    }

    pub(super) fn set_body_position_impl(
        &mut self,
        handle: BodyHandle,
        pos: [f32; 3],
    ) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_translation(vector![pos[0], pos[1], pos[2]], true);
        Ok(())
    }

    pub(super) fn body_rotation_impl(&self, handle: BodyHandle) -> GoudResult<[f32; 4]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let q = body.rotation();
        Ok([q.i, q.j, q.k, q.w])
    }

    pub(super) fn set_body_rotation_impl(
        &mut self,
        handle: BodyHandle,
        rot: [f32; 4],
    ) -> GoudResult<()> {
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

    pub(super) fn body_velocity_impl(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let v = body.linvel();
        Ok([v.x, v.y, v.z])
    }

    pub(super) fn set_body_velocity_impl(
        &mut self,
        handle: BodyHandle,
        vel: [f32; 3],
    ) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_linvel(vector![vel[0], vel[1], vel[2]], true);
        Ok(())
    }

    pub(super) fn apply_force_impl(
        &mut self,
        handle: BodyHandle,
        force: [f32; 3],
    ) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.add_force(vector![force[0], force[1], force[2]], true);
        Ok(())
    }

    pub(super) fn apply_impulse_impl(
        &mut self,
        handle: BodyHandle,
        impulse: [f32; 3],
    ) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.apply_impulse(vector![impulse[0], impulse[1], impulse[2]], true);
        Ok(())
    }

    pub(super) fn create_collider_impl(
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

    pub(super) fn destroy_collider_impl(&mut self, handle: EngineColliderHandle) {
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
}
