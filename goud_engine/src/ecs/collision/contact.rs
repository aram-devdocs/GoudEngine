//! Contact information returned by collision detection algorithms.

use crate::core::math::Vec2;

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
