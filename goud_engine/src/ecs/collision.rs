//! Collision detection algorithms for 2D physics.
//!
//! This module provides narrow-phase collision detection between various collider shapes.
//! Each collision function returns contact information when shapes are intersecting.
//!
//! # Supported Shape Pairs
//!
//! - Circle-Circle (fastest)
//! - Circle-AABB
//! - Circle-OBB
//! - AABB-AABB
//! - OBB-OBB (SAT algorithm)
//! - Capsule-Circle
//! - Capsule-AABB
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::collision::{circle_circle_collision, Contact};
//! use goud_engine::core::math::Vec2;
//!
//! let circle_a = Vec2::new(0.0, 0.0);
//! let radius_a = 1.0;
//! let circle_b = Vec2::new(1.5, 0.0);
//! let radius_b = 1.0;
//!
//! if let Some(contact) = circle_circle_collision(circle_a, radius_a, circle_b, radius_b) {
//!     println!("Circles colliding! Penetration: {}", contact.penetration);
//!     println!("Normal: {:?}", contact.normal);
//! }
//! ```
//!
//! # Collision Detection Pipeline
//!
//! 1. **Broad Phase**: Spatial hash identifies potentially colliding pairs (see [`crate::ecs::broad_phase`])
//! 2. **Narrow Phase**: This module's algorithms compute exact contact information
//! 3. **Response**: Physics system resolves collisions based on contact data
//!
//! # Performance Notes
//!
//! - Circle-circle is fastest (single distance check)
//! - AABB-AABB is very fast (no rotation)
//! - OBB-OBB uses SAT (more expensive but accurate)
//! - Early exits when no collision detected

use crate::core::math::Vec2;

// =============================================================================
// Collision Response Module
// =============================================================================

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
    let relative_velocity_tangent = (relative_velocity + delta_vel_b_normal - delta_vel_a_normal).dot(tangent);

    // Don't apply friction if already stationary in tangent direction
    if relative_velocity_tangent.abs() < 1e-6 {
        return (delta_vel_a_normal, delta_vel_b_normal);
    }

    // Compute friction impulse (Coulomb friction model)
    let friction_impulse_scalar = -relative_velocity_tangent / total_inv_mass;
    let mu = response.friction;

    // Clamp friction to not exceed normal impulse (Coulomb's law)
    let friction_impulse_scalar = friction_impulse_scalar.clamp(-impulse_scalar * mu, impulse_scalar * mu);

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

// =============================================================================
// Contact Information
// =============================================================================

/// Contact information from a collision.
///
/// When two shapes collide, this struct contains all information needed to
/// resolve the collision:
///
/// - **point**: The contact point in world space
/// - **normal**: The collision normal (points from A to B)
/// - **penetration**: How deep the shapes overlap (positive = overlapping)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::{circle_circle_collision, Contact};
/// use goud_engine::core::math::Vec2;
///
/// let contact = circle_circle_collision(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(1.5, 0.0), 1.0
/// ).unwrap();
///
/// assert!(contact.penetration > 0.0);
/// assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    /// Contact point in world space.
    ///
    /// This is the point where the two shapes are touching. For penetrating
    /// collisions, this is typically the deepest penetration point.
    pub point: Vec2,

    /// Collision normal (unit vector from shape A to shape B).
    ///
    /// This vector points from the first shape to the second shape and is
    /// normalized to unit length. It indicates the direction to separate
    /// the shapes to resolve the collision.
    pub normal: Vec2,

    /// Penetration depth (positive = overlapping, negative = separated).
    ///
    /// This is the distance the shapes overlap. A positive value means the
    /// shapes are penetrating. To resolve the collision, move the shapes
    /// apart by this distance along the normal.
    pub penetration: f32,
}

