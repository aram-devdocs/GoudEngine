//! Collision response configuration and resolution functions.

use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;

/// Collision response configuration for impulse resolution.
///
/// These settings control how collisions are resolved using impulse-based physics.
/// Different collision types (bounce, slide, etc.) require different configurations.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::collision::CollisionResponse;
///
/// // Bouncy ball (high restitution)
/// let bouncy = CollisionResponse::bouncy();
///
/// // Character controller (no bounce, high friction)
/// let character = CollisionResponse::character();
///
/// // Realistic physics (default)
/// let realistic = CollisionResponse::default();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResponse {
    /// How much velocity is retained after collision (0.0 = no bounce, 1.0 = perfect bounce).
    ///
    /// Typical values:
    /// - 0.0-0.1: Non-bouncy (characters, boxes)
    /// - 0.3-0.5: Moderately bouncy (rubber balls)
    /// - 0.8-1.0: Very bouncy (super balls)
    pub restitution: f32,

    /// Resistance to sliding (0.0 = ice, 1.0 = rubber on concrete).
    ///
    /// Typical values:
    /// - 0.0-0.1: Ice, very slippery
    /// - 0.2-0.4: Wood on wood, metal on metal
    /// - 0.6-0.8: Rubber on concrete
    pub friction: f32,

    /// Percentage of overlap to resolve (0.0-1.0).
    ///
    /// Helps prevent objects from sinking into each other due to numerical errors.
    /// - 0.0: No positional correction
    /// - 0.2-0.8: Recommended range (0.4 is common)
    /// - 1.0: Full correction (may cause jitter)
    pub position_correction: f32,

    /// Threshold below which positional correction is not applied.
    ///
    /// Prevents over-correction for small penetrations.
    /// Typical value: 0.01 units
    pub slop: f32,
}

impl CollisionResponse {
    /// Creates a new collision response configuration.
    ///
    /// # Arguments
    ///
    /// * `restitution` - Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    /// * `friction` - Sliding resistance (0.0 = ice, 1.0 = rubber)
    /// * `position_correction` - Overlap correction percentage (0.0-1.0)
    /// * `slop` - Minimum penetration for correction
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::CollisionResponse;
    ///
    /// let response = CollisionResponse::new(0.5, 0.4, 0.4, 0.01);
    /// ```
    pub fn new(restitution: f32, friction: f32, position_correction: f32, slop: f32) -> Self {
        Self {
            restitution: restitution.clamp(0.0, 1.0),
            friction: friction.clamp(0.0, 1.0),
            position_correction: position_correction.clamp(0.0, 1.0),
            slop: slop.max(0.0),
        }
    }

    /// Creates a bouncy collision response (high restitution, low friction).
    ///
    /// Good for: balls, projectiles, bouncing objects
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::CollisionResponse;
    ///
    /// let bouncy = CollisionResponse::bouncy();
    /// assert_eq!(bouncy.restitution, 0.8);
    /// ```
    pub fn bouncy() -> Self {
        Self::new(0.8, 0.2, 0.4, 0.01)
    }

    /// Creates a character controller response (no bounce, high friction).
    ///
    /// Good for: player characters, NPCs, walking entities
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::CollisionResponse;
    ///
    /// let character = CollisionResponse::character();
    /// assert_eq!(character.restitution, 0.0);
    /// assert_eq!(character.friction, 0.8);
    /// ```
    pub fn character() -> Self {
        Self::new(0.0, 0.8, 0.6, 0.01)
    }

    /// Creates a slippery collision response (low friction, some bounce).
    ///
    /// Good for: ice, smooth surfaces, low-friction materials
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::CollisionResponse;
    ///
    /// let slippery = CollisionResponse::slippery();
    /// assert!(slippery.friction < 0.2);
    /// ```
    pub fn slippery() -> Self {
        Self::new(0.3, 0.1, 0.4, 0.01)
    }

