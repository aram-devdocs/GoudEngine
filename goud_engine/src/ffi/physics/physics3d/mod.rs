//! 3D physics FFI exports.
//!
//! Provides C-compatible functions for Rapier3D physics: body creation,
//! collider attachment, forces, impulses, and simulation stepping.

mod access;
mod bodies;
mod lifecycle;
mod simulation;

pub use bodies::{
    goud_physics3d_add_collider, goud_physics3d_add_rigid_body, goud_physics3d_add_rigid_body_ex,
    goud_physics3d_create_joint, goud_physics3d_remove_body, goud_physics3d_remove_joint,
};
pub use lifecycle::{goud_physics3d_create, goud_physics3d_destroy, goud_physics3d_set_gravity};
pub use simulation::{
    goud_physics3d_apply_force, goud_physics3d_apply_impulse, goud_physics3d_get_position,
    goud_physics3d_set_velocity, goud_physics3d_step,
};

pub(crate) use access::{with_provider, with_provider_mut, INVALID_HANDLE};
