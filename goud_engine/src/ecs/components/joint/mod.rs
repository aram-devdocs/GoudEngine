//! Joint component for ECS-authored physics constraints.
//!
//! The ECS physics surface is currently 2D-oriented (`RigidBody`, `Collider`,
//! `Transform2D`), so this component stores 2D anchors/axes and references the
//! connected entity rather than provider handles.

mod component;

#[cfg(test)]
mod tests;

pub use crate::core::providers::types::{JointKind, JointLimits, JointMotor};
pub use component::Joint;
