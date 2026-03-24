//! Character controller implementation for Rapier3D physics provider.

use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

use super::Rapier3DPhysicsProvider;
use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::types::{
    CharacterControllerDesc3D, CharacterControllerHandle, CharacterMoveResult3D,
};

/// Internal data for a character controller instance.
pub(super) struct CharacterControllerData {
    /// The Rapier kinematic character controller.
    pub controller: KinematicCharacterController,
    /// Handle to the kinematic rigid body.
    pub body_handle: RigidBodyHandle,
    /// Handle to the capsule collider.
    pub collider_handle: ColliderHandle,
    /// The capsule shape used for move_shape queries.
    pub shape: SharedShape,
    /// Whether the character is currently touching the ground.
    pub grounded: bool,
    /// Accumulated vertical velocity for gravity.
    pub vertical_velocity: f32,
    /// Gravity magnitude (m/s^2, positive = downward).
    pub gravity: f32,
}

impl Rapier3DPhysicsProvider {
    pub(super) fn create_character_controller_impl(
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

    pub(super) fn move_character_impl(
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

    pub(super) fn character_position_impl(
        &self,
        handle: CharacterControllerHandle,
    ) -> GoudResult<[f32; 3]> {
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

    pub(super) fn is_character_grounded_impl(
        &self,
        handle: CharacterControllerHandle,
    ) -> GoudResult<bool> {
        let data = self
            .controllers
            .get(&handle.0)
            .ok_or(GoudError::InvalidHandle)?;
        Ok(data.grounded)
    }

    pub(super) fn destroy_character_controller_impl(&mut self, handle: CharacterControllerHandle) {
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