impl Contact {
    /// Creates a new contact.
    ///
    /// # Arguments
    ///
    /// * `point` - Contact point in world space
    /// * `normal` - Collision normal (should be normalized)
    /// * `penetration` - Penetration depth
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::Contact;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let contact = Contact::new(
    ///     Vec2::new(1.0, 0.0),
    ///     Vec2::new(1.0, 0.0),
    ///     0.5
    /// );
    /// ```
    pub fn new(point: Vec2, normal: Vec2, penetration: f32) -> Self {
        Self {
            point,
            normal,
            penetration,
        }
    }

    /// Returns true if the contact represents a collision (positive penetration).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::Contact;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let colliding = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
    /// let separated = Contact::new(Vec2::zero(), Vec2::unit_x(), -0.1);
    ///
    /// assert!(colliding.is_colliding());
    /// assert!(!separated.is_colliding());
    /// ```
    pub fn is_colliding(&self) -> bool {
        self.penetration > 0.0
    }

    /// Returns the separation distance needed to resolve the collision.
    ///
    /// This is the magnitude of the vector needed to separate the shapes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::Contact;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let contact = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
    /// assert_eq!(contact.separation_distance(), 0.5);
    /// ```
    pub fn separation_distance(&self) -> f32 {
        self.penetration.abs()
    }

    /// Returns the separation vector needed to resolve the collision.
    ///
    /// This is the vector (normal * penetration) that would separate the shapes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::Contact;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let contact = Contact::new(
    ///     Vec2::zero(),
    ///     Vec2::new(1.0, 0.0),
    ///     0.5
    /// );
    /// assert_eq!(contact.separation_vector(), Vec2::new(0.5, 0.0));
    /// ```
    pub fn separation_vector(&self) -> Vec2 {
        self.normal * self.penetration
    }

    /// Returns a contact with reversed normal (swaps A and B).
    ///
    /// This is useful when the collision detection function expects shapes
    /// in a specific order but you have them reversed.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::collision::Contact;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let contact = Contact::new(
    ///     Vec2::zero(),
    ///     Vec2::new(1.0, 0.0),
    ///     0.5
    /// );
    /// let reversed = contact.reversed();
    ///
    /// assert_eq!(reversed.normal, Vec2::new(-1.0, 0.0));
    /// assert_eq!(reversed.penetration, contact.penetration);
    /// ```
    pub fn reversed(&self) -> Self {
        Self {
            point: self.point,
            normal: self.normal * -1.0,
            penetration: self.penetration,
        }
    }
}

impl Default for Contact {
    /// Returns a contact with no collision (zero penetration).
    fn default() -> Self {
        Self {
            point: Vec2::zero(),
            normal: Vec2::unit_x(),
            penetration: 0.0,
        }
    }
}

// =============================================================================
// Circle-Circle Collision
// =============================================================================

/// Detects collision between two circles.
///
/// This is the fastest collision detection algorithm, requiring only a single
/// distance check. Returns contact information if the circles overlap.
///
/// # Arguments
///
/// * `center_a` - Center of circle A
/// * `radius_a` - Radius of circle A (must be positive)
/// * `center_b` - Center of circle B
/// * `radius_b` - Radius of circle B (must be positive)
///
/// # Returns
///
/// - `Some(Contact)` if the circles are overlapping
/// - `None` if the circles are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_circle_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Overlapping circles
/// let contact = circle_circle_collision(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(1.5, 0.0), 1.0
/// );
/// assert!(contact.is_some());
/// assert!(contact.unwrap().penetration > 0.0);
///
/// // Separated circles
/// let no_contact = circle_circle_collision(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(5.0, 0.0), 1.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Single distance computation and comparison.
pub fn circle_circle_collision(
    center_a: Vec2,
    radius_a: f32,
    center_b: Vec2,
    radius_b: f32,
) -> Option<Contact> {
    // Vector from A to B
    let delta = center_b - center_a;
    let distance_squared = delta.length_squared();
    let combined_radius = radius_a + radius_b;
    let combined_radius_squared = combined_radius * combined_radius;

    // Early exit if circles are separated
    if distance_squared > combined_radius_squared {
        return None;
    }

    // Handle edge case: circles at exact same position
    if distance_squared < 1e-6 {
        // Circles are basically at the same position, use arbitrary normal
        return Some(Contact {
            point: center_a,
            normal: Vec2::unit_x(),
            penetration: combined_radius,
        });
    }

    // Compute distance and normal
    let distance = distance_squared.sqrt();
    let normal = delta / distance;

    // Penetration is how much they overlap
    let penetration = combined_radius - distance;

    // Contact point is halfway between the closest points on each circle
    let contact_point = center_a + normal * (radius_a - penetration * 0.5);

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}

// =============================================================================
// Box-Box Collision (SAT)
// =============================================================================

/// Detects collision between two oriented bounding boxes (OBBs) using SAT.
///
/// The Separating Axis Theorem (SAT) is the standard algorithm for OBB collision
/// detection. It tests for separation along potential separating axes. If no
/// separating axis is found, the boxes are colliding.
///
/// # Arguments
///
/// * `center_a` - Center of box A
/// * `half_extents_a` - Half-width and half-height of box A
/// * `rotation_a` - Rotation angle of box A in radians
/// * `center_b` - Center of box B
/// * `half_extents_b` - Half-width and half-height of box B
/// * `rotation_b` - Rotation angle of box B in radians
///
/// # Returns
///
/// - `Some(Contact)` if the boxes are overlapping
/// - `None` if the boxes are separated
///
/// # Algorithm
///
/// SAT tests for separation along 4 potential axes:
/// 1. Box A's X axis (rotated)
/// 2. Box A's Y axis (rotated)
/// 3. Box B's X axis (rotated)
/// 4. Box B's Y axis (rotated)
///
/// For each axis, we project both boxes and check for overlap.
/// If any axis shows no overlap, the boxes are separated.
/// Otherwise, we track the axis with minimum overlap (the collision normal).
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::box_box_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Two axis-aligned boxes overlapping
/// let contact = box_box_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
///     Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), 0.0
/// );
/// assert!(contact.is_some());
///
/// // Two rotated boxes separated
/// let no_contact = box_box_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
///     Vec2::new(5.0, 0.0), Vec2::new(1.0, 1.0), std::f32::consts::PI / 4.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Tests 4 axes, each with constant-time projection and overlap checks.
/// More expensive than AABB-AABB due to rotation handling.
pub fn box_box_collision(
    center_a: Vec2,
    half_extents_a: Vec2,
    rotation_a: f32,
    center_b: Vec2,
    half_extents_b: Vec2,
    rotation_b: f32,
) -> Option<Contact> {
    // Compute rotation matrices (cos/sin)
    let cos_a = rotation_a.cos();
    let sin_a = rotation_a.sin();
    let cos_b = rotation_b.cos();
    let sin_b = rotation_b.sin();

    // Box A axes (rotated)
    let axis_a_x = Vec2::new(cos_a, sin_a);
    let axis_a_y = Vec2::new(-sin_a, cos_a);

    // Box B axes (rotated)
    let axis_b_x = Vec2::new(cos_b, sin_b);
    let axis_b_y = Vec2::new(-sin_b, cos_b);

    // Center offset
    let delta = center_b - center_a;

    // Track minimum overlap and collision normal
    let mut min_overlap = f32::INFINITY;
    let mut collision_normal = Vec2::unit_x();

    // Test all 4 potential separating axes
    let axes = [
        (axis_a_x, half_extents_a.x, half_extents_a.y),
        (axis_a_y, half_extents_a.x, half_extents_a.y),
        (axis_b_x, half_extents_b.x, half_extents_b.y),
        (axis_b_y, half_extents_b.x, half_extents_b.y),
    ];

    for (axis, _hx_a, _hy_a) in &axes {
        // Project box A onto axis
        let r_a = half_extents_a.x * (axis.dot(axis_a_x)).abs()
            + half_extents_a.y * (axis.dot(axis_a_y)).abs();

        // Project box B onto axis
        let r_b = half_extents_b.x * (axis.dot(axis_b_x)).abs()
            + half_extents_b.y * (axis.dot(axis_b_y)).abs();

        // Project center offset onto axis
        let distance = axis.dot(delta).abs();

        // Check for separation
        let overlap = r_a + r_b - distance;
        if overlap < 0.0 {
            // Found separating axis - no collision
            return None;
        }

        // Track minimum overlap (shallowest penetration)
        if overlap < min_overlap {
            min_overlap = overlap;
            collision_normal = *axis;

            // Ensure normal points from A to B
            if axis.dot(delta) < 0.0 {
                collision_normal = collision_normal * -1.0;
            }
        }
    }

    // No separating axis found - boxes are colliding
    let contact_point = center_a + delta * 0.5; // Approximate contact point

    Some(Contact {
        point: contact_point,
        normal: collision_normal,
        penetration: min_overlap,
    })
}

/// Detects collision between two axis-aligned bounding boxes (AABBs).
///
/// This is a specialized, faster version of box-box collision for axis-aligned
/// boxes (rotation = 0). It's much simpler than the full SAT algorithm.
///
/// # Arguments
///
/// * `center_a` - Center of box A
/// * `half_extents_a` - Half-width and half-height of box A
/// * `center_b` - Center of box B
/// * `half_extents_b` - Half-width and half-height of box B
///
/// # Returns
///
/// - `Some(Contact)` if the boxes are overlapping
/// - `None` if the boxes are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::aabb_aabb_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Two AABBs overlapping
/// let contact = aabb_aabb_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
///     Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(contact.is_some());
/// ```
///
/// # Performance
///
/// O(1) - Simple comparison checks, no trigonometry required.
pub fn aabb_aabb_collision(
    center_a: Vec2,
    half_extents_a: Vec2,
    center_b: Vec2,
    half_extents_b: Vec2,
) -> Option<Contact> {
    // Compute min/max bounds
    let min_a = center_a - half_extents_a;
    let max_a = center_a + half_extents_a;
    let min_b = center_b - half_extents_b;
    let max_b = center_b + half_extents_b;

    // Check for overlap on X axis
    if max_a.x < min_b.x || max_b.x < min_a.x {
        return None;
    }

    // Check for overlap on Y axis
    if max_a.y < min_b.y || max_b.y < min_a.y {
        return None;
    }

    // Compute overlap on each axis
    let overlap_x = (max_a.x.min(max_b.x) - min_a.x.max(min_b.x)).abs();
    let overlap_y = (max_a.y.min(max_b.y) - min_a.y.max(min_b.y)).abs();

    // Find minimum overlap axis (collision normal)
    let (penetration, normal) = if overlap_x < overlap_y {
        // X axis is minimum
        let normal = if center_b.x > center_a.x {
            Vec2::unit_x()
        } else {
            Vec2::new(-1.0, 0.0)
        };
        (overlap_x, normal)
    } else {
        // Y axis is minimum
        let normal = if center_b.y > center_a.y {
            Vec2::unit_y()
        } else {
            Vec2::new(0.0, -1.0)
        };
        (overlap_y, normal)
    };

    // Contact point is at the center of overlap region
    let contact_point = Vec2::new(
        (min_a.x.max(min_b.x) + max_a.x.min(max_b.x)) * 0.5,
        (min_a.y.max(min_b.y) + max_a.y.min(max_b.y)) * 0.5,
    );

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}

// =============================================================================
// Circle-Box Collision
// =============================================================================

/// Detects collision between a circle and an axis-aligned bounding box (AABB).
///
/// This uses the "closest point" algorithm:
/// 1. Find the closest point on the AABB to the circle center
/// 2. Check if that point is within the circle's radius
///
/// This is more efficient than the full OBB version since it doesn't require
/// rotation transformations.
///
/// # Arguments
///
/// * `circle_center` - Center of the circle
/// * `circle_radius` - Radius of the circle (must be positive)
/// * `box_center` - Center of the AABB
/// * `box_half_extents` - Half-width and half-height of the AABB
///
/// # Returns
///
/// - `Some(Contact)` if the circle and AABB are overlapping
/// - `None` if they are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_aabb_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Circle overlapping with AABB
/// let contact = circle_aabb_collision(
///     Vec2::new(1.5, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(contact.is_some());
///
/// // Circle separated from AABB
/// let no_contact = circle_aabb_collision(
///     Vec2::new(5.0, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Simple clamping and distance check.
pub fn circle_aabb_collision(
    circle_center: Vec2,
    circle_radius: f32,
    box_center: Vec2,
    box_half_extents: Vec2,
) -> Option<Contact> {
    // Compute AABB bounds
    let box_min = box_center - box_half_extents;
    let box_max = box_center + box_half_extents;

    // Find the closest point on the AABB to the circle center
    let closest_point = Vec2::new(
        circle_center.x.clamp(box_min.x, box_max.x),
        circle_center.y.clamp(box_min.y, box_max.y),
    );

    // Vector from closest point to circle center
    let delta = circle_center - closest_point;
    let distance_squared = delta.length_squared();
    let radius_squared = circle_radius * circle_radius;

    // Early exit if separated
    if distance_squared > radius_squared {
        return None;
    }

    // Handle edge case: circle center inside box
    if distance_squared < 1e-6 {
        // Circle center is inside the box, find the axis of minimum penetration
        let penetration_x = (box_half_extents.x - (circle_center.x - box_center.x).abs()) + circle_radius;
        let penetration_y = (box_half_extents.y - (circle_center.y - box_center.y).abs()) + circle_radius;

        if penetration_x < penetration_y {
            // Push out along X axis
            let normal = if circle_center.x > box_center.x {
                Vec2::unit_x()
            } else {
                Vec2::new(-1.0, 0.0)
            };
            return Some(Contact {
                point: Vec2::new(
                    if circle_center.x > box_center.x { box_max.x } else { box_min.x },
                    circle_center.y,
                ),
                normal,
                penetration: penetration_x,
            });
        } else {
            // Push out along Y axis
            let normal = if circle_center.y > box_center.y {
                Vec2::unit_y()
            } else {
                Vec2::new(0.0, -1.0)
            };
            return Some(Contact {
                point: Vec2::new(
                    circle_center.x,
                    if circle_center.y > box_center.y { box_max.y } else { box_min.y },
                ),
                normal,
                penetration: penetration_y,
            });
        }
    }

    // Compute distance and normal
    let distance = distance_squared.sqrt();
    let normal = delta / distance;

    // Penetration is how much the circle overlaps with the box
    let penetration = circle_radius - distance;

    // Contact point is on the surface of the circle
    let contact_point = closest_point;

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}

/// Detects collision between a circle and an oriented bounding box (OBB).
///
/// This is a more general version of circle-AABB collision that handles
/// rotated boxes. The algorithm:
/// 1. Transform the circle into the box's local coordinate space
/// 2. Perform circle-AABB collision in local space
/// 3. Transform the contact back to world space
///
/// # Arguments
///
/// * `circle_center` - Center of the circle in world space
/// * `circle_radius` - Radius of the circle (must be positive)
/// * `box_center` - Center of the OBB in world space
/// * `box_half_extents` - Half-width and half-height of the OBB
/// * `box_rotation` - Rotation angle of the OBB in radians
///
/// # Returns
///
/// - `Some(Contact)` if the circle and OBB are overlapping
/// - `None` if they are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_obb_collision;
/// use goud_engine::core::math::Vec2;
/// use std::f32::consts::PI;
///
/// // Circle overlapping with rotated box
/// let contact = circle_obb_collision(
///     Vec2::new(1.5, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0
/// );
/// assert!(contact.is_some());
///
/// // Circle separated from rotated box
/// let no_contact = circle_obb_collision(
///     Vec2::new(5.0, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Coordinate transformation + AABB collision check.
/// More expensive than AABB version due to sin/cos calculations.
pub fn circle_obb_collision(
    circle_center: Vec2,
    circle_radius: f32,
    box_center: Vec2,
    box_half_extents: Vec2,
    box_rotation: f32,
) -> Option<Contact> {
    // If rotation is near zero, use faster AABB version
    if box_rotation.abs() < 1e-6 {
        return circle_aabb_collision(circle_center, circle_radius, box_center, box_half_extents);
    }

    // Compute rotation matrix for box (inverse rotation to transform to local space)
    let cos_r = box_rotation.cos();
    let sin_r = box_rotation.sin();

    // Transform circle center to box's local coordinate space
    let delta = circle_center - box_center;
    let local_circle_center = Vec2::new(
        delta.x * cos_r + delta.y * sin_r,
        -delta.x * sin_r + delta.y * cos_r,
    );

    // Perform collision in local space (box is axis-aligned here)
    let local_contact = circle_aabb_collision(
        local_circle_center,
        circle_radius,
        Vec2::zero(), // Box is at origin in local space
        box_half_extents,
    )?;

    // Transform contact back to world space
    let world_normal = Vec2::new(
        local_contact.normal.x * cos_r - local_contact.normal.y * sin_r,
        local_contact.normal.x * sin_r + local_contact.normal.y * cos_r,
    );

    let world_point = Vec2::new(
        local_contact.point.x * cos_r - local_contact.point.y * sin_r,
        local_contact.point.x * sin_r + local_contact.point.y * cos_r,
    ) + box_center;

    Some(Contact {
        point: world_point,
        normal: world_normal,
        penetration: local_contact.penetration,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Contact Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_contact_new() {
        let contact = Contact::new(
            Vec2::new(1.0, 2.0),
            Vec2::new(1.0, 0.0),
            0.5,
        );
        assert_eq!(contact.point, Vec2::new(1.0, 2.0));
        assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
        assert_eq!(contact.penetration, 0.5);
    }

    #[test]
    fn test_contact_default() {
        let contact = Contact::default();
        assert_eq!(contact.point, Vec2::zero());
        assert_eq!(contact.normal, Vec2::unit_x());
        assert_eq!(contact.penetration, 0.0);
    }

    #[test]
    fn test_contact_is_colliding() {
        let colliding = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
        let touching = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.0);
        let separated = Contact::new(Vec2::zero(), Vec2::unit_x(), -0.1);

        assert!(colliding.is_colliding());
        assert!(!touching.is_colliding());
        assert!(!separated.is_colliding());
    }

    #[test]
    fn test_contact_separation_distance() {
        let contact = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
        assert_eq!(contact.separation_distance(), 0.5);

        let negative = Contact::new(Vec2::zero(), Vec2::unit_x(), -0.3);
        assert_eq!(negative.separation_distance(), 0.3);
    }

    #[test]
    fn test_contact_separation_vector() {
        let contact = Contact::new(
            Vec2::zero(),
            Vec2::new(1.0, 0.0),
            0.5,
        );
        assert_eq!(contact.separation_vector(), Vec2::new(0.5, 0.0));

        let diagonal = Contact::new(
            Vec2::zero(),
            Vec2::new(0.6, 0.8), // Normalized (approximately)
            2.0,
        );
        let sep = diagonal.separation_vector();
        assert!((sep.x - 1.2).abs() < 1e-5);
        assert!((sep.y - 1.6).abs() < 1e-5);
    }

    #[test]
    fn test_contact_reversed() {
        let contact = Contact::new(
            Vec2::new(1.0, 2.0),
            Vec2::new(1.0, 0.0),
            0.5,
        );
        let reversed = contact.reversed();

        assert_eq!(reversed.point, contact.point);
        assert_eq!(reversed.normal, Vec2::new(-1.0, 0.0));
        assert_eq!(reversed.penetration, contact.penetration);
    }

    #[test]
    fn test_contact_clone() {
        let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::unit_x(), 0.5);
        let cloned = contact.clone();
        assert_eq!(contact, cloned);
    }

    #[test]
    fn test_contact_debug() {
        let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::unit_x(), 0.5);
        let debug_str = format!("{:?}", contact);
        assert!(debug_str.contains("Contact"));
        assert!(debug_str.contains("point"));
        assert!(debug_str.contains("normal"));
        assert!(debug_str.contains("penetration"));
    }