    /// Creates a perfectly elastic collision (no energy loss).
    ///
    /// Good for: billiard balls, perfect bounce scenarios
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::CollisionResponse;
    ///
    /// let elastic = CollisionResponse::elastic();
    /// assert_eq!(elastic.restitution, 1.0);
    /// ```
    pub fn elastic() -> Self {
        Self::new(1.0, 0.4, 0.4, 0.01)
    }
}

impl Default for CollisionResponse {
    /// Default collision response: moderate bounce and friction.
    fn default() -> Self {
        Self::new(0.4, 0.4, 0.4, 0.01)
    }
}

/// Resolves a collision between two bodies using impulse-based physics.
///
/// This function computes and applies the impulse needed to resolve a collision
/// between two dynamic bodies. It handles both normal impulse (bounce/separation)
/// and tangent impulse (friction).
///
/// # Algorithm
///
/// 1. Compute relative velocity at contact point
/// 2. Apply restitution (bounce) along normal
/// 3. Apply friction along tangent
/// 4. Apply position correction to prevent sinking
///
/// # Arguments
///
/// * `contact` - Contact information from collision detection
/// * `velocity_a` - Linear velocity of body A
/// * `velocity_b` - Linear velocity of body B
/// * `inv_mass_a` - Inverse mass of body A (0.0 for static)
/// * `inv_mass_b` - Inverse mass of body B (0.0 for static)
/// * `response` - Collision response configuration
///
/// # Returns
///
/// Tuple of velocity changes: `(delta_velocity_a, delta_velocity_b)`
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::collision::{Contact, CollisionResponse, resolve_collision};
/// use goud_engine::core::math::Vec2;
///
/// let contact = Contact::new(
///     Vec2::new(1.0, 0.0),  // contact point
///     Vec2::new(1.0, 0.0),  // normal (A to B)
///     0.1,                   // penetration
/// );
///
/// let vel_a = Vec2::new(10.0, 0.0);  // A moving right
/// let vel_b = Vec2::new(-5.0, 0.0);  // B moving left
/// let inv_mass_a = 1.0;               // 1 kg
/// let inv_mass_b = 1.0;               // 1 kg
/// let response = CollisionResponse::default();
///
/// let (delta_vel_a, delta_vel_b) = resolve_collision(
///     &contact,
///     vel_a,
///     vel_b,
///     inv_mass_a,
///     inv_mass_b,
///     &response,
/// );
///
/// // Apply velocity changes
/// let new_vel_a = vel_a + delta_vel_a;
/// let new_vel_b = vel_b + delta_vel_b;
/// ```
///
/// # Physics Notes
///
/// - Static bodies (inv_mass = 0.0) do not move
/// - Higher mass (lower inv_mass) means less velocity change
/// - Restitution of 1.0 means perfect elastic collision
/// - Friction is applied perpendicular to collision normal
pub fn resolve_collision(
    contact: &Contact,
    velocity_a: Vec2,
    velocity_b: Vec2,
    inv_mass_a: f32,
    inv_mass_b: f32,
    response: &CollisionResponse,
) -> (Vec2, Vec2) {
    // Early exit if both bodies are static
    let total_inv_mass = inv_mass_a + inv_mass_b;
    if total_inv_mass < 1e-6 {
        return (Vec2::zero(), Vec2::zero());
    }

    // Compute relative velocity
    let relative_velocity = velocity_b - velocity_a;
    let velocity_along_normal = relative_velocity.dot(contact.normal);

    // Don't resolve if velocities are separating
    if velocity_along_normal > 0.0 {
        return (Vec2::zero(), Vec2::zero());
    }

    // =================================================================
    // Normal Impulse (bounce/separation)
    // =================================================================

    // Compute impulse scalar with restitution
    let restitution = response.restitution;
    let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / total_inv_mass;

    // Apply impulse along normal
    let impulse = contact.normal * impulse_scalar;
    let delta_vel_a_normal = impulse * -inv_mass_a;
    let delta_vel_b_normal = impulse * inv_mass_b;

    // =================================================================
    // Tangent Impulse (friction)
    // =================================================================

    // Compute tangent (perpendicular to normal)
    let tangent = Vec2::new(-contact.normal.y, contact.normal.x);

    // Relative velocity along tangent
    let relative_velocity_tangent =
        (relative_velocity + delta_vel_b_normal - delta_vel_a_normal).dot(tangent);

    // Don't apply friction if already stationary in tangent direction
    if relative_velocity_tangent.abs() < 1e-6 {
        return (delta_vel_a_normal, delta_vel_b_normal);
    }

    // Compute friction impulse (Coulomb friction model)
    let friction_impulse_scalar = -relative_velocity_tangent / total_inv_mass;
    let mu = response.friction;

    // Clamp friction to not exceed normal impulse (Coulomb's law)
    let friction_impulse_scalar =
        friction_impulse_scalar.clamp(-impulse_scalar * mu, impulse_scalar * mu);

    let friction_impulse = tangent * friction_impulse_scalar;
    let delta_vel_a_friction = friction_impulse * -inv_mass_a;
    let delta_vel_b_friction = friction_impulse * inv_mass_b;

    // =================================================================
    // Combine normal and friction impulses
    // =================================================================

    let delta_vel_a = delta_vel_a_normal + delta_vel_a_friction;
    let delta_vel_b = delta_vel_b_normal + delta_vel_b_friction;

    (delta_vel_a, delta_vel_b)
}

