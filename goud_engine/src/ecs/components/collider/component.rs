//! The [`Collider`] component definition and builder API.

use crate::core::math::Rect;
use crate::ecs::Component;

use super::shape::ColliderShape;

// =============================================================================
// Collider Component
// =============================================================================

/// Collider component for physics collision detection.
///
/// Defines the collision shape, material properties (friction, restitution),
/// and filtering (layers, masks) for an entity.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::Collider;
/// use goud_engine::core::math::Vec2;
///
/// // Circle collider
/// let ball = Collider::circle(0.5);
///
/// // Box collider
/// let wall = Collider::aabb(Vec2::new(5.0, 1.0));
///
/// // Capsule collider
/// let player = Collider::capsule(0.3, 1.0);
///
/// // Sensor (trigger)
/// let trigger = Collider::circle(2.0).with_is_sensor(true);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Collider {
    /// The geometric shape of the collider
    shape: ColliderShape,

    /// Coefficient of restitution (bounciness)
    ///
    /// 0.0 = no bounce, 1.0 = perfect bounce
    restitution: f32,

    /// Coefficient of friction
    ///
    /// 0.0 = frictionless, 1.0 = high friction
    friction: f32,

    /// Density for mass calculation (kg/m²)
    ///
    /// If set, the mass is automatically calculated from shape area × density.
    /// If None, the [`RigidBody`](crate::ecs::components::RigidBody)'s mass is used directly.
    density: Option<f32>,

    /// Collision layer bitmask (which layer this collider is on)
    ///
    /// Use powers of 2 for layer values: 0b0001, 0b0010, 0b0100, etc.
    layer: u32,

    /// Collision mask bitmask (which layers this collider can collide with)
    ///
    /// Collision occurs if: (layer_a & mask_b) != 0 && (layer_b & mask_a) != 0
    mask: u32,

    /// If true, this collider is a sensor (trigger) that detects collisions
    /// but doesn't produce physical response
    is_sensor: bool,

    /// If true, this collider is enabled and participates in collision detection
    enabled: bool,
}