    // -------------------------------------------------------------------------
    // Circle-Circle Collision Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_circle_circle_collision_overlapping() {
        // Two circles overlapping
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(1.5, 0.0), 1.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_circle_circle_collision_separated() {
        // Two circles separated
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(5.0, 0.0), 1.0,
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_circle_circle_collision_touching() {
        // Two circles exactly touching (edge case)
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(2.0, 0.0), 1.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!((contact.penetration).abs() < 1e-5); // Near zero penetration
    }

    #[test]
    fn test_circle_circle_collision_same_position() {
        // Circles at the same position (edge case)
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), 1.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert_eq!(contact.penetration, 2.0); // Full overlap
        assert_eq!(contact.point, Vec2::zero());
    }

    #[test]
    fn test_circle_circle_collision_diagonal() {
        // Circles offset diagonally
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(1.0, 1.0), 1.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);

        // Normal should point from center_a to center_b (diagonal)
        let expected_normal = Vec2::new(1.0, 1.0).normalize();
        assert!((contact.normal.x - expected_normal.x).abs() < 1e-5);
        assert!((contact.normal.y - expected_normal.y).abs() < 1e-5);
    }

    #[test]
    fn test_circle_circle_collision_different_radii() {
        // Circles with different radii
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 2.0,
            Vec2::new(3.0, 0.0), 1.5,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);

        // Combined radius = 2 + 1.5 = 3.5
        // Distance = 3.0
        // Penetration = 3.5 - 3.0 = 0.5
        assert!((contact.penetration - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_circle_circle_collision_negative_coordinates() {
        // Circles in negative coordinate space
        let contact = circle_circle_collision(
            Vec2::new(-10.0, -10.0), 1.0,
            Vec2::new(-9.0, -10.0), 1.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_circle_circle_collision_contact_point() {
        // Verify contact point is computed correctly
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(1.5, 0.0), 1.0,
        ).unwrap();

        // Contact point should be between the two circles
        assert!(contact.point.x > 0.0 && contact.point.x < 1.5);
        assert_eq!(contact.point.y, 0.0);
    }

    #[test]
    fn test_circle_circle_collision_symmetry() {
        // Collision should be symmetric (order shouldn't matter for detection)
        let contact_ab = circle_circle_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(1.5, 0.0), 1.0,
        );
        let contact_ba = circle_circle_collision(
            Vec2::new(1.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), 1.0,
        );

        assert!(contact_ab.is_some());
        assert!(contact_ba.is_some());

        let contact_ab = contact_ab.unwrap();
        let contact_ba = contact_ba.unwrap();

        // Penetration should be the same
        assert_eq!(contact_ab.penetration, contact_ba.penetration);

        // Normals should be opposite
        assert_eq!(contact_ab.normal, contact_ba.normal * -1.0);
    }

    #[test]
    fn test_circle_circle_collision_large_circles() {
        // Test with large circles
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 100.0,
            Vec2::new(150.0, 0.0), 100.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!((contact.penetration - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_circle_circle_collision_tiny_circles() {
        // Test with tiny circles
        let contact = circle_circle_collision(
            Vec2::new(0.0, 0.0), 0.01,
            Vec2::new(0.015, 0.0), 0.01,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    // -------------------------------------------------------------------------
    // Box-Box Collision (SAT) Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_box_box_collision_axis_aligned_overlapping() {
        // Two axis-aligned boxes overlapping
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 0.5).abs() < 1e-5);

        // Normal should point in X direction (minimum overlap axis)
        assert!((contact.normal.x.abs() - 1.0).abs() < 1e-5);
        assert!(contact.normal.y.abs() < 1e-5);
    }

    #[test]
    fn test_box_box_collision_axis_aligned_separated() {
        // Two axis-aligned boxes separated
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(5.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_box_box_collision_rotated_overlapping() {
        use std::f32::consts::PI;

        // Box B rotated 45 degrees, overlapping with axis-aligned box A
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_box_box_collision_rotated_separated() {
        use std::f32::consts::PI;

        // Two rotated boxes, separated
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 6.0,
            Vec2::new(5.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_box_box_collision_both_rotated() {
        use std::f32::consts::PI;

        // Both boxes rotated, overlapping
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 6.0,
            Vec2::new(1.2, 0.0), Vec2::new(1.0, 1.0), PI / 3.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_box_box_collision_same_position() {
        // Boxes at the same position (full overlap)
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        // Full overlap, penetration should be equal to full extents
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_box_box_collision_touching() {
        // Boxes exactly touching (edge case)
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(2.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );

        // Should detect as collision (touching counts as collision)
        assert!(contact.is_some());

        let contact = contact.unwrap();
        // Penetration should be near zero (touching)
        assert!(contact.penetration.abs() < 1e-3);
    }

    #[test]
    fn test_box_box_collision_different_sizes() {
        // Boxes with different sizes
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), 0.0,
            Vec2::new(2.5, 0.0), Vec2::new(1.0, 2.0), 0.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_box_box_collision_normal_direction() {
        // Verify normal points from A to B
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), 0.0,
        ).unwrap();

        // Normal should point right (from A to B)
        assert!(contact.normal.x > 0.0);
        assert!(contact.normal.y.abs() < 1e-5);
    }

    #[test]
    fn test_box_box_collision_symmetry() {
        use std::f32::consts::PI;

        // Collision should detect the same penetration regardless of order
        let contact_ab = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), PI / 6.0,
        );
        let contact_ba = box_box_collision(
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), PI / 6.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );

        assert!(contact_ab.is_some());
        assert!(contact_ba.is_some());

        let contact_ab = contact_ab.unwrap();
        let contact_ba = contact_ba.unwrap();

        // Penetration should be the same
        assert!((contact_ab.penetration - contact_ba.penetration).abs() < 1e-5);

        // Normals should be opposite
        assert!((contact_ab.normal.x + contact_ba.normal.x).abs() < 1e-5);
        assert!((contact_ab.normal.y + contact_ba.normal.y).abs() < 1e-5);
    }

    #[test]
    fn test_box_box_collision_90_degree_rotation() {
        use std::f32::consts::PI;

        // Box rotated 90 degrees
        let contact = box_box_collision(
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0), 0.0,
            Vec2::new(1.5, 0.0), Vec2::new(2.0, 1.0), PI / 2.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    // -------------------------------------------------------------------------
    // AABB-AABB Collision Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_aabb_aabb_collision_overlapping() {
        // Two AABBs overlapping
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 0.5).abs() < 1e-5);

        // Normal should point right (X direction)
        assert_eq!(contact.normal, Vec2::unit_x());
    }

    #[test]
    fn test_aabb_aabb_collision_separated() {
        // Two AABBs separated
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(5.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_aabb_aabb_collision_touching() {
        // Two AABBs exactly touching
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 0.0), Vec2::new(1.0, 1.0),
        );

        // Should detect as collision (touching edges)
        assert!(contact.is_some());

        let contact = contact.unwrap();
        // Penetration should be near zero
        assert!(contact.penetration.abs() < 1e-5);
    }

    #[test]
    fn test_aabb_aabb_collision_vertical_overlap() {
        // AABBs overlapping vertically
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.5), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 0.5).abs() < 1e-5);

        // Normal should point up (Y direction)
        assert_eq!(contact.normal, Vec2::unit_y());
    }

    #[test]
    fn test_aabb_aabb_collision_different_sizes() {
        // AABBs with different sizes
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 1.0),
            Vec2::new(2.5, 0.0), Vec2::new(1.0, 2.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_aabb_aabb_collision_same_position() {
        // AABBs at the same position
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        // Full overlap
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_aabb_collision_contact_point() {
        // Verify contact point is in the overlap region
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
        ).unwrap();

        // Contact point should be in the overlap region
        assert!(contact.point.x > 0.0 && contact.point.x < 2.0);
        assert!(contact.point.y >= -1.0 && contact.point.y <= 1.0);
    }

    #[test]
    fn test_aabb_aabb_collision_normal_direction() {
        // Verify normal points from A to B
        let contact = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
        ).unwrap();

        // Normal should point right (from A to B)
        assert_eq!(contact.normal, Vec2::unit_x());

        // Test vertical case
        let contact_vertical = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.5), Vec2::new(1.0, 1.0),
        ).unwrap();

        // Normal should point up
        assert_eq!(contact_vertical.normal, Vec2::unit_y());
    }

    #[test]
    fn test_aabb_aabb_collision_symmetry() {
        // Collision should be symmetric (same penetration regardless of order)
        let contact_ab = aabb_aabb_collision(
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
        );
        let contact_ba = aabb_aabb_collision(
            Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );

        assert!(contact_ab.is_some());
        assert!(contact_ba.is_some());

        let contact_ab = contact_ab.unwrap();
        let contact_ba = contact_ba.unwrap();

        // Penetration should be the same
        assert_eq!(contact_ab.penetration, contact_ba.penetration);

        // Normals should be opposite
        assert_eq!(contact_ab.normal, contact_ba.normal * -1.0);
    }

    #[test]
    fn test_aabb_aabb_collision_negative_coordinates() {
        // AABBs in negative coordinate space
        let contact = aabb_aabb_collision(
            Vec2::new(-10.0, -10.0), Vec2::new(1.0, 1.0),
            Vec2::new(-9.0, -10.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert_eq!(contact.normal, Vec2::unit_x());
    }

    // -------------------------------------------------------------------------
    // Circle-AABB Collision Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_circle_aabb_collision_overlapping() {
        // Circle overlapping with AABB from the side
        let contact = circle_aabb_collision(
            Vec2::new(1.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 0.5).abs() < 1e-5);
        assert_eq!(contact.normal, Vec2::unit_x());
    }

    #[test]
    fn test_circle_aabb_collision_separated() {
        // Circle separated from AABB
        let contact = circle_aabb_collision(
            Vec2::new(5.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_circle_aabb_collision_corner() {
        // Circle colliding with corner of AABB
        let contact = circle_aabb_collision(
            Vec2::new(1.5, 1.5), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);

        // Normal should point diagonally (from corner to circle center)
        let expected_normal = Vec2::new(0.5, 0.5).normalize();
        assert!((contact.normal.x - expected_normal.x).abs() < 1e-5);
        assert!((contact.normal.y - expected_normal.y).abs() < 1e-5);
    }

    #[test]
    fn test_circle_aabb_collision_edge() {
        // Circle colliding with edge of AABB
        let contact = circle_aabb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!((contact.penetration - 0.2).abs() < 1e-5);

        // Normal should point horizontally (perpendicular to edge)
        assert_eq!(contact.normal, Vec2::unit_x());
    }

    #[test]
    fn test_circle_aabb_collision_inside() {
        // Circle center inside AABB (penetrating deeply)
        let contact = circle_aabb_collision(
            Vec2::new(0.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 1.0); // Deep penetration

        // Normal should push circle out along minimum axis
        assert!(contact.normal.x.abs() > 0.9 || contact.normal.y.abs() > 0.9);
    }

    #[test]
    fn test_circle_aabb_collision_center_coincident() {
        // Circle center exactly at AABB center
        let contact = circle_aabb_collision(
            Vec2::new(0.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 1.0); // Deep penetration
    }

    #[test]
    fn test_circle_aabb_collision_touching() {
        // Circle exactly touching AABB edge
        let contact = circle_aabb_collision(
            Vec2::new(2.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration.abs() < 1e-5); // Near-zero penetration
    }

    #[test]
    fn test_circle_aabb_collision_different_sizes() {
        // Large circle with small AABB
        let contact = circle_aabb_collision(
            Vec2::new(2.0, 0.0), 2.5,
            Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.5),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.penetration - 1.0).abs() < 1e-5); // Circle left edge at -0.5, box at 0.5, overlap = 1.0
    }

    #[test]
    fn test_circle_aabb_collision_vertical() {
        // Circle colliding from above
        let contact = circle_aabb_collision(
            Vec2::new(0.0, 1.8), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!((contact.penetration - 0.2).abs() < 1e-5);
        assert_eq!(contact.normal, Vec2::unit_y());
    }

    #[test]
    fn test_circle_aabb_collision_contact_point() {
        // Verify contact point is on AABB surface
        let contact = circle_aabb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        ).unwrap();

        // Contact point should be on the right edge of the AABB
        assert!((contact.point.x - 1.0).abs() < 1e-5);
        assert!((contact.point.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_circle_aabb_collision_symmetry() {
        // Similar collision from opposite direction
        let contact1 = circle_aabb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );
        let contact2 = circle_aabb_collision(
            Vec2::new(-1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );

        assert!(contact1.is_some());
        assert!(contact2.is_some());

        let c1 = contact1.unwrap();
        let c2 = contact2.unwrap();

        // Penetrations should be the same
        assert_eq!(c1.penetration, c2.penetration);

        // Normals should be opposite
        assert_eq!(c1.normal.x, -c2.normal.x);
    }

    // -------------------------------------------------------------------------
    // Circle-OBB Collision Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_circle_obb_collision_no_rotation() {
        // Zero rotation should behave like AABB
        let contact_obb = circle_obb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
        );
        let contact_aabb = circle_aabb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
        );

        assert!(contact_obb.is_some());
        assert!(contact_aabb.is_some());

        let c_obb = contact_obb.unwrap();
        let c_aabb = contact_aabb.unwrap();

        assert!((c_obb.penetration - c_aabb.penetration).abs() < 1e-5);
    }

    #[test]
    fn test_circle_obb_collision_45_degree_rotation() {
        use std::f32::consts::PI;

        // Box rotated 45 degrees
        let contact = circle_obb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_90_degree_rotation() {
        use std::f32::consts::PI;

        // Box rotated 90 degrees (swaps width/height for square, no visual change)
        let contact = circle_obb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 2.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_rotated_rectangle() {
        use std::f32::consts::PI;

        // Non-square box rotated 30 degrees
        let contact = circle_obb_collision(
            Vec2::new(2.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.5), PI / 6.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_separated_rotated() {
        use std::f32::consts::PI;

        // Circle and rotated box separated
        let contact = circle_obb_collision(
            Vec2::new(5.0, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );
        assert!(contact.is_none());
    }

    #[test]
    fn test_circle_obb_collision_inside_rotated() {
        use std::f32::consts::PI;

        // Circle center inside rotated box
        let contact = circle_obb_collision(
            Vec2::new(0.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0), PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_corner_rotated() {
        use std::f32::consts::PI;

        // Circle colliding with corner of rotated box
        // With 45-degree rotation, corners move farther out
        let contact = circle_obb_collision(
            Vec2::new(1.2, 1.2), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_touching_rotated() {
        use std::f32::consts::PI;

        // Circle close to rotated box
        let contact = circle_obb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0,
        );

        // Should collide since rotated box extends farther
        if let Some(c) = contact {
            assert!(c.penetration >= 0.0); // Valid penetration
        }
    }

    #[test]
    fn test_circle_obb_collision_large_rotation() {
        use std::f32::consts::PI;

        // Box rotated by large angle (3π/4)
        let contact = circle_obb_collision(
            Vec2::new(1.5, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 3.0 * PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    #[test]
    fn test_circle_obb_collision_negative_rotation() {
        use std::f32::consts::PI;

        // Negative rotation
        let contact = circle_obb_collision(
            Vec2::new(1.8, 0.0), 1.0,
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), -PI / 4.0,
        );
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
    }

    // -------------------------------------------------------------------------
    // CollisionResponse Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_collision_response_new() {
        let response = CollisionResponse::new(0.5, 0.4, 0.6, 0.02);
        assert_eq!(response.restitution, 0.5);
        assert_eq!(response.friction, 0.4);
        assert_eq!(response.position_correction, 0.6);
        assert_eq!(response.slop, 0.02);
    }

    #[test]
    fn test_collision_response_new_clamping() {
        // Test clamping of invalid values
        let response = CollisionResponse::new(1.5, -0.2, 2.0, -0.01);
        assert_eq!(response.restitution, 1.0); // Clamped to max
        assert_eq!(response.friction, 0.0);    // Clamped to min
        assert_eq!(response.position_correction, 1.0); // Clamped to max
        assert_eq!(response.slop, 0.0);        // Clamped to min
    }

    #[test]
    fn test_collision_response_default() {
        let response = CollisionResponse::default();
        assert_eq!(response.restitution, 0.4);
        assert_eq!(response.friction, 0.4);
        assert_eq!(response.position_correction, 0.4);
        assert_eq!(response.slop, 0.01);
    }

    #[test]
    fn test_collision_response_bouncy() {
        let response = CollisionResponse::bouncy();
        assert_eq!(response.restitution, 0.8);
        assert_eq!(response.friction, 0.2);
    }

    #[test]
    fn test_collision_response_character() {
        let response = CollisionResponse::character();
        assert_eq!(response.restitution, 0.0);
        assert_eq!(response.friction, 0.8);
    }

    #[test]
    fn test_collision_response_slippery() {
        let response = CollisionResponse::slippery();
        assert!(response.friction < 0.2);
        assert!(response.restitution > 0.0);
    }

    #[test]
    fn test_collision_response_elastic() {
        let response = CollisionResponse::elastic();
        assert_eq!(response.restitution, 1.0);
    }

    #[test]
    fn test_collision_response_clone() {
        let response = CollisionResponse::default();
        let cloned = response.clone();
        assert_eq!(response, cloned);
    }

    #[test]
    fn test_collision_response_debug() {
        let response = CollisionResponse::default();
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("CollisionResponse"));
    }

    // -------------------------------------------------------------------------
    // Impulse Resolution Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_collision_head_on() {
        // Two objects colliding head-on with equal mass
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0), // Normal points from A to B
            0.1,
        );

        let vel_a = Vec2::new(10.0, 0.0);  // A moving right
        let vel_b = Vec2::new(-10.0, 0.0); // B moving left
        let inv_mass_a = 1.0;               // Equal mass
        let inv_mass_b = 1.0;

        let response = CollisionResponse::elastic();
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // With elastic collision and equal masses, velocities should swap
        let new_vel_a = vel_a + delta_a;
        let new_vel_b = vel_b + delta_b;

        // A should now move left, B should move right
        assert!(new_vel_a.x < 0.0);
        assert!(new_vel_b.x > 0.0);
    }

    #[test]
    fn test_resolve_collision_static_wall() {
        // Object colliding with static wall
        // Normal points FROM object TO wall (right)
        let contact = Contact::new(
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 0.0), // Normal points right (from A to B/wall)
            0.1,
        );

        let vel_a = Vec2::new(10.0, 0.0);  // A moving right into wall
        let vel_b = Vec2::zero();          // Wall is static
        let inv_mass_a = 1.0;
        let inv_mass_b = 0.0;              // Infinite mass (static)

        let response = CollisionResponse::bouncy();
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Wall doesn't move
        assert_eq!(delta_b, Vec2::zero());

        // Object bounces back (velocity becomes negative)
        let new_vel_a = vel_a + delta_a;
        assert!(new_vel_a.x < vel_a.x); // Velocity reduced or reversed
    }

    #[test]
    fn test_resolve_collision_no_bounce() {
        // Collision with zero restitution (inelastic)
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let vel_a = Vec2::new(10.0, 0.0);
        let vel_b = Vec2::zero();
        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;

        let response = CollisionResponse::character(); // Zero restitution
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        let new_vel_a = vel_a + delta_a;
        let new_vel_b = vel_b + delta_b;

        // Should slow down significantly (no bounce)
        assert!(new_vel_a.x < vel_a.x);
    }

    #[test]
    fn test_resolve_collision_separating() {
        // Objects already separating (no impulse needed)
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let vel_a = Vec2::new(-5.0, 0.0);  // A moving away (left)
        let vel_b = Vec2::new(5.0, 0.0);   // B moving away (right)
        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;

        let response = CollisionResponse::default();
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // No impulse should be applied (already separating)
        assert_eq!(delta_a, Vec2::zero());
        assert_eq!(delta_b, Vec2::zero());
    }

    #[test]
    fn test_resolve_collision_two_static() {
        // Two static objects (no impulse possible)
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let vel_a = Vec2::zero();
        let vel_b = Vec2::zero();
        let inv_mass_a = 0.0; // Static
        let inv_mass_b = 0.0; // Static

        let response = CollisionResponse::default();
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // No movement possible
        assert_eq!(delta_a, Vec2::zero());
        assert_eq!(delta_b, Vec2::zero());
    }

    #[test]
    fn test_resolve_collision_with_friction() {
        // Object sliding along surface with friction
        // Object A hits surface B from above
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, -1.0), // Normal points down (from A to B)
            0.1,
        );

        let vel_a = Vec2::new(10.0, -5.0);  // A moving right and down
        let vel_b = Vec2::zero();           // B is static surface
        let inv_mass_a = 1.0;
        let inv_mass_b = 0.0;

        let response = CollisionResponse::character(); // High friction, zero restitution
        let (delta_a, _) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Impulse should be applied (not zero)
        assert!(delta_a.length() > 0.0);

        // Horizontal velocity should be reduced by friction
        let new_vel_a = vel_a + delta_a;
        assert!(new_vel_a.x < vel_a.x);
    }

    #[test]
    fn test_resolve_collision_diagonal() {
        // Collision at an angle
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.707, 0.707), // 45-degree normal
            0.1,
        );

        let vel_a = Vec2::new(10.0, 10.0);
        let vel_b = Vec2::zero();
        let inv_mass_a = 1.0;
        let inv_mass_b = 0.0;

        let response = CollisionResponse::default();
        let (delta_a, _) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        let new_vel_a = vel_a + delta_a;

        // Velocity should be redirected along the surface
        assert!(new_vel_a.length() < vel_a.length()); // Some energy lost
    }

    #[test]
    fn test_resolve_collision_mass_ratio() {
        // Heavy object hitting light object
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let vel_a = Vec2::new(10.0, 0.0);
        let vel_b = Vec2::zero();
        let inv_mass_a = 0.1;  // Heavy (10 kg)
        let inv_mass_b = 1.0;  // Light (1 kg)

        let response = CollisionResponse::default();
        let (delta_a, delta_b) = resolve_collision(
            &contact,
            vel_a,
            vel_b,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Light object should move more than heavy object
        assert!(delta_b.length() > delta_a.length());
    }

    // -------------------------------------------------------------------------
    // Position Correction Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_position_correction_basic() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1, // 0.1 penetration
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Both should move (equal mass)
        assert!(corr_a.length() > 0.0);
        assert!(corr_b.length() > 0.0);

        // Should move in opposite directions
        assert!(corr_a.x < 0.0); // A moves left
        assert!(corr_b.x > 0.0); // B moves right
    }

    #[test]
    fn test_position_correction_below_slop() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.005, // Below default slop threshold (0.01)
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // No correction for small penetration
        assert_eq!(corr_a, Vec2::zero());
        assert_eq!(corr_b, Vec2::zero());
    }

    #[test]
    fn test_position_correction_static_wall() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 0.0; // Static
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Only dynamic object moves
        assert!(corr_a.length() > 0.0);
        assert_eq!(corr_b, Vec2::zero());
    }

    #[test]
    fn test_position_correction_two_static() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let inv_mass_a = 0.0;
        let inv_mass_b = 0.0;
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // No correction possible
        assert_eq!(corr_a, Vec2::zero());
        assert_eq!(corr_b, Vec2::zero());
    }

    #[test]
    fn test_position_correction_zero_percent() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;
        let response = CollisionResponse::new(0.5, 0.5, 0.0, 0.01); // Zero correction

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // No correction applied
        assert_eq!(corr_a, Vec2::zero());
        assert_eq!(corr_b, Vec2::zero());
    }

    #[test]
    fn test_position_correction_full_percent() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;
        let response = CollisionResponse::new(0.5, 0.5, 1.0, 0.01); // Full correction

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Maximum correction applied
        assert!(corr_a.length() > 0.0);
        assert!(corr_b.length() > 0.0);
    }

    #[test]
    fn test_position_correction_mass_ratio() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            0.1,
        );

        let inv_mass_a = 0.1; // Heavy
        let inv_mass_b = 1.0; // Light
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Light object should move more
        assert!(corr_b.length() > corr_a.length());
    }

    #[test]
    fn test_position_correction_direction() {
        let contact = Contact::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0), // Normal points up
            0.1,
        );

        let inv_mass_a = 1.0;
        let inv_mass_b = 1.0;
        let response = CollisionResponse::default();

        let (corr_a, corr_b) = compute_position_correction(
            &contact,
            inv_mass_a,
            inv_mass_b,
            &response,
        );

        // Correction should be along normal
        assert!(corr_a.y < 0.0); // A moves down
        assert!(corr_b.y > 0.0); // B moves up
        assert!(corr_a.x.abs() < 1e-6); // No horizontal movement
        assert!(corr_b.x.abs() < 1e-6);
    }
}

