//! Core `System` trait, `BoxedSystem` wrapper, and `IntoSystem` conversion trait.

use crate::ecs::query::Access;
use crate::ecs::World;

use super::system_id::SystemId;

// =============================================================================
// System Trait
// =============================================================================

/// A trait for types that can be run as systems on a [`World`].
///
/// Systems are the behavior layer of the ECS. They operate on entities and
/// components, implementing game logic, physics, rendering, and more.
///
/// # Requirements
///
/// Systems must be `Send` to enable future parallel execution. This means
/// all data accessed by the system must be thread-safe.
///
/// # Implementing System
///
/// For simple systems, implement this trait directly:
///
/// ```
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::system::System;
///
/// struct CountEntities {
///     count: u32,
/// }
///
/// impl System for CountEntities {
///     fn name(&self) -> &'static str {
///         "CountEntities"
///     }
///
///     fn run(&mut self, world: &mut World) {
///         self.count = world.entity_count() as u32;
///     }
/// }
/// ```
///
/// # Access Tracking
///
/// Override `component_access()` to declare what components your system
/// reads and writes. This enables the scheduler to detect conflicts and
/// run non-conflicting systems in parallel.
///
/// ```
/// use goud_engine::ecs::{World, Component, ComponentId};
/// use goud_engine::ecs::system::System;
/// use goud_engine::ecs::query::Access;
///
/// #[derive(Clone, Copy)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// #[derive(Clone, Copy)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// struct MovementSystem;
///
/// impl System for MovementSystem {
///     fn name(&self) -> &'static str {
///         "MovementSystem"
///     }
///
///     fn component_access(&self) -> Access {
///         let mut access = Access::new();
///         access.add_write(ComponentId::of::<Position>());
///         access.add_read(ComponentId::of::<Velocity>());
///         access
///     }
///
///     fn run(&mut self, world: &mut World) {
///         // Movement logic here
///     }
/// }
/// ```
///
/// # Lifecycle
///
/// Systems have optional lifecycle hooks:
///
/// - `initialize()`: Called once when the system is first added
/// - `run()`: Called each time the system executes
///
/// # Thread Safety
///
/// The `Send` bound ensures systems can be sent between threads. However,
/// the scheduler ensures only one system runs on a World at a time (for now).
pub trait System: Send {
    /// Returns the name of this system.
    ///
    /// Used for debugging, logging, and profiling. Should be a short,
    /// descriptive name.
    fn name(&self) -> &'static str;

    /// Returns the component access pattern for this system.
    ///
    /// Override this to declare which components your system reads and writes.
    /// The default implementation returns empty access (no components).
    ///
    /// Accurate access information enables:
    /// - Parallel execution of non-conflicting systems
    /// - Compile-time verification of system ordering
    /// - Runtime conflict detection
    #[inline]
    fn component_access(&self) -> Access {
        Access::new()
    }

    /// Called once when the system is first added to a scheduler.
    ///
    /// Use this for one-time initialization that requires world access.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world
    #[inline]
    fn initialize(&mut self, _world: &mut World) {
        // Default: no initialization needed
    }

    /// Runs the system on the given world.
    ///
    /// This is called each frame (or each time the system's stage runs).
    /// Implement your system logic here.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world containing all ECS data
    fn run(&mut self, world: &mut World);

    /// Returns whether this system should run.
    ///
    /// Override this to conditionally skip system execution.
    /// The default implementation always returns `true`.
    ///
    /// # Arguments
    ///
    /// * `world` - Reference to the world (for checking conditions)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    /// use goud_engine::ecs::system::System;
    ///
    /// struct GamePaused;
    /// impl Component for GamePaused {}
    ///
    /// struct PhysicsSystem;
    ///
    /// impl System for PhysicsSystem {
    ///     fn name(&self) -> &'static str { "PhysicsSystem" }
    ///
    ///     fn should_run(&self, world: &World) -> bool {
    ///         // Skip physics when game is paused
    ///         // (In a real implementation, you'd check a resource)
    ///         true
    ///     }
    ///
    ///     fn run(&mut self, world: &mut World) {
    ///         // Physics logic
    ///     }
    /// }
    /// ```
    #[inline]
    fn should_run(&self, _world: &World) -> bool {
        true
    }

