//! 2D physics FFI exports.
//!
//! Provides C-compatible functions for Rapier2D physics: body creation,
//! collider attachment, forces, impulses, simulation stepping, and raycasting.

pub mod bodies;
pub mod lifecycle;
pub mod simulation;

pub use bodies::{
    goud_physics_add_collider, goud_physics_add_rigid_body, goud_physics_add_rigid_body_ex,
    goud_physics_create_joint, goud_physics_remove_body, goud_physics_remove_joint,
};
pub use lifecycle::{
    goud_physics_create, goud_physics_create_with_backend, goud_physics_destroy,
    goud_physics_set_gravity,
};
pub use simulation::{
    goud_physics_apply_force, goud_physics_apply_impulse, goud_physics_get_position,
    goud_physics_get_velocity, goud_physics_raycast, goud_physics_set_velocity, goud_physics_step,
};