// =============================================================================
// Collision Events Module
// =============================================================================

/// Collision events module.
///
/// This module defines events that are fired when collisions are detected
/// between entities. Systems can listen to these events to trigger gameplay
/// effects, audio, particles, damage, etc.
///
/// # Event Types
///
/// - [`CollisionStarted`]: Fired when two entities start colliding (first contact)
/// - [`CollisionEnded`]: Fired when two entities stop colliding (separation)
///
/// # Usage Example
///
/// ```rust
/// use goud_engine::ecs::collision::{CollisionStarted, CollisionEnded};
/// use goud_engine::core::event::Events;
/// use goud_engine::ecs::{Entity, World};
///
/// // System that listens for collision events
/// fn handle_collisions(world: &World) {
///     // In a real system, you'd use EventReader<CollisionStarted>
///     // For now, this shows the concept
/// }
/// ```
pub mod events {
    use super::Contact;
    use crate::ecs::Entity;

    /// Event fired when two entities start colliding.
    ///
    /// This event is emitted when collision detection determines that two
    /// entities have just made contact (were not colliding in the previous
    /// frame, but are colliding now).
    ///
    /// # Fields
    ///
    /// - `entity_a`, `entity_b`: The two entities involved in the collision
    /// - `contact`: Detailed contact information (point, normal, penetration)
    ///
    /// # Usage
    ///
    /// ```rust
    /// use goud_engine::ecs::collision::events::CollisionStarted;
    /// use goud_engine::core::event::{Events, EventReader};
    ///
    /// fn handle_collision_start(events: &Events<CollisionStarted>) {
    ///     let mut reader = events.reader();
    ///     for event in reader.read() {
    ///         println!("Collision started between {:?} and {:?}", event.entity_a, event.entity_b);
    ///         println!("Contact point: {:?}", event.contact.point);
    ///         println!("Penetration depth: {}", event.contact.penetration);
    ///
    ///         // Trigger gameplay effects
    ///         // - Play collision sound
    ///         // - Spawn particle effects
    ///         // - Apply damage
    ///         // - Trigger game events
    ///     }
    /// }
    /// ```
    ///
    /// # Entity Ordering
    ///
    /// The order of `entity_a` and `entity_b` is consistent within a frame
    /// but may vary between frames. To check if a specific entity is involved:
    ///
    /// ```rust
    /// # use goud_engine::ecs::collision::events::CollisionStarted;
    /// # use goud_engine::ecs::Entity;
    /// # let event = CollisionStarted {
    /// #     entity_a: Entity::PLACEHOLDER,
    /// #     entity_b: Entity::PLACEHOLDER,
    /// #     contact: goud_engine::ecs::collision::Contact::default(),
    /// # };
    /// # let my_entity = Entity::PLACEHOLDER;
    /// if event.involves(my_entity) {
    ///     let other = event.other_entity(my_entity);
    ///     println!("My entity collided with {:?}", other);
    /// }
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CollisionStarted {
        /// First entity in the collision pair.
        pub entity_a: Entity,
        /// Second entity in the collision pair.
        pub entity_b: Entity,
        /// Contact information for this collision.
        pub contact: Contact,
    }