/// Computes positional correction to prevent sinking due to numerical drift.
///
/// This function uses Baumgarte stabilization to gradually push objects apart
/// when they penetrate each other. This prevents accumulation of small errors
/// that would cause objects to sink over time.
///
/// # Arguments
///
/// * `contact` - Contact information with penetration depth
/// * `inv_mass_a` - Inverse mass of body A (0.0 for static)
/// * `inv_mass_b` - Inverse mass of body B (0.0 for static)
/// * `response` - Collision response configuration (uses position_correction and slop)
///
/// # Returns
///
/// Tuple of position corrections: `(correction_a, correction_b)`
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::collision::{Contact, CollisionResponse, compute_position_correction};
/// use goud_engine::core::math::Vec2;
///
/// let contact = Contact::new(
///     Vec2::new(1.0, 0.0),
///     Vec2::new(1.0, 0.0),
///     0.1,  // 0.1 units penetration
/// );
///
/// let inv_mass_a = 1.0;
/// let inv_mass_b = 1.0;
/// let response = CollisionResponse::default();
///
/// let (correction_a, correction_b) = compute_position_correction(
///     &contact,
///     inv_mass_a,
///     inv_mass_b,
///     &response,
/// );
///
/// // Apply corrections to positions
/// // position_a += correction_a;
/// // position_b += correction_b;
/// ```
///
/// # Physics Notes
///
/// - Only corrects penetration above `slop` threshold
/// - Correction percentage controls trade-off between stability and jitter
/// - Static bodies (inv_mass = 0.0) do not move
/// - Heavier objects move less than lighter objects
pub fn compute_position_correction(
    contact: &Contact,
    inv_mass_a: f32,
    inv_mass_b: f32,
    response: &CollisionResponse,
) -> (Vec2, Vec2) {
    // Early exit if both bodies are static
    let total_inv_mass = inv_mass_a + inv_mass_b;
    if total_inv_mass < 1e-6 {
        return (Vec2::zero(), Vec2::zero());
    }

    // Only correct penetration above slop threshold
    let penetration = contact.penetration - response.slop;
    if penetration <= 0.0 {
        return (Vec2::zero(), Vec2::zero());
    }

    // Compute correction magnitude (Baumgarte stabilization)
    let correction_percent = response.position_correction;
    let correction_magnitude = penetration * correction_percent / total_inv_mass;

    // Apply correction along collision normal
    let correction = contact.normal * correction_magnitude;
    let correction_a = correction * -inv_mass_a;
    let correction_b = correction * inv_mass_b;

    (correction_a, correction_b)
}
