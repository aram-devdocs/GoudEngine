use crate::core::error::set_last_error;
use crate::core::providers::types::{BodyHandle, JointHandle, JointKind, JointLimits, JointMotor};
use crate::core::providers::types3d::{BodyDesc3D, ColliderDesc3D, JointDesc3D};
use crate::ffi::context::GoudContextId;

use super::{with_provider_mut, INVALID_HANDLE};

/// Creates a rigid body in the 3D physics world.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y`, `z` - Initial position
/// * `gravity_scale` - Per-body gravity multiplier (1.0 = normal)
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_add_rigid_body(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
    z: f32,
    gravity_scale: f32,
) -> i64 {
    goud_physics3d_add_rigid_body_ex(ctx, body_type, x, y, z, gravity_scale, false)
}

/// Creates a rigid body in the 3D physics world with explicit CCD control.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_type` - 0 = static, 1 = dynamic, 2 = kinematic
/// * `x`, `y`, `z` - Initial position
/// * `gravity_scale` - Per-body gravity multiplier (1.0 = normal)
/// * `ccd_enabled` - Enables continuous collision detection for this body
///
/// # Returns
///
/// A positive body handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_add_rigid_body_ex(
    ctx: GoudContextId,
    body_type: u32,
    x: f32,
    y: f32,
    z: f32,
    gravity_scale: f32,
    ccd_enabled: bool,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = BodyDesc3D {
            position: [x, y, z],
            body_type,
            gravity_scale,
            ccd_enabled,
            ..BodyDesc3D::default()
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

/// Removes a rigid body from the 3D physics world.
///
/// # Returns
///
/// 0 on success, negative error code on failure.
#[no_mangle]
pub extern "C" fn goud_physics3d_remove_body(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_body(BodyHandle(handle));
        0
    })
}

/// Creates a joint between two rigid bodies in the 3D physics world.
///
/// `joint_kind` mapping:
/// - `0`: revolute
/// - `1`: prismatic
/// - `2+`: distance
#[no_mangle]
pub extern "C" fn goud_physics3d_create_joint(
    ctx: GoudContextId,
    body_a: u64,
    body_b: u64,
    joint_kind: u32,
    anchor_ax: f32,
    anchor_ay: f32,
    anchor_az: f32,
    anchor_bx: f32,
    anchor_by: f32,
    anchor_bz: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    use_limits: bool,
    limit_min: f32,
    limit_max: f32,
    use_motor: bool,
    motor_target_velocity: f32,
    motor_max_force: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = JointDesc3D {
            body_a: Some(BodyHandle(body_a)),
            body_b: Some(BodyHandle(body_b)),
            kind: joint_kind_from_u32(joint_kind),
            anchor_a: [anchor_ax, anchor_ay, anchor_az],
            anchor_b: [anchor_bx, anchor_by, anchor_bz],
            axis: [axis_x, axis_y, axis_z],
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

/// Removes a joint from the 3D physics world.
#[no_mangle]
pub extern "C" fn goud_physics3d_remove_joint(ctx: GoudContextId, handle: u64) -> i32 {
    with_provider_mut(ctx, |p| {
        p.destroy_joint(JointHandle(handle));
        0
    })
}

/// Attaches a collider to a rigid body in 3D.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_handle` - Handle of the body to attach to
/// * `shape_type` - 0 = sphere, 1 = box, 2 = capsule
/// * `hx`, `hy`, `hz` - Half-extents for box shapes
/// * `radius` - Radius for sphere/capsule shapes
/// * `friction` - Friction coefficient (e.g. 0.5)
/// * `restitution` - Bounciness coefficient (e.g. 0.0)
///
/// # Returns
///
/// A positive collider handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics3d_add_collider(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    hx: f32,
    hy: f32,
    hz: f32,
    radius: f32,
    friction: f32,
    restitution: f32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = ColliderDesc3D {
            shape: shape_type,
            half_extents: [hx, hy, hz],
            radius,
            friction,
            restitution,
            is_sensor: false,
            ..ColliderDesc3D::default()
        };
        match p.create_collider(BodyHandle(body_handle), &desc) {
            Ok(handle) => handle.0 as i64,
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

fn joint_kind_from_u32(raw: u32) -> JointKind {
    match raw {
        0 => JointKind::Revolute,
        1 => JointKind::Prismatic,
        _ => JointKind::Distance,
    }
}