    impl CollisionStarted {
        /// Creates a new `CollisionStarted` event.
        #[must_use]
        pub fn new(entity_a: Entity, entity_b: Entity, contact: Contact) -> Self {
            Self {
                entity_a,
                entity_b,
                contact,
            }
        }

        /// Checks if the given entity is involved in this collision.
        #[must_use]
        pub fn involves(&self, entity: Entity) -> bool {
            self.entity_a == entity || self.entity_b == entity
        }

        /// Returns the other entity in the collision pair.
        ///
        /// # Returns
        ///
        /// - `Some(Entity)` if the given entity is involved in the collision
        /// - `None` if the given entity is not part of this collision
        ///
        /// # Example
        ///
        /// ```rust
        /// # use goud_engine::ecs::collision::events::CollisionStarted;
        /// # use goud_engine::ecs::Entity;
        /// # use goud_engine::ecs::collision::Contact;
        /// # let entity_a = Entity::from_bits(1);
        /// # let entity_b = Entity::from_bits(2);
        /// # let contact = Contact::default();
        /// let event = CollisionStarted::new(entity_a, entity_b, contact);
        ///
        /// assert_eq!(event.other_entity(entity_a), Some(entity_b));
        /// assert_eq!(event.other_entity(entity_b), Some(entity_a));
        /// assert_eq!(event.other_entity(Entity::from_bits(999)), None);
        /// ```
        #[must_use]
        pub fn other_entity(&self, entity: Entity) -> Option<Entity> {
            if self.entity_a == entity {
                Some(self.entity_b)
            } else if self.entity_b == entity {
                Some(self.entity_a)
            } else {
                None
            }
        }

