//! Physics interpolation resource.
//!
//! Stores the interpolation alpha computed by the physics step systems
//! so that rendering systems can smoothly interpolate visual positions
//! between discrete physics steps.

/// Resource holding the physics interpolation factor.
///
/// After the physics step loop finishes, the remaining time in the
/// accumulator is expressed as `alpha = accumulator / timestep`, a value
/// in `[0.0, 1.0)`. Rendering systems use this to lerp between the
/// previous and current physics state for jitter-free visuals.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::physics_world::interpolation::PhysicsInterpolation;
///
/// let interp = PhysicsInterpolation { alpha: 0.5 };
/// assert_eq!(interp.alpha, 0.5);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PhysicsInterpolation {
    /// Interpolation factor in `[0.0, 1.0)`.
    pub alpha: f32,
}