impl Collider {
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /// Creates a new circle collider with the given radius.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::Collider;
    ///
    /// let ball = Collider::circle(0.5);
    /// ```
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Circle { radius },
            restitution: 0.3,
            friction: 0.5,
            density: None,
            layer: 0xFFFFFFFF, // Default: all layers
            mask: 0xFFFFFFFF,  // Default: collide with all layers
            is_sensor: false,
            enabled: true,
        }
    }

    /// Creates a new axis-aligned box collider with the given half-extents.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::Collider;
    /// use goud_engine::core::math::Vec2;
    ///
    /// // 10x2 box (half-extents are half the size)
    /// let wall = Collider::aabb(Vec2::new(5.0, 1.0));
    /// ```
    pub fn aabb(half_extents: crate::core::math::Vec2) -> Self {
        Self {
            shape: ColliderShape::Aabb { half_extents },
            restitution: 0.3,
            friction: 0.5,
            density: None,
            layer: 0xFFFFFFFF,
            mask: 0xFFFFFFFF,
            is_sensor: false,
            enabled: true,
        }
    }

    /// Creates a new oriented box collider with the given half-extents.
    ///
    /// Similar to `aabb()` but supports rotation via the entity's Transform2D.
    pub fn obb(half_extents: crate::core::math::Vec2) -> Self {
        Self {
            shape: ColliderShape::Obb { half_extents },
            restitution: 0.3,
            friction: 0.5,
            density: None,
            layer: 0xFFFFFFFF,
            mask: 0xFFFFFFFF,
            is_sensor: false,
            enabled: true,
        }
    }

    /// Creates a new capsule collider with the given half-height and radius.
    ///
    /// Good for character controllers with smooth movement.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::Collider;
    ///
    /// // Capsule with 0.6 radius and 2.0 total height
    /// let player = Collider::capsule(0.3, 1.0);
    /// ```
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            restitution: 0.3,
            friction: 0.5,
            density: None,
            layer: 0xFFFFFFFF,
            mask: 0xFFFFFFFF,
            is_sensor: false,
            enabled: true,
        }
    }

    /// Creates a new convex polygon collider with the given vertices.
    ///
    /// Vertices must be in counter-clockwise order and form a convex hull.
    ///
    /// # Panics
    ///
    /// Panics if fewer than 3 vertices are provided.
    pub fn polygon(vertices: Vec<crate::core::math::Vec2>) -> Self {
        assert!(
            vertices.len() >= 3,
            "Polygon collider must have at least 3 vertices"
        );
        Self {
            shape: ColliderShape::Polygon { vertices },
            restitution: 0.3,
            friction: 0.5,
            density: None,
            layer: 0xFFFFFFFF,
            mask: 0xFFFFFFFF,
            is_sensor: false,
            enabled: true,
        }
    }

    // -------------------------------------------------------------------------
    // Builder Pattern
    // -------------------------------------------------------------------------

    /// Sets the restitution (bounciness) coefficient.
    ///
    /// 0.0 = no bounce, 1.0 = perfect bounce.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Sets the friction coefficient.
    ///
    /// 0.0 = frictionless, 1.0 = high friction.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0);
        self
    }

    /// Sets the density for automatic mass calculation.
    ///
    /// Mass will be calculated as: area × density
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = Some(density.max(0.0));
        self
    }

    /// Sets the collision layer bitmask.
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the collision mask bitmask.
    pub fn with_mask(mut self, mask: u32) -> Self {
        self.mask = mask;
        self
    }

    /// Sets whether this collider is a sensor (trigger).
    pub fn with_is_sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    /// Sets whether this collider is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the collision shape.
    pub fn shape(&self) -> &ColliderShape {
        &self.shape
    }

    /// Returns the restitution coefficient.
    pub fn restitution(&self) -> f32 {
        self.restitution
    }

    /// Returns the friction coefficient.
    pub fn friction(&self) -> f32 {
        self.friction
    }

    /// Returns the density, if set.
    pub fn density(&self) -> Option<f32> {
        self.density
    }

    /// Returns the collision layer.
    pub fn layer(&self) -> u32 {
        self.layer
    }

    /// Returns the collision mask.
    pub fn mask(&self) -> u32 {
        self.mask
    }

    /// Returns true if this collider is a sensor (trigger).
    pub fn is_sensor(&self) -> bool {
        self.is_sensor
    }

    /// Returns true if this collider is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Computes the axis-aligned bounding box (AABB) for this collider.
    pub fn compute_aabb(&self) -> Rect {
        self.shape.compute_aabb()
    }

    /// Checks if this collider can collide with another based on layer filtering.
    ///
    /// Returns true if: (self.layer & other.mask) != 0 && (other.layer & self.mask) != 0
    pub fn can_collide_with(&self, other: &Collider) -> bool {
        (self.layer & other.mask) != 0 && (other.layer & self.mask) != 0
    }

    // -------------------------------------------------------------------------
    // Mutators
    // -------------------------------------------------------------------------

    /// Sets the restitution coefficient.
    pub fn set_restitution(&mut self, restitution: f32) {
        self.restitution = restitution.clamp(0.0, 1.0);
    }

    /// Sets the friction coefficient.
    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction.max(0.0);
    }

    /// Sets the density for automatic mass calculation.
    pub fn set_density(&mut self, density: Option<f32>) {
        self.density = density.map(|d| d.max(0.0));
    }

    /// Sets the collision layer.
    pub fn set_layer(&mut self, layer: u32) {
        self.layer = layer;
    }

    /// Sets the collision mask.
    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    /// Sets whether this collider is a sensor.
    pub fn set_is_sensor(&mut self, is_sensor: bool) {
        self.is_sensor = is_sensor;
    }

    /// Sets whether this collider is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Replaces the collision shape.
    pub fn set_shape(&mut self, shape: ColliderShape) {
        self.shape = shape;
    }
}

impl Default for Collider {
    /// Returns a default circle collider with radius 0.5.
    fn default() -> Self {
        Self::circle(0.5)
    }
}

impl std::fmt::Display for Collider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Collider({}, restitution: {:.2}, friction: {:.2}{}{})",
            self.shape.type_name(),
            self.restitution,
            self.friction,
            if self.is_sensor { ", sensor" } else { "" },
            if !self.enabled { ", disabled" } else { "" }
        )
    }
}

// Implement Component trait
impl Component for Collider {}