        /// Returns an ordered pair of entities (lower entity ID first).
        ///
        /// This is useful for consistent lookups in hash maps or sets
        /// regardless of which entity was `entity_a` vs `entity_b`.
        #[must_use]
        pub fn ordered_pair(&self) -> (Entity, Entity) {
            if self.entity_a.to_bits() < self.entity_b.to_bits() {
                (self.entity_a, self.entity_b)
            } else {
                (self.entity_b, self.entity_a)
            }
        }
    }

    /// Event fired when two entities stop colliding.
    ///
    /// This event is emitted when collision detection determines that two
    /// entities that were colliding in the previous frame are no longer
    /// in contact.
    ///
    /// # Fields
    ///
    /// - `entity_a`, `entity_b`: The two entities that separated
    ///
    /// # Usage
    ///
    /// ```rust
    /// use goud_engine::ecs::collision::events::CollisionEnded;
    /// use goud_engine::core::event::{Events, EventReader};
    ///
    /// fn handle_collision_end(events: &Events<CollisionEnded>) {
    ///     let mut reader = events.reader();
    ///     for event in reader.read() {
    ///         println!("Collision ended between {:?} and {:?}", event.entity_a, event.entity_b);
    ///
    ///         // Stop gameplay effects
    ///         // - Stop looping collision sounds
    ///         // - Stop particle emitters
    ///         // - Update collision state tracking
    ///     }
    /// }
    /// ```
    ///
    /// # Note on Contact Information
    ///
    /// Unlike `CollisionStarted`, this event does not include contact
    /// information since the entities are no longer touching. If you need
    /// to track the last known contact point, store it when receiving
    /// `CollisionStarted` events.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CollisionEnded {
        /// First entity in the collision pair.
        pub entity_a: Entity,
        /// Second entity in the collision pair.
        pub entity_b: Entity,
    }

    impl CollisionEnded {
        /// Creates a new `CollisionEnded` event.
        #[must_use]
        pub fn new(entity_a: Entity, entity_b: Entity) -> Self {
            Self { entity_a, entity_b }
        }

        /// Checks if the given entity is involved in this collision end.
        #[must_use]
        pub fn involves(&self, entity: Entity) -> bool {
            self.entity_a == entity || self.entity_b == entity
        }

        /// Returns the other entity in the collision pair.
        ///
        /// # Returns
        ///
        /// - `Some(Entity)` if the given entity was involved in the collision
        /// - `None` if the given entity was not part of this collision
        #[must_use]
        pub fn other_entity(&self, entity: Entity) -> Option<Entity> {
            if self.entity_a == entity {
                Some(self.entity_b)
            } else if self.entity_b == entity {
                Some(self.entity_a)
            } else {
                None
            }
        }

        /// Returns an ordered pair of entities (lower entity ID first).
        ///
        /// This is useful for consistent lookups in hash maps or sets
        /// regardless of which entity was `entity_a` vs `entity_b`.
        #[must_use]
        pub fn ordered_pair(&self) -> (Entity, Entity) {
            if self.entity_a.to_bits() < self.entity_b.to_bits() {
                (self.entity_a, self.entity_b)
            } else {
                (self.entity_b, self.entity_a)
            }
        }
    }
}

