//! Collider component for physics collision detection.
//!
//! The [`Collider`] component defines the collision shape for an entity in the physics
//! simulation. It works in conjunction with [`RigidBody`](crate::ecs::components::RigidBody) to enable collision detection
//! and response.
//!
//! # Collision Shapes
//!
//! GoudEngine supports the following 2D collision shapes:
//!
//! - **Circle**: Defined by a radius, fastest collision detection
//! - **Box**: Axis-aligned or rotated rectangle (AABB or OBB)
//! - **Capsule**: Rounded rectangle, good for characters
//! - **Polygon**: Convex polygons for complex shapes
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::{Collider, ColliderShape};
//! use goud_engine::core::math::Vec2;
//!
//! // Create a circle collider (player, ball)
//! let ball = Collider::circle(0.5)
//!     .with_restitution(0.8) // Bouncy
//!     .with_friction(0.2);
//!
//! // Create a box collider (walls, platforms)
//! let wall = Collider::aabb(Vec2::new(10.0, 1.0))
//!     .with_friction(0.5);
//!
//! // Create a capsule collider (character controller)
//! let player = Collider::capsule(0.3, 1.0)
//!     .with_friction(0.0) // Frictionless movement
//!     .with_is_sensor(true); // Trigger only
//! ```
//!
//! # Collision Layers
//!
//! Colliders support layer-based filtering to control which objects can collide:
//!
//! ```
//! use goud_engine::ecs::components::Collider;
//!
//! // Player collides with enemies and walls
//! let player = Collider::circle(0.5)
//!     .with_layer(0b0001)
//!     .with_mask(0b0110);
//!
//! // Enemy collides with player and walls
//! let enemy = Collider::circle(0.4)
//!     .with_layer(0b0010)
//!     .with_mask(0b0101);
//! ```
//!
//! # Sensors (Triggers)
//!
//! Set `is_sensor` to true to create a trigger volume that detects collisions but
//! doesn't produce collision response:
//!
//! ```
//! use goud_engine::ecs::components::Collider;
//! use goud_engine::core::math::Vec2;
//!
//! // Pickup item trigger
//! let pickup = Collider::aabb(Vec2::new(1.0, 1.0))
//!     .with_is_sensor(true);
//! ```
//!
//! # Integration with RigidBody
//!
//! Colliders work with RigidBody components:
//!
//! - **Dynamic bodies**: Full collision detection and response
//! - **Kinematic bodies**: Collision detection only (no response)
//! - **Static bodies**: Acts as immovable obstacles
//!
//! # Thread Safety
//!
//! Collider is `Send + Sync` and can be safely used in parallel systems.

use crate::core::math::{Rect, Vec2};
use crate::ecs::Component;

// =============================================================================
// ColliderShape Enum
// =============================================================================

/// The geometric shape of a collider.
///
/// Each shape type has different performance characteristics and use cases:
///
/// - **Circle**: Fastest collision detection, best for balls, projectiles
/// - **Box**: Axis-aligned (AABB) or oriented (OBB), good for walls and platforms
/// - **Capsule**: Good for characters, combines efficiency with smooth edges
/// - **Polygon**: Most flexible but slowest, use sparingly for complex shapes
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape {
    /// Circle collider defined by radius.
    ///
    /// Center is at the entity's position. Fastest collision detection.
    Circle {
        /// Radius of the circle in world units
        radius: f32,
    },

    /// Axis-Aligned Bounding Box (AABB).
    ///
    /// Defined by half-extents (half-width, half-height) from the center.
    /// Fast collision detection, no rotation support.
    Aabb {
        /// Half-extents (half-width, half-height)
        half_extents: Vec2,
    },

    /// Oriented Bounding Box (OBB).
    ///
    /// Similar to AABB but can be rotated. Slightly slower than AABB.
    Obb {
        /// Half-extents (half-width, half-height)
        half_extents: Vec2,
    },

    /// Capsule collider (rounded rectangle).
    ///
    /// Defined by half-height and radius. Good for character controllers.
    /// The capsule extends vertically from the center.
    Capsule {
        /// Half-height of the capsule's cylindrical section
        half_height: f32,
        /// Radius of the capsule's rounded ends
        radius: f32,
    },

    /// Convex polygon collider.
    ///
    /// Vertices must be in counter-clockwise order and form a convex hull.
    /// Slowest collision detection, use sparingly.
    Polygon {
        /// Vertices in local space, counter-clockwise order
        vertices: Vec<Vec2>,
    },
}