    /// Returns true if this system only reads data.
    ///
    /// Read-only systems can potentially run in parallel with other
    /// read-only systems. The default implementation delegates to
    /// `component_access().is_read_only()`.
    #[inline]
    fn is_read_only(&self) -> bool {
        self.component_access().is_read_only()
    }
}

// =============================================================================
// BoxedSystem
// =============================================================================

/// A type-erased, boxed system.
///
/// `BoxedSystem` wraps any `System` implementor in a `Box`, enabling
/// dynamic dispatch and storage in collections.
///
/// # When to Use
///
/// Use `BoxedSystem` when you need to:
/// - Store systems of different types in a single collection
/// - Pass systems around without knowing their concrete type
/// - Build dynamic system pipelines
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::system::{System, BoxedSystem};
///
/// struct SystemA;
/// impl System for SystemA {
///     fn name(&self) -> &'static str { "SystemA" }
///     fn run(&mut self, _world: &mut World) {}
/// }
///
/// struct SystemB;
/// impl System for SystemB {
///     fn name(&self) -> &'static str { "SystemB" }
///     fn run(&mut self, _world: &mut World) {}
/// }
///
/// // Store different system types in a Vec
/// let mut systems: Vec<BoxedSystem> = vec![
///     BoxedSystem::new(SystemA),
///     BoxedSystem::new(SystemB),
/// ];
///
/// // Run all systems
/// let mut world = World::new();
/// for system in &mut systems {
///     system.run(&mut world);
/// }
/// ```
pub struct BoxedSystem {
    /// The boxed system.
    inner: Box<dyn System>,
    /// Cached system ID.
    id: SystemId,
}

impl BoxedSystem {
    /// Creates a new boxed system from any System implementor.
    #[inline]
    pub fn new<S: System + 'static>(system: S) -> Self {
        Self {
            inner: Box::new(system),
            id: SystemId::new(),
        }
    }

    /// Returns the system's unique ID.
    #[inline]
    pub fn id(&self) -> SystemId {
        self.id
    }

    /// Returns the system's name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// Returns the system's component access pattern.
    #[inline]
    pub fn component_access(&self) -> Access {
        self.inner.component_access()
    }

    /// Initializes the system.
    #[inline]
    pub fn initialize(&mut self, world: &mut World) {
        self.inner.initialize(world);
    }

    /// Runs the system.
    #[inline]
    pub fn run(&mut self, world: &mut World) {
        self.inner.run(world);
    }

    /// Checks if the system should run.
    #[inline]
    pub fn should_run(&self, world: &World) -> bool {
        self.inner.should_run(world)
    }

    /// Returns true if this system only reads data.
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    /// Checks if this system conflicts with another.
    #[inline]
    pub fn conflicts_with(&self, other: &BoxedSystem) -> bool {
        self.component_access()
            .conflicts_with(&other.component_access())
    }
}

impl std::fmt::Debug for BoxedSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedSystem")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("is_read_only", &self.is_read_only())
            .finish()
    }
}

// =============================================================================
// IntoSystem Trait
// =============================================================================

/// A trait for types that can be converted into a [`System`].
///
/// This trait enables ergonomic system registration by allowing functions
/// and other types to be automatically converted into boxed systems.
///
/// # Implemented For
///
/// - Any type implementing [`System`]
/// - Functions with valid system parameters (future, Step 3.1.5)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::system::{System, BoxedSystem, IntoSystem};
///
/// struct MySystem;
///
/// impl System for MySystem {
///     fn name(&self) -> &'static str { "MySystem" }
///     fn run(&mut self, _world: &mut World) {}
/// }
///
/// // Convert to BoxedSystem using IntoSystem
/// let boxed: BoxedSystem = MySystem.into_system();
/// assert_eq!(boxed.name(), "MySystem");
/// ```
pub trait IntoSystem<Marker = ()> {
    /// The concrete system type this converts to.
    type System: System + 'static;

    /// Converts this into a system.
    fn into_system(self) -> BoxedSystem;
}

// Blanket implementation for any System
impl<S: System + 'static> IntoSystem for S {
    type System = S;

    #[inline]
    fn into_system(self) -> BoxedSystem {
        BoxedSystem::new(self)
    }
}