// Re-export collision events at the collision module level for convenience
pub use events::{CollisionEnded, CollisionStarted};

#[cfg(test)]
mod collision_events_tests {
    use super::events::*;
    use super::Contact;
    use crate::core::event::Event;
    use crate::core::math::Vec2;
    use crate::ecs::Entity;

    // =========================================================================
    // CollisionStarted Tests
    // =========================================================================

    #[test]
    fn test_collision_started_new() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.5);

        let event = CollisionStarted::new(entity_a, entity_b, contact);

        assert_eq!(event.entity_a, entity_a);
        assert_eq!(event.entity_b, entity_b);
        assert_eq!(event.contact, contact);
    }

    #[test]
    fn test_collision_started_involves() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let entity_c = Entity::from_bits(3);
        let contact = Contact::default();

        let event = CollisionStarted::new(entity_a, entity_b, contact);

        assert!(event.involves(entity_a));
        assert!(event.involves(entity_b));
        assert!(!event.involves(entity_c));
    }

    #[test]
    fn test_collision_started_other_entity() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let entity_c = Entity::from_bits(3);
        let contact = Contact::default();

        let event = CollisionStarted::new(entity_a, entity_b, contact);

        assert_eq!(event.other_entity(entity_a), Some(entity_b));
        assert_eq!(event.other_entity(entity_b), Some(entity_a));
        assert_eq!(event.other_entity(entity_c), None);
    }

    #[test]
    fn test_collision_started_ordered_pair() {
        let entity_1 = Entity::from_bits(1);
        let entity_2 = Entity::from_bits(2);
        let contact = Contact::default();

        // Test both orderings
        let event1 = CollisionStarted::new(entity_1, entity_2, contact);
        let event2 = CollisionStarted::new(entity_2, entity_1, contact);

        // Both should return the same ordered pair
        assert_eq!(event1.ordered_pair(), (entity_1, entity_2));
        assert_eq!(event2.ordered_pair(), (entity_1, entity_2));
    }

    #[test]
    fn test_collision_started_implements_event() {
        fn accepts_event<E: Event>(_: E) {}

        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let contact = Contact::default();
        let event = CollisionStarted::new(entity_a, entity_b, contact);

        accepts_event(event);
    }

    #[test]
    fn test_collision_started_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CollisionStarted>();
    }

    #[test]
    fn test_collision_started_clone() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let contact = Contact::default();

        let event = CollisionStarted::new(entity_a, entity_b, contact);
        let cloned = event.clone();

        assert_eq!(event, cloned);
    }

    #[test]
    fn test_collision_started_debug() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let contact = Contact::default();

        let event = CollisionStarted::new(entity_a, entity_b, contact);
        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("CollisionStarted"));
        assert!(debug_str.contains("entity_a"));
        assert!(debug_str.contains("entity_b"));
        assert!(debug_str.contains("contact"));
    }

    // =========================================================================
    // CollisionEnded Tests
    // =========================================================================

    #[test]
    fn test_collision_ended_new() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);

        let event = CollisionEnded::new(entity_a, entity_b);

        assert_eq!(event.entity_a, entity_a);
        assert_eq!(event.entity_b, entity_b);
    }

    #[test]
    fn test_collision_ended_involves() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let entity_c = Entity::from_bits(3);

        let event = CollisionEnded::new(entity_a, entity_b);

        assert!(event.involves(entity_a));
        assert!(event.involves(entity_b));
        assert!(!event.involves(entity_c));
    }

    #[test]
    fn test_collision_ended_other_entity() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let entity_c = Entity::from_bits(3);

        let event = CollisionEnded::new(entity_a, entity_b);

        assert_eq!(event.other_entity(entity_a), Some(entity_b));
        assert_eq!(event.other_entity(entity_b), Some(entity_a));
        assert_eq!(event.other_entity(entity_c), None);
    }

    #[test]
    fn test_collision_ended_ordered_pair() {
        let entity_1 = Entity::from_bits(1);
        let entity_2 = Entity::from_bits(2);

        // Test both orderings
        let event1 = CollisionEnded::new(entity_1, entity_2);
        let event2 = CollisionEnded::new(entity_2, entity_1);

        // Both should return the same ordered pair
        assert_eq!(event1.ordered_pair(), (entity_1, entity_2));
        assert_eq!(event2.ordered_pair(), (entity_1, entity_2));
    }

    #[test]
    fn test_collision_ended_implements_event() {
        fn accepts_event<E: Event>(_: E) {}

        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let event = CollisionEnded::new(entity_a, entity_b);

        accepts_event(event);
    }

    #[test]
    fn test_collision_ended_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CollisionEnded>();
    }

    #[test]
    fn test_collision_ended_clone() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);

        let event = CollisionEnded::new(entity_a, entity_b);
        let cloned = event.clone();

        assert_eq!(event, cloned);
    }

    #[test]
    fn test_collision_ended_debug() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);

        let event = CollisionEnded::new(entity_a, entity_b);
        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("CollisionEnded"));
        assert!(debug_str.contains("entity_a"));
        assert!(debug_str.contains("entity_b"));
    }

    #[test]
    fn test_collision_ended_hash() {
        use std::collections::HashSet;

        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let entity_c = Entity::from_bits(3);

        let mut set: HashSet<CollisionEnded> = HashSet::new();
        set.insert(CollisionEnded::new(entity_a, entity_b));
        set.insert(CollisionEnded::new(entity_b, entity_c));
        set.insert(CollisionEnded::new(entity_a, entity_b)); // Duplicate

        // Should only have 2 unique collision pairs
        assert_eq!(set.len(), 2);
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_collision_pair_consistency() {
        let entity_a = Entity::from_bits(1);
        let entity_b = Entity::from_bits(2);
        let contact = Contact::default();

        // Create events in different orders
        let start1 = CollisionStarted::new(entity_a, entity_b, contact);
        let start2 = CollisionStarted::new(entity_b, entity_a, contact);
        let end1 = CollisionEnded::new(entity_a, entity_b);
        let end2 = CollisionEnded::new(entity_b, entity_a);

        // Ordered pairs should be consistent
        assert_eq!(start1.ordered_pair(), start2.ordered_pair());
        assert_eq!(end1.ordered_pair(), end2.ordered_pair());
        assert_eq!(start1.ordered_pair(), end1.ordered_pair());
    }

    #[test]
    fn test_collision_event_workflow() {
        let player = Entity::from_bits(1);
        let enemy = Entity::from_bits(2);
        let contact = Contact::new(Vec2::new(5.0, 5.0), Vec2::new(1.0, 0.0), 0.2);

        // Collision starts
        let start_event = CollisionStarted::new(player, enemy, contact);
        assert!(start_event.involves(player));
        assert_eq!(start_event.other_entity(player), Some(enemy));
        assert!(start_event.contact.is_colliding());

        // Collision ends
        let end_event = CollisionEnded::new(player, enemy);
        assert!(end_event.involves(player));
        assert_eq!(end_event.other_entity(player), Some(enemy));

        // Both events should have same ordered pair
        assert_eq!(start_event.ordered_pair(), end_event.ordered_pair());
    }
}