impl ColliderShape {
    /// Returns the type name of this shape.
    pub fn type_name(&self) -> &'static str {
        match self {
            ColliderShape::Circle { .. } => "Circle",
            ColliderShape::Aabb { .. } => "AABB",
            ColliderShape::Obb { .. } => "OBB",
            ColliderShape::Capsule { .. } => "Capsule",
            ColliderShape::Polygon { .. } => "Polygon",
        }
    }

    /// Computes the axis-aligned bounding box (AABB) for this shape.
    ///
    /// Returns a rectangle in local space that fully contains the shape.
    /// For rotated shapes (OBB, Polygon), this is a conservative bound.
    pub fn compute_aabb(&self) -> Rect {
        match self {
            ColliderShape::Circle { radius } => {
                Rect::new(-radius, -radius, radius * 2.0, radius * 2.0)
            }
            ColliderShape::Aabb { half_extents } | ColliderShape::Obb { half_extents } => {
                Rect::new(
                    -half_extents.x,
                    -half_extents.y,
                    half_extents.x * 2.0,
                    half_extents.y * 2.0,
                )
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => {
                let width = radius * 2.0;
                let height = (half_height + radius) * 2.0;
                Rect::new(-radius, -(half_height + radius), width, height)
            }
            ColliderShape::Polygon { vertices } => {
                if vertices.is_empty() {
                    return Rect::unit();
                }

                let mut min_x = vertices[0].x;
                let mut min_y = vertices[0].y;
                let mut max_x = vertices[0].x;
                let mut max_y = vertices[0].y;

                for v in vertices.iter().skip(1) {
                    min_x = min_x.min(v.x);
                    min_y = min_y.min(v.y);
                    max_x = max_x.max(v.x);
                    max_y = max_y.max(v.y);
                }

                Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }

    /// Returns true if this shape is a circle.
    pub fn is_circle(&self) -> bool {
        matches!(self, ColliderShape::Circle { .. })
    }

    /// Returns true if this shape is an axis-aligned box (AABB).
    pub fn is_aabb(&self) -> bool {
        matches!(self, ColliderShape::Aabb { .. })
    }

    /// Returns true if this shape is an oriented box (OBB).
    pub fn is_obb(&self) -> bool {
        matches!(self, ColliderShape::Obb { .. })
    }

    /// Returns true if this shape is a capsule.
    pub fn is_capsule(&self) -> bool {
        matches!(self, ColliderShape::Capsule { .. })
    }

    /// Returns true if this shape is a polygon.
    pub fn is_polygon(&self) -> bool {
        matches!(self, ColliderShape::Polygon { .. })
    }

    /// Validates that the shape is well-formed.
    ///
    /// Returns `true` if:
    /// - Radii and extents are positive
    /// - Polygons have at least 3 vertices
    /// - Polygon vertices form a convex hull (not checked, assumed by user)
    pub fn is_valid(&self) -> bool {
        match self {
            ColliderShape::Circle { radius } => *radius > 0.0,
            ColliderShape::Aabb { half_extents } | ColliderShape::Obb { half_extents } => {
                half_extents.x > 0.0 && half_extents.y > 0.0
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => *half_height > 0.0 && *radius > 0.0,
            ColliderShape::Polygon { vertices } => vertices.len() >= 3,
        }
    }
}

impl Default for ColliderShape {
    /// Returns a unit circle (radius 1.0) as the default shape.
    fn default() -> Self {
        ColliderShape::Circle { radius: 1.0 }
    }
}

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
    pub fn aabb(half_extents: Vec2) -> Self {
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
    pub fn obb(half_extents: Vec2) -> Self {
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
    pub fn polygon(vertices: Vec<Vec2>) -> Self {
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

// =============================================================================
// AABB Utilities
// =============================================================================

/// Utilities for Axis-Aligned Bounding Box (AABB) calculations and operations.
///
/// These functions are used for broad-phase collision detection, spatial queries,
/// and efficient geometric tests.
pub mod aabb {
    use crate::core::math::{Rect, Vec2};
    use crate::ecs::components::{ColliderShape, Transform2D};

    /// Computes the world-space AABB for a collider shape with a transform.
    ///
    /// This takes a collider's local AABB and transforms it to world space
    /// using the entity's Transform2D. The result is always axis-aligned
    /// even if the shape is rotated.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::{Collider, Transform2D, collider};
    /// use goud_engine::core::math::Vec2;
    ///
    /// let collider = Collider::circle(1.0);
    /// let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));
    ///
    /// let world_aabb = collider::aabb::compute_world_aabb(collider.shape(), &transform);
    /// assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
    /// ```
    pub fn compute_world_aabb(shape: &ColliderShape, transform: &Transform2D) -> Rect {
        let local_aabb = shape.compute_aabb();

        // For circles and AABBs without rotation, we can optimize
        if matches!(shape, ColliderShape::Circle { .. })
            || (matches!(shape, ColliderShape::Aabb { .. })
                && transform.rotation.abs() < f32::EPSILON)
        {
            // Simple translation + scale
            let half_size = local_aabb.size() * 0.5;
            let scaled_half_size = Vec2::new(
                half_size.x * transform.scale.x.abs(),
                half_size.y * transform.scale.y.abs(),
            );
            let center = transform.position;
            return Rect::from_min_max(center - scaled_half_size, center + scaled_half_size);
        }

        // For rotated shapes, transform all corners and compute bounding box
        let corners = [
            Vec2::new(local_aabb.x, local_aabb.y),
            Vec2::new(local_aabb.x + local_aabb.width, local_aabb.y),
            Vec2::new(
                local_aabb.x + local_aabb.width,
                local_aabb.y + local_aabb.height,
            ),
            Vec2::new(local_aabb.x, local_aabb.y + local_aabb.height),
        ];

        let matrix = transform.matrix();
        let transformed_corners: Vec<Vec2> = corners
            .iter()
            .map(|&corner| matrix.transform_point(corner))
            .collect();

        // Find min/max of transformed corners
        let mut min_x = transformed_corners[0].x;
        let mut min_y = transformed_corners[0].y;
        let mut max_x = transformed_corners[0].x;
        let mut max_y = transformed_corners[0].y;

        for corner in &transformed_corners[1..] {
            min_x = min_x.min(corner.x);
            min_y = min_y.min(corner.y);
            max_x = max_x.max(corner.x);
            max_y = max_y.max(corner.y);
        }

        Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Tests if two AABBs overlap.
    ///
    /// Returns true if the rectangles intersect or touch.
    #[inline]
    pub fn overlaps(a: &Rect, b: &Rect) -> bool {
        a.intersects(b)
    }

    /// Computes the intersection of two AABBs.
    ///
    /// Returns Some(Rect) with the overlapping region, or None if they don't overlap.
    #[inline]
    pub fn intersection(a: &Rect, b: &Rect) -> Option<Rect> {
        a.intersection(b)
    }

    /// Expands an AABB by a margin on all sides.
    ///
    /// Useful for creating query regions or tolerance zones.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::collider;
    /// use goud_engine::core::math::Rect;
    ///
    /// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
    /// let expanded = collider::aabb::expand(&aabb, 1.0);
    /// assert_eq!(expanded.width, 12.0);
    /// assert_eq!(expanded.height, 12.0);
    /// ```
    pub fn expand(aabb: &Rect, margin: f32) -> Rect {
        Rect::new(
            aabb.x - margin,
            aabb.y - margin,
            aabb.width + margin * 2.0,
            aabb.height + margin * 2.0,
        )
    }

    /// Merges two AABBs into a single AABB that contains both.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::collider;
    /// use goud_engine::core::math::Rect;
    ///
    /// let a = Rect::new(0.0, 0.0, 5.0, 5.0);
    /// let b = Rect::new(3.0, 3.0, 5.0, 5.0);
    /// let merged = collider::aabb::merge(&a, &b);
    /// assert_eq!(merged.x, 0.0);
    /// assert_eq!(merged.y, 0.0);
    /// assert_eq!(merged.width, 8.0);
    /// assert_eq!(merged.height, 8.0);
    /// ```
    pub fn merge(a: &Rect, b: &Rect) -> Rect {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = (a.x + a.width).max(b.x + b.width);
        let max_y = (a.y + a.height).max(b.y + b.height);
        Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Tests if a point is inside an AABB.
    #[inline]
    pub fn contains_point(aabb: &Rect, point: Vec2) -> bool {
        aabb.contains(point)
    }

    /// Performs a raycast against an AABB.
    ///
    /// Returns Some(t) with the intersection parameter [0, 1] if the ray hits,
    /// or None if it misses. The intersection point is: ray_origin + ray_direction * t.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::collider;
    /// use goud_engine::core::math::{Rect, Vec2};
    ///
    /// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
    /// let ray_origin = Vec2::new(-5.0, 5.0);
    /// let ray_direction = Vec2::new(1.0, 0.0);
    ///
    /// let hit = collider::aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
    /// assert!(hit.is_some());
    /// ```
    pub fn raycast(
        aabb: &Rect,
        ray_origin: Vec2,
        ray_direction: Vec2,
        max_distance: f32,
    ) -> Option<f32> {
        // Slab method for AABB raycast
        let inv_dir = Vec2::new(
            if ray_direction.x.abs() < f32::EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray_direction.x
            },
            if ray_direction.y.abs() < f32::EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray_direction.y
            },
        );

        let min = aabb.min();
        let max = aabb.max();

        let t1 = (min.x - ray_origin.x) * inv_dir.x;
        let t2 = (max.x - ray_origin.x) * inv_dir.x;
        let t3 = (min.y - ray_origin.y) * inv_dir.y;
        let t4 = (max.y - ray_origin.y) * inv_dir.y;

        let tmin = t1.min(t2).max(t3.min(t4)).max(0.0);
        let tmax = t1.max(t2).min(t3.max(t4)).min(max_distance);

        if tmax >= tmin && tmin <= max_distance {
            Some(tmin)
        } else {
            None
        }
    }

    /// Computes the closest point on an AABB to a given point.
    ///
    /// If the point is inside the AABB, returns the point itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::collider;
    /// use goud_engine::core::math::{Rect, Vec2};
    ///
    /// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
    /// let point = Vec2::new(-5.0, 5.0);
    ///
    /// let closest = collider::aabb::closest_point(&aabb, point);
    /// assert_eq!(closest, Vec2::new(0.0, 5.0));
    /// ```
    pub fn closest_point(aabb: &Rect, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(aabb.x, aabb.x + aabb.width),
            point.y.clamp(aabb.y, aabb.y + aabb.height),
        )
    }

    /// Computes the squared distance from a point to the surface of an AABB.
    ///
    /// Returns 0.0 if the point is inside the AABB.
    pub fn distance_squared_to_point(aabb: &Rect, point: Vec2) -> f32 {
        let closest = closest_point(aabb, point);
        let dx = point.x - closest.x;
        let dy = point.y - closest.y;
        dx * dx + dy * dy
    }

    /// Computes the area of an AABB.
    #[inline]
    pub fn area(aabb: &Rect) -> f32 {
        aabb.area()
    }

    /// Computes the perimeter of an AABB.
    pub fn perimeter(aabb: &Rect) -> f32 {
        2.0 * (aabb.width + aabb.height)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ColliderShape Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_collider_shape_type_names() {
        assert_eq!(ColliderShape::Circle { radius: 1.0 }.type_name(), "Circle");
        assert_eq!(
            ColliderShape::Aabb {
                half_extents: Vec2::one()
            }
            .type_name(),
            "AABB"
        );
        assert_eq!(
            ColliderShape::Obb {
                half_extents: Vec2::one()
            }
            .type_name(),
            "OBB"
        );
        assert_eq!(
            ColliderShape::Capsule {
                half_height: 1.0,
                radius: 0.5
            }
            .type_name(),
            "Capsule"
        );
        assert_eq!(
            ColliderShape::Polygon {
                vertices: vec![Vec2::zero(), Vec2::unit_x(), Vec2::unit_y()]
            }
            .type_name(),
            "Polygon"
        );
    }

    #[test]
    fn test_collider_shape_predicates() {
        let circle = ColliderShape::Circle { radius: 1.0 };
        assert!(circle.is_circle());
        assert!(!circle.is_aabb());
        assert!(!circle.is_obb());
        assert!(!circle.is_capsule());
        assert!(!circle.is_polygon());

        let aabb = ColliderShape::Aabb {
            half_extents: Vec2::one(),
        };
        assert!(!aabb.is_circle());
        assert!(aabb.is_aabb());
        assert!(!aabb.is_obb());
    }

    #[test]
    fn test_collider_shape_is_valid() {
        // Valid shapes
        assert!(ColliderShape::Circle { radius: 1.0 }.is_valid());
        assert!(ColliderShape::Aabb {
            half_extents: Vec2::one()
        }
        .is_valid());
        assert!(ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5
        }
        .is_valid());
        assert!(ColliderShape::Polygon {
            vertices: vec![Vec2::zero(), Vec2::unit_x(), Vec2::unit_y()]
        }
        .is_valid());

        // Invalid shapes
        assert!(!ColliderShape::Circle { radius: 0.0 }.is_valid());
        assert!(!ColliderShape::Circle { radius: -1.0 }.is_valid());
        assert!(!ColliderShape::Aabb {
            half_extents: Vec2::zero()
        }
        .is_valid());
        assert!(!ColliderShape::Polygon {
            vertices: vec![Vec2::zero(), Vec2::unit_x()]
        }
        .is_valid());
    }

    #[test]
    fn test_collider_shape_compute_aabb_circle() {
        let shape = ColliderShape::Circle { radius: 2.0 };
        let aabb = shape.compute_aabb();
        assert_eq!(aabb.x, -2.0);
        assert_eq!(aabb.y, -2.0);
        assert_eq!(aabb.width, 4.0);
        assert_eq!(aabb.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_box() {
        let shape = ColliderShape::Aabb {
            half_extents: Vec2::new(3.0, 2.0),
        };
        let aabb = shape.compute_aabb();
        assert_eq!(aabb.x, -3.0);
        assert_eq!(aabb.y, -2.0);
        assert_eq!(aabb.width, 6.0);
        assert_eq!(aabb.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_capsule() {
        let shape = ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5,
        };
        let aabb = shape.compute_aabb();
        assert_eq!(aabb.x, -0.5);
        assert_eq!(aabb.y, -1.5);
        assert_eq!(aabb.width, 1.0);
        assert_eq!(aabb.height, 3.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_polygon() {
        let shape = ColliderShape::Polygon {
            vertices: vec![
                Vec2::new(-1.0, -1.0),
                Vec2::new(2.0, 0.0),
                Vec2::new(0.0, 3.0),
            ],
        };
        let aabb = shape.compute_aabb();
        assert_eq!(aabb.x, -1.0);
        assert_eq!(aabb.y, -1.0);
        assert_eq!(aabb.width, 3.0);
        assert_eq!(aabb.height, 4.0);
    }

    #[test]
    fn test_collider_shape_compute_aabb_empty_polygon() {
        let shape = ColliderShape::Polygon { vertices: vec![] };
        let aabb = shape.compute_aabb();
        // Should return unit rect for empty polygon
        assert_eq!(aabb, Rect::unit());
    }

    #[test]
    fn test_collider_shape_default() {
        let shape = ColliderShape::default();
        assert!(shape.is_circle());
        if let ColliderShape::Circle { radius } = shape {
            assert_eq!(radius, 1.0);
        }
    }

    // -------------------------------------------------------------------------
    // Collider Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_collider_circle() {
        let collider = Collider::circle(1.5);
        assert!(collider.shape().is_circle());
        assert_eq!(collider.restitution(), 0.3);
        assert_eq!(collider.friction(), 0.5);
        assert!(!collider.is_sensor());
        assert!(collider.is_enabled());
    }

    #[test]
    fn test_collider_aabb() {
        let collider = Collider::aabb(Vec2::new(2.0, 3.0));
        assert!(collider.shape().is_aabb());
    }

    #[test]
    fn test_collider_obb() {
        let collider = Collider::obb(Vec2::new(2.0, 3.0));
        assert!(collider.shape().is_obb());
    }

    #[test]
    fn test_collider_capsule() {
        let collider = Collider::capsule(1.0, 0.5);
        assert!(collider.shape().is_capsule());
    }

    #[test]
    fn test_collider_polygon() {
        let collider = Collider::polygon(vec![Vec2::zero(), Vec2::unit_x(), Vec2::new(0.5, 1.0)]);
        assert!(collider.shape().is_polygon());
    }

    #[test]
    #[should_panic(expected = "must have at least 3 vertices")]
    fn test_collider_polygon_panics_with_too_few_vertices() {
        Collider::polygon(vec![Vec2::zero(), Vec2::unit_x()]);
    }

    #[test]
    fn test_collider_builder_pattern() {
        let collider = Collider::circle(1.0)
            .with_restitution(0.8)
            .with_friction(0.1)
            .with_density(2.5)
            .with_layer(0b0010)
            .with_mask(0b1100)
            .with_is_sensor(true)
            .with_enabled(false);

        assert_eq!(collider.restitution(), 0.8);
        assert_eq!(collider.friction(), 0.1);
        assert_eq!(collider.density(), Some(2.5));
        assert_eq!(collider.layer(), 0b0010);
        assert_eq!(collider.mask(), 0b1100);
        assert!(collider.is_sensor());
        assert!(!collider.is_enabled());
    }

    #[test]
    fn test_collider_restitution_clamping() {
        let collider = Collider::circle(1.0).with_restitution(1.5);
        assert_eq!(collider.restitution(), 1.0);

        let collider = Collider::circle(1.0).with_restitution(-0.5);
        assert_eq!(collider.restitution(), 0.0);
    }

    #[test]
    fn test_collider_friction_clamping() {
        let collider = Collider::circle(1.0).with_friction(-1.0);
        assert_eq!(collider.friction(), 0.0);

        // Friction can exceed 1.0
        let collider = Collider::circle(1.0).with_friction(2.0);
        assert_eq!(collider.friction(), 2.0);
    }

    #[test]
    fn test_collider_density_clamping() {
        let collider = Collider::circle(1.0).with_density(-1.0);
        assert_eq!(collider.density(), Some(0.0));
    }

    #[test]
    fn test_collider_mutators() {
        let mut collider = Collider::circle(1.0);

        collider.set_restitution(0.9);
        assert_eq!(collider.restitution(), 0.9);

        collider.set_friction(0.2);
        assert_eq!(collider.friction(), 0.2);

        collider.set_density(Some(3.0));
        assert_eq!(collider.density(), Some(3.0));

        collider.set_layer(0b0100);
        assert_eq!(collider.layer(), 0b0100);

        collider.set_mask(0b1000);
        assert_eq!(collider.mask(), 0b1000);

        collider.set_is_sensor(true);
        assert!(collider.is_sensor());

        collider.set_enabled(false);
        assert!(!collider.is_enabled());
    }

    #[test]
    fn test_collider_can_collide_with() {
        let collider_a = Collider::circle(1.0).with_layer(0b0001).with_mask(0b0010);
        let collider_b = Collider::circle(1.0).with_layer(0b0010).with_mask(0b0001);
        let collider_c = Collider::circle(1.0).with_layer(0b0100).with_mask(0b1000);

        // A and B should collide (mutual layer/mask match)
        assert!(collider_a.can_collide_with(&collider_b));
        assert!(collider_b.can_collide_with(&collider_a));

        // A and C should not collide (no layer/mask overlap)
        assert!(!collider_a.can_collide_with(&collider_c));
        assert!(!collider_c.can_collide_with(&collider_a));

        // B and C should not collide
        assert!(!collider_b.can_collide_with(&collider_c));
    }

    #[test]
    fn test_collider_compute_aabb() {
        let collider = Collider::circle(2.0);
        let aabb = collider.compute_aabb();
        assert_eq!(aabb.x, -2.0);
        assert_eq!(aabb.y, -2.0);
        assert_eq!(aabb.width, 4.0);
        assert_eq!(aabb.height, 4.0);
    }

    #[test]
    fn test_collider_set_shape() {
        let mut collider = Collider::circle(1.0);
        assert!(collider.shape().is_circle());

        collider.set_shape(ColliderShape::Aabb {
            half_extents: Vec2::one(),
        });
        assert!(collider.shape().is_aabb());
    }

    #[test]
    fn test_collider_default() {
        let collider = Collider::default();
        assert!(collider.shape().is_circle());
        assert_eq!(collider.restitution(), 0.3);
        assert_eq!(collider.friction(), 0.5);
        assert!(!collider.is_sensor());
        assert!(collider.is_enabled());
    }

    #[test]
    fn test_collider_display() {
        let collider = Collider::circle(1.0);
        let display = format!("{collider}");
        assert!(display.contains("Circle"));
        assert!(display.contains("restitution"));
        assert!(display.contains("friction"));

        let sensor = Collider::circle(1.0).with_is_sensor(true);
        let display = format!("{sensor}");
        assert!(display.contains("sensor"));

        let disabled = Collider::circle(1.0).with_enabled(false);
        let display = format!("{disabled}");
        assert!(display.contains("disabled"));
    }

    #[test]
    fn test_collider_is_component() {
        fn assert_component<T: Component>() {}
        assert_component::<Collider>();
    }

    #[test]
    fn test_collider_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Collider>();
    }

    #[test]
    fn test_collider_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Collider>();
    }

    #[test]
    fn test_collider_clone() {
        let collider = Collider::circle(1.0).with_restitution(0.8);
        let cloned = collider.clone();
        assert_eq!(collider, cloned);
    }

    #[test]
    fn test_collider_shape_clone() {
        let shape = ColliderShape::Circle { radius: 1.0 };
        let cloned = shape.clone();
        assert_eq!(shape, cloned);
    }

    // -------------------------------------------------------------------------
    // AABB Utilities Tests
    // -------------------------------------------------------------------------

    use super::aabb;
    use crate::ecs::components::Transform2D;

    #[test]
    fn test_aabb_compute_world_aabb_circle_no_rotation() {
        let shape = ColliderShape::Circle { radius: 2.0 };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
        assert_eq!(world_aabb.width, 4.0);
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_circle_with_scale() {
        let shape = ColliderShape::Circle { radius: 1.0 };
        let mut transform = Transform2D::from_position(Vec2::new(5.0, 5.0));
        transform.set_scale_uniform(2.0);

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(5.0, 5.0));
        assert_eq!(world_aabb.width, 4.0); // 2 * radius * scale
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_box_no_rotation() {
        let shape = ColliderShape::Aabb {
            half_extents: Vec2::new(3.0, 2.0),
        };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
        assert_eq!(world_aabb.width, 6.0);
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_box_with_rotation() {
        let shape = ColliderShape::Obb {
            half_extents: Vec2::new(2.0, 1.0),
        };
        let mut transform = Transform2D::from_position(Vec2::new(0.0, 0.0));
        transform.set_rotation(std::f32::consts::PI / 4.0); // 45 degrees

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);

        // At 45 degrees, a 4x2 box should have AABB approximately sqrt(20) ≈ 4.47 on each side
        // But corners at (±2, ±1) rotated 45° gives max extent ≈ 2.12
        assert!(world_aabb.width > 4.0 && world_aabb.width < 4.5);
        assert!(world_aabb.height > 4.0 && world_aabb.height < 4.5);
        assert_eq!(world_aabb.center(), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_aabb_compute_world_aabb_capsule() {
        let shape = ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5,
        };
        let transform = Transform2D::from_position(Vec2::new(5.0, 10.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(5.0, 10.0));
        assert_eq!(world_aabb.width, 1.0); // 2 * radius
        assert_eq!(world_aabb.height, 3.0); // 2 * (half_height + radius)
    }

    #[test]
    fn test_aabb_overlaps() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        let c = Rect::new(20.0, 20.0, 10.0, 10.0);

        assert!(aabb::overlaps(&a, &b));
        assert!(aabb::overlaps(&b, &a));
        assert!(!aabb::overlaps(&a, &c));
    }

    #[test]
    fn test_aabb_intersection() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);

        let intersection = aabb::intersection(&a, &b);
        assert!(intersection.is_some());
        let rect = intersection.unwrap();
        assert_eq!(rect.x, 5.0);
        assert_eq!(rect.y, 5.0);
        assert_eq!(rect.width, 5.0);
        assert_eq!(rect.height, 5.0);
    }

    #[test]
    fn test_aabb_intersection_none() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(20.0, 20.0, 10.0, 10.0);

        assert!(aabb::intersection(&a, &b).is_none());
    }

    #[test]
    fn test_aabb_expand() {
        let aabb = Rect::new(5.0, 5.0, 10.0, 10.0);
        let expanded = aabb::expand(&aabb, 2.0);

        assert_eq!(expanded.x, 3.0);
        assert_eq!(expanded.y, 3.0);
        assert_eq!(expanded.width, 14.0);
        assert_eq!(expanded.height, 14.0);
    }

    #[test]
    fn test_aabb_expand_negative_margin() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let shrunk = aabb::expand(&aabb, -1.0);

        assert_eq!(shrunk.x, 1.0);
        assert_eq!(shrunk.y, 1.0);
        assert_eq!(shrunk.width, 8.0);
        assert_eq!(shrunk.height, 8.0);
    }

    #[test]
    fn test_aabb_merge() {
        let a = Rect::new(0.0, 0.0, 5.0, 5.0);
        let b = Rect::new(3.0, 3.0, 5.0, 5.0);
        let merged = aabb::merge(&a, &b);

        assert_eq!(merged.x, 0.0);
        assert_eq!(merged.y, 0.0);
        assert_eq!(merged.width, 8.0);
        assert_eq!(merged.height, 8.0);
    }

    #[test]
    fn test_aabb_merge_disjoint() {
        let a = Rect::new(0.0, 0.0, 5.0, 5.0);
        let b = Rect::new(10.0, 10.0, 5.0, 5.0);
        let merged = aabb::merge(&a, &b);

        assert_eq!(merged.x, 0.0);
        assert_eq!(merged.y, 0.0);
        assert_eq!(merged.width, 15.0);
        assert_eq!(merged.height, 15.0);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);

        assert!(aabb::contains_point(&aabb, Vec2::new(5.0, 5.0)));
        assert!(aabb::contains_point(&aabb, Vec2::new(0.0, 0.0)));
        assert!(!aabb::contains_point(&aabb, Vec2::new(-1.0, 5.0)));
        assert!(!aabb::contains_point(&aabb, Vec2::new(5.0, 11.0)));
    }

    #[test]
    fn test_aabb_raycast_hit() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 5.0).abs() < 0.001); // Should hit at t=5
    }

    #[test]
    fn test_aabb_raycast_miss() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, 15.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_aabb_raycast_from_inside() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(5.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap(), 0.0); // Ray starts inside
    }

    #[test]
    fn test_aabb_raycast_max_distance() {
        let aabb = Rect::new(100.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(0.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        // Should not hit because max_distance is too short
        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 50.0);
        assert!(hit.is_none());

        // Should hit with longer max_distance
        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 200.0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_aabb_closest_point_outside() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-5.0, 5.0);

        let closest = aabb::closest_point(&aabb, point);
        assert_eq!(closest, Vec2::new(0.0, 5.0));
    }

    #[test]
    fn test_aabb_closest_point_inside() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(5.0, 5.0);

        let closest = aabb::closest_point(&aabb, point);
        assert_eq!(closest, point); // Point is inside, returns itself
    }

    #[test]
    fn test_aabb_closest_point_corner() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-5.0, -5.0);

        let closest = aabb::closest_point(&aabb, point);
        assert_eq!(closest, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_aabb_distance_squared_to_point_outside() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-3.0, 0.0);

        let dist_sq = aabb::distance_squared_to_point(&aabb, point);
        assert_eq!(dist_sq, 9.0); // Distance is 3.0, squared is 9.0
    }

    #[test]
    fn test_aabb_distance_squared_to_point_inside() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(5.0, 5.0);

        let dist_sq = aabb::distance_squared_to_point(&aabb, point);
        assert_eq!(dist_sq, 0.0);
    }

    #[test]
    fn test_aabb_area() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(aabb::area(&aabb), 50.0);
    }

    #[test]
    fn test_aabb_perimeter() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(aabb::perimeter(&aabb), 30.0); // 2 * (10 + 5)
    }

    #[test]
    fn test_aabb_compute_world_aabb_polygon() {
        let vertices = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, 3.0),
        ];
        let shape = ColliderShape::Polygon { vertices };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);

        // Local AABB is (-1, -1) to (2, 3), size (3, 4)
        // Translated to (10, 20) should give center at (10.5, 22)
        let expected_min = Vec2::new(9.0, 19.0);
        let expected_max = Vec2::new(12.0, 23.0);

        assert!((world_aabb.x - expected_min.x).abs() < 0.001);
        assert!((world_aabb.y - expected_min.y).abs() < 0.001);
        assert!((world_aabb.max().x - expected_max.x).abs() < 0.001);
        assert!((world_aabb.max().y - expected_max.y).abs() < 0.001);
    }

    #[test]
    fn test_aabb_raycast_diagonal() {
        let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, -5.0);
        let ray_direction = Vec2::new(1.0, 1.0).normalize();

        let hit = aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());

        // Hit point should be near (0, 0)
        let t = hit.unwrap();
        let hit_point = ray_origin + ray_direction * t;
        assert!((hit_point.x - 0.0).abs() < 0.1);
        assert!((hit_point.y - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_aabb_expand_zero_margin() {
        let aabb = Rect::new(5.0, 5.0, 10.0, 10.0);
        let expanded = aabb::expand(&aabb, 0.0);

        assert_eq!(expanded, aabb);
    }
}
