//! 3D physics FFI exports.
//!
//! Provides C-compatible functions for Rapier3D physics: body creation,
//! collider attachment, forces, impulses, and simulation stepping.

pub(crate) mod access;
pub(crate) mod bodies;
pub(crate) mod character_controller;
pub(crate) mod lifecycle;
pub(crate) mod simulation;

pub use bodies::{
    goud_physics3d_add_collider, goud_physics3d_add_rigid_body, goud_physics3d_add_rigid_body_ex,
    goud_physics3d_create_joint, goud_physics3d_remove_body, goud_physics3d_remove_joint,
};
pub use character_controller::{
    goud_physics3d_create_character_controller, goud_physics3d_destroy_character_controller,
    goud_physics3d_get_character_position, goud_physics3d_is_character_grounded,
    goud_physics3d_move_character,
};
pub use lifecycle::{goud_physics3d_create, goud_physics3d_destroy, goud_physics3d_set_gravity};
pub use simulation::{
    goud_physics3d_apply_force, goud_physics3d_apply_impulse, goud_physics3d_get_position,
    goud_physics3d_set_velocity, goud_physics3d_step,
};

pub(crate) use access::{
    debug_shapes_for_context, with_provider, with_provider_mut, INVALID_HANDLE,
};
