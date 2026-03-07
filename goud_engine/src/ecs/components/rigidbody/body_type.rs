//! [`RigidBodyType`] enum defining physics behavior classification.

// =============================================================================
// RigidBodyType Enum
// =============================================================================

/// Defines the physics behavior of a rigid body.
///
/// Different body types interact with the physics system in different ways.
/// This determines whether the body is affected by forces, can be moved by
/// the simulation, and how it collides with other bodies.
///
/// # FFI Safety
///
/// `#[repr(u8)]` ensures this enum has a stable ABI for FFI.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RigidBodyType {
    /// Dynamic bodies are fully simulated by the physics engine.
    ///
    /// - Affected by gravity and forces
    /// - Responds to collisions
    /// - Can be moved by constraints
    /// - Most expensive to simulate
    ///
    /// Use for: players, enemies, projectiles, movable objects
    Dynamic = 0,

    /// Kinematic bodies move via velocity but are not affected by forces.
    ///
    /// - NOT affected by gravity or forces
    /// - Does NOT respond to collisions (but affects other bodies)
    /// - Moved by setting velocity or position directly
    /// - Cheaper than dynamic
    ///
    /// Use for: moving platforms, elevators, doors, cutscene objects
    Kinematic = 1,

    /// Static bodies do not move and are not affected by forces.
    ///
    /// - Immovable
    /// - NOT affected by gravity or forces
    /// - Acts as obstacles for other bodies
    /// - Cheapest to simulate (often excluded from updates)
    ///
    /// Use for: walls, floors, terrain, static obstacles
    Static = 2,
}

impl Default for RigidBodyType {
    /// Defaults to Dynamic for most common use case.
    fn default() -> Self {
        Self::Dynamic
    }
}

impl RigidBodyType {
    /// Returns true if this body type is affected by gravity.
    #[inline]
    pub fn is_affected_by_gravity(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns true if this body type is affected by forces and impulses.
    #[inline]
    pub fn is_affected_by_forces(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns true if this body type can move.
    #[inline]
    pub fn can_move(self) -> bool {
        !matches!(self, RigidBodyType::Static)
    }

    /// Returns true if this body type responds to collisions.
    #[inline]
    pub fn responds_to_collisions(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns the name of this body type.
    pub fn name(self) -> &'static str {
        match self {
            RigidBodyType::Dynamic => "Dynamic",
            RigidBodyType::Kinematic => "Kinematic",
            RigidBodyType::Static => "Static",
        }
    }
}

impl std::fmt::Display for RigidBodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
