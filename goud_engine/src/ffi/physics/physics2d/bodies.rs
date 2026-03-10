use crate::core::error::set_last_error;
use crate::core::providers::types::{
    BodyDesc, BodyHandle, JointDesc, JointHandle, JointKind, JointLimits, JointMotor,
};
use crate::ffi::context::GoudContextId;

use super::super::physics2d_common::{with_provider_mut, INVALID_HANDLE};
use super::super::physics2d_ex::{
    add_collider_with_filter, DEFAULT_COLLISION_LAYER, DEFAULT_COLLISION_MASK,
};
use super::super::physics2d_state::remove_body as remove_body_state;

/// Creates a rigid body in the 2D physics world.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y` - Initial position
/// * `gravity_scale` - Per-body gravity multiplier (1.0 = normal)
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_rigid_body(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
    gravity_scale: f32,
) -> i64 {
    goud_physics_add_rigid_body_ex(ctx, body_type, x, y, gravity_scale, false)
}

/// Creates a rigid body in the 2D physics world with explicit CCD control.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y` - Initial position
/// * `gravity_scale` - Per-body gravity multiplier (1.0 = normal)
/// * `ccd_enabled` - Enables continuous collision detection for this body
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_rigid_body_ex(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
    gravity_scale: f32,
    ccd_enabled: bool,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = BodyDesc {
            position: [x, y],
            body_type,
            gravity_scale,
            ccd_enabled,
            ..BodyDesc::default()
        };
        match p.create_body(&desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

/// Removes a rigid body from the 2D physics world.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics_remove_body(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_body(BodyHandle(handle));
        remove_body_state(ctx, handle);
        0
    })
}

/// Creates a joint between two rigid bodies in the 2D physics world.
///
/// `joint_kind` mapping:
/// - `0`: revolute
/// - `1`: prismatic
/// - `2+`: distance
///
/// # Returns
///
/// A positive joint handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_create_joint(
    ctx: GoudContextId,
    body_a: u64,
    body_b: u64,
    joint_kind: u32,
    anchor_ax: f32,
    anchor_ay: f32,
    anchor_bx: f32,
    anchor_by: f32,
    axis_x: f32,
    axis_y: f32,
    use_limits: bool,
    limit_min: f32,
    limit_max: f32,
    use_motor: bool,
    motor_target_velocity: f32,
    motor_max_force: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = JointDesc {
            body_a: Some(BodyHandle(body_a)),
            body_b: Some(BodyHandle(body_b)),
            kind: joint_kind_from_u32(joint_kind),
            anchor_a: [anchor_ax, anchor_ay],
            anchor_b: [anchor_bx, anchor_by],
            axis: [axis_x, axis_y],
            limits: use_limits.then_some(JointLimits {
                min: limit_min,
                max: limit_max,
            }),
            motor: use_motor.then_some(JointMotor {
                target_velocity: motor_target_velocity,
                max_force: motor_max_force,
            }),
        };
        match p.create_joint(&desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

/// Removes a joint from the 2D physics world.
#[no_mangle]
pub extern "C" fn goud_physics_remove_joint(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_joint(JointHandle(handle));
        0
    })
}

/// Attaches a collider to a rigid body.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_handle` - Handle of the body to attach to
/// * `shape_type` - 0 = circle, 1 = box, 2 = capsule
/// * `width`, `height` - Half-extents for box shapes
/// * `radius` - Radius for circle/capsule shapes
/// * `friction` - Friction coefficient (e.g. 0.5)
/// * `restitution` - Bounciness coefficient (e.g. 0.0)
///
/// # Returns
///
/// A positive collider handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_collider(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    width: f32,
    height: f32,
    radius: f32,
    friction: f32,
    restitution: f32,
) -> i64 {
    add_collider_with_filter(
        ctx,
        body_handle,
        shape_type,
        width,
        height,
        radius,
        friction,
        restitution,
        false,
        DEFAULT_COLLISION_LAYER,
        DEFAULT_COLLISION_MASK,
    )
}

fn joint_kind_from_u32(raw: u32) -> JointKind {
    match raw {
        0 => JointKind::Revolute,
        1 => JointKind::Prismatic,
        _ => JointKind::Distance,
    }
}
