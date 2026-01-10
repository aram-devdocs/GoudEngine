//! Core System trait and related types.
//!
//! This module defines the foundational [`System`] trait that all systems implement,
//! along with supporting types for system identification and metadata.

use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ecs::query::Access;
use crate::ecs::World;

// =============================================================================
// SystemId
// =============================================================================

/// Atomic counter for generating unique SystemIds.
static NEXT_SYSTEM_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a system instance.
///
/// `SystemId` provides a way to identify and reference systems after they've
/// been registered with a scheduler. Each system instance gets a unique ID
/// when registered.
///
/// # Thread Safety
///
/// SystemId is Copy and can be shared freely between threads.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::system::SystemId;
///
/// // Create a new unique ID
/// let id1 = SystemId::new();
/// let id2 = SystemId::new();
///
/// assert_ne!(id1, id2);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemId(u64);

impl SystemId {
    /// An invalid/placeholder system ID.
    ///
    /// Used as a default value before a system is registered.
    pub const INVALID: SystemId = SystemId(0);

    /// Creates a new unique SystemId.
    ///
    /// Each call returns a different ID, guaranteed to be unique
    /// within the process lifetime.
    #[inline]
    pub fn new() -> Self {
        SystemId(NEXT_SYSTEM_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates a SystemId from a raw value.
    ///
    /// # Safety Note
    ///
    /// This should only be used for deserialization or testing.
    /// For normal use, prefer [`SystemId::new()`].
    #[inline]
    pub const fn from_raw(id: u64) -> Self {
        SystemId(id)
    }

    /// Returns the raw ID value.
    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Returns true if this is the INVALID system ID.
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this is a valid (non-INVALID) system ID.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl Default for SystemId {
    fn default() -> Self {
        SystemId::INVALID
    }
}

impl std::fmt::Debug for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "SystemId(INVALID)")
        } else {
            write!(f, "SystemId({})", self.0)
        }
    }
}

impl std::fmt::Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "INVALID")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

// =============================================================================
// SystemMeta
// =============================================================================

/// Metadata about a system.
///
/// `SystemMeta` stores information about a system that is useful for:
/// - Debugging and logging (name)
/// - Conflict detection (component access)
/// - Scheduling (dependencies, ordering)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::system::SystemMeta;
///
/// let meta = SystemMeta::new("MySystem");
/// assert_eq!(meta.name(), "MySystem");
/// ```
#[derive(Debug, Clone)]
pub struct SystemMeta {
    /// Human-readable name of the system.
    name: Cow<'static, str>,
    /// Component and resource access patterns.
    component_access: Access,
}

impl SystemMeta {
    /// Creates new system metadata with the given name.
    #[inline]
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            component_access: Access::new(),
        }
    }

    /// Returns the system's name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the system's name.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = name.into();
    }

    /// Returns the system's component access pattern.
    #[inline]
    pub fn component_access(&self) -> &Access {
        &self.component_access
    }

    /// Returns a mutable reference to the component access pattern.
    #[inline]
    pub fn component_access_mut(&mut self) -> &mut Access {
        &mut self.component_access
    }

    /// Sets the component access pattern.
    #[inline]
    pub fn set_component_access(&mut self, access: Access) {
        self.component_access = access;
    }

    /// Checks if this system's access conflicts with another.
    ///
    /// Two systems conflict if one writes to a component that the other
    /// reads or writes.
    #[inline]
    pub fn conflicts_with(&self, other: &SystemMeta) -> bool {
        self.component_access
            .conflicts_with(&other.component_access)
    }

    /// Returns true if this system only reads data (no writes).
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.component_access.is_read_only()
    }
}

impl Default for SystemMeta {
    fn default() -> Self {
        Self::new("UnnamedSystem")
    }
}

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentId;
    use crate::ecs::Component;

    // Test components
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    // =========================================================================
    // SystemId Tests
    // =========================================================================

    mod system_id {
        use super::*;

        #[test]
        fn test_new_creates_unique_ids() {
            let id1 = SystemId::new();
            let id2 = SystemId::new();
            let id3 = SystemId::new();

            assert_ne!(id1, id2);
            assert_ne!(id2, id3);
            assert_ne!(id1, id3);
        }

        #[test]
        fn test_invalid_constant() {
            let invalid = SystemId::INVALID;
            assert!(invalid.is_invalid());
            assert!(!invalid.is_valid());
            assert_eq!(invalid.raw(), 0);
        }

        #[test]
        fn test_valid_ids() {
            let id = SystemId::new();
            assert!(id.is_valid());
            assert!(!id.is_invalid());
        }

        #[test]
        fn test_from_raw() {
            let id = SystemId::from_raw(42);
            assert_eq!(id.raw(), 42);
            assert!(id.is_valid());
        }

        #[test]
        fn test_from_raw_zero_is_invalid() {
            let id = SystemId::from_raw(0);
            assert!(id.is_invalid());
        }

        #[test]
        fn test_default_is_invalid() {
            let id = SystemId::default();
            assert!(id.is_invalid());
        }

        #[test]
        fn test_equality() {
            let id1 = SystemId::from_raw(10);
            let id2 = SystemId::from_raw(10);
            let id3 = SystemId::from_raw(20);

            assert_eq!(id1, id2);
            assert_ne!(id1, id3);
        }

        #[test]
        fn test_ordering() {
            let id1 = SystemId::from_raw(10);
            let id2 = SystemId::from_raw(20);

            assert!(id1 < id2);
            assert!(id2 > id1);
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let id1 = SystemId::from_raw(10);
            let id2 = SystemId::from_raw(20);
            let id3 = SystemId::from_raw(10);

            let mut set = HashSet::new();
            set.insert(id1);
            set.insert(id2);
            set.insert(id3);

            assert_eq!(set.len(), 2); // id1 and id3 are the same
        }

        #[test]
        fn test_debug_format_invalid() {
            let id = SystemId::INVALID;
            let debug = format!("{id:?}");
            assert!(debug.contains("INVALID"));
        }

        #[test]
        fn test_debug_format_valid() {
            let id = SystemId::from_raw(42);
            let debug = format!("{id:?}");
            assert!(debug.contains("42"));
        }

        #[test]
        fn test_display_format() {
            let id = SystemId::from_raw(42);
            let display = format!("{id}");
            assert_eq!(display, "42");

            let invalid = SystemId::INVALID;
            let display_invalid = format!("{invalid}");
            assert_eq!(display_invalid, "INVALID");
        }

        #[test]
        fn test_copy_and_clone() {
            let id1 = SystemId::new();
            let id2 = id1; // Copy
            let id3 = id1.clone(); // Clone

            assert_eq!(id1, id2);
            assert_eq!(id1, id3);
        }

        #[test]
        fn test_thread_safety() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<SystemId>();
        }
    }

    // =========================================================================
    // SystemMeta Tests
    // =========================================================================

    mod system_meta {
        use super::*;

        #[test]
        fn test_new() {
            let meta = SystemMeta::new("TestSystem");
            assert_eq!(meta.name(), "TestSystem");
            assert!(meta.is_read_only());
        }

        #[test]
        fn test_new_with_string() {
            let name = String::from("DynamicSystem");
            let meta = SystemMeta::new(name);
            assert_eq!(meta.name(), "DynamicSystem");
        }

        #[test]
        fn test_set_name() {
            let mut meta = SystemMeta::new("OldName");
            meta.set_name("NewName");
            assert_eq!(meta.name(), "NewName");
        }

        #[test]
        fn test_default() {
            let meta = SystemMeta::default();
            assert_eq!(meta.name(), "UnnamedSystem");
        }

        #[test]
        fn test_component_access() {
            let mut meta = SystemMeta::new("Test");
            assert!(meta.component_access().is_read_only());

            meta.component_access_mut()
                .add_write(ComponentId::of::<Position>());
            assert!(!meta.is_read_only());
        }

        #[test]
        fn test_set_component_access() {
            let mut meta = SystemMeta::new("Test");

            let mut access = Access::new();
            access.add_read(ComponentId::of::<Position>());
            meta.set_component_access(access);

            assert!(meta.component_access().is_read_only());
        }

        #[test]
        fn test_conflicts_with_no_conflict() {
            let mut meta1 = SystemMeta::new("System1");
            let mut meta2 = SystemMeta::new("System2");

            meta1
                .component_access_mut()
                .add_read(ComponentId::of::<Position>());
            meta2
                .component_access_mut()
                .add_read(ComponentId::of::<Velocity>());

            assert!(!meta1.conflicts_with(&meta2));
        }

        #[test]
        fn test_conflicts_with_read_read_same() {
            let mut meta1 = SystemMeta::new("System1");
            let mut meta2 = SystemMeta::new("System2");

            meta1
                .component_access_mut()
                .add_read(ComponentId::of::<Position>());
            meta2
                .component_access_mut()
                .add_read(ComponentId::of::<Position>());

            // Two reads don't conflict
            assert!(!meta1.conflicts_with(&meta2));
        }

        #[test]
        fn test_conflicts_with_write_read() {
            let mut meta1 = SystemMeta::new("System1");
            let mut meta2 = SystemMeta::new("System2");

            meta1
                .component_access_mut()
                .add_write(ComponentId::of::<Position>());
            meta2
                .component_access_mut()
                .add_read(ComponentId::of::<Position>());

            assert!(meta1.conflicts_with(&meta2));
            assert!(meta2.conflicts_with(&meta1));
        }

        #[test]
        fn test_conflicts_with_write_write() {
            let mut meta1 = SystemMeta::new("System1");
            let mut meta2 = SystemMeta::new("System2");

            meta1
                .component_access_mut()
                .add_write(ComponentId::of::<Position>());
            meta2
                .component_access_mut()
                .add_write(ComponentId::of::<Position>());

            assert!(meta1.conflicts_with(&meta2));
        }

        #[test]
        fn test_is_read_only() {
            let mut meta = SystemMeta::new("Test");
            assert!(meta.is_read_only());

            meta.component_access_mut()
                .add_read(ComponentId::of::<Position>());
            assert!(meta.is_read_only());

            meta.component_access_mut()
                .add_write(ComponentId::of::<Velocity>());
            assert!(!meta.is_read_only());
        }

        #[test]
        fn test_clone() {
            let mut meta = SystemMeta::new("Test");
            meta.component_access_mut()
                .add_write(ComponentId::of::<Position>());

            let cloned = meta.clone();
            assert_eq!(cloned.name(), "Test");
            assert!(!cloned.is_read_only());
        }

        #[test]
        fn test_debug() {
            let meta = SystemMeta::new("TestSystem");
            let debug = format!("{meta:?}");
            assert!(debug.contains("TestSystem"));
        }
    }

    // =========================================================================
    // System Trait Tests
    // =========================================================================

    mod system_trait {
        use super::*;

        // A simple test system
        struct TestSystem {
            run_count: u32,
            initialized: bool,
        }

        impl TestSystem {
            fn new() -> Self {
                Self {
                    run_count: 0,
                    initialized: false,
                }
            }
        }

        impl System for TestSystem {
            fn name(&self) -> &'static str {
                "TestSystem"
            }

            fn initialize(&mut self, _world: &mut World) {
                self.initialized = true;
            }

            fn run(&mut self, _world: &mut World) {
                self.run_count += 1;
            }
        }

        // A system with component access
        struct AccessTrackingSystem;

        impl System for AccessTrackingSystem {
            fn name(&self) -> &'static str {
                "AccessTrackingSystem"
            }

            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Position>());
                access.add_read(ComponentId::of::<Velocity>());
                access
            }

            fn run(&mut self, _world: &mut World) {}
        }

        // A conditional system
        struct ConditionalSystem {
            should_run: bool,
        }

        impl System for ConditionalSystem {
            fn name(&self) -> &'static str {
                "ConditionalSystem"
            }

            fn should_run(&self, _world: &World) -> bool {
                self.should_run
            }

            fn run(&mut self, _world: &mut World) {}
        }

        #[test]
        fn test_system_name() {
            let system = TestSystem::new();
            assert_eq!(system.name(), "TestSystem");
        }

        #[test]
        fn test_system_run() {
            let mut system = TestSystem::new();
            let mut world = World::new();

            assert_eq!(system.run_count, 0);
            system.run(&mut world);
            assert_eq!(system.run_count, 1);
            system.run(&mut world);
            assert_eq!(system.run_count, 2);
        }

        #[test]
        fn test_system_initialize() {
            let mut system = TestSystem::new();
            let mut world = World::new();

            assert!(!system.initialized);
            system.initialize(&mut world);
            assert!(system.initialized);
        }

        #[test]
        fn test_system_component_access_default() {
            let system = TestSystem::new();
            let access = system.component_access();
            assert!(access.is_read_only());
        }

        #[test]
        fn test_system_component_access_custom() {
            let system = AccessTrackingSystem;
            let access = system.component_access();

            assert!(!access.is_read_only());
            assert!(access.writes().contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_system_should_run_default() {
            let system = TestSystem::new();
            let world = World::new();

            assert!(system.should_run(&world));
        }

        #[test]
        fn test_system_should_run_custom() {
            let world = World::new();

            let system_yes = ConditionalSystem { should_run: true };
            let system_no = ConditionalSystem { should_run: false };

            assert!(system_yes.should_run(&world));
            assert!(!system_no.should_run(&world));
        }

        #[test]
        fn test_system_is_read_only() {
            let system1 = TestSystem::new();
            let system2 = AccessTrackingSystem;

            assert!(system1.is_read_only());
            assert!(!system2.is_read_only());
        }

        #[test]
        fn test_system_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TestSystem>();
            assert_send::<AccessTrackingSystem>();
        }
    }

    // =========================================================================
    // BoxedSystem Tests
    // =========================================================================

    mod boxed_system {
        use super::*;

        struct SimpleSystem {
            run_count: u32,
        }

        impl System for SimpleSystem {
            fn name(&self) -> &'static str {
                "SimpleSystem"
            }

            fn run(&mut self, _world: &mut World) {
                self.run_count += 1;
            }
        }

        struct WriteSystem;

        impl System for WriteSystem {
            fn name(&self) -> &'static str {
                "WriteSystem"
            }

            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Position>());
                access
            }

            fn run(&mut self, _world: &mut World) {}
        }

        struct ReadSystem;

        impl System for ReadSystem {
            fn name(&self) -> &'static str {
                "ReadSystem"
            }

            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_read(ComponentId::of::<Position>());
                access
            }

            fn run(&mut self, _world: &mut World) {}
        }

        #[test]
        fn test_new() {
            let boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
            assert_eq!(boxed.name(), "SimpleSystem");
            assert!(boxed.id().is_valid());
        }

        #[test]
        fn test_unique_ids() {
            let boxed1 = BoxedSystem::new(SimpleSystem { run_count: 0 });
            let boxed2 = BoxedSystem::new(SimpleSystem { run_count: 0 });

            assert_ne!(boxed1.id(), boxed2.id());
        }

        #[test]
        fn test_run() {
            let mut boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
            let mut world = World::new();

            boxed.run(&mut world);
            boxed.run(&mut world);
            // Can't directly check run_count, but we can verify no panics
        }

        #[test]
        fn test_component_access() {
            let boxed = BoxedSystem::new(WriteSystem);
            let access = boxed.component_access();

            assert!(access.writes().contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_is_read_only() {
            let write_system = BoxedSystem::new(WriteSystem);
            let read_system = BoxedSystem::new(ReadSystem);

            assert!(!write_system.is_read_only());
            assert!(read_system.is_read_only());
        }

        #[test]
        fn test_conflicts_with() {
            let write = BoxedSystem::new(WriteSystem);
            let read = BoxedSystem::new(ReadSystem);

            // Write and Read of same component conflict
            assert!(write.conflicts_with(&read));
            assert!(read.conflicts_with(&write));
        }

        #[test]
        fn test_no_conflict_different_components() {
            struct VelocityWriter;
            impl System for VelocityWriter {
                fn name(&self) -> &'static str {
                    "VelocityWriter"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Velocity>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let pos_write = BoxedSystem::new(WriteSystem);
            let vel_write = BoxedSystem::new(VelocityWriter);

            assert!(!pos_write.conflicts_with(&vel_write));
        }

        #[test]
        fn test_should_run() {
            struct AlwaysSkip;
            impl System for AlwaysSkip {
                fn name(&self) -> &'static str {
                    "AlwaysSkip"
                }
                fn should_run(&self, _: &World) -> bool {
                    false
                }
                fn run(&mut self, _: &mut World) {}
            }

            let world = World::new();
            let normal = BoxedSystem::new(SimpleSystem { run_count: 0 });
            let skip = BoxedSystem::new(AlwaysSkip);

            assert!(normal.should_run(&world));
            assert!(!skip.should_run(&world));
        }

        #[test]
        fn test_initialize() {
            struct InitSystem {
                initialized: bool,
            }
            impl System for InitSystem {
                fn name(&self) -> &'static str {
                    "InitSystem"
                }
                fn initialize(&mut self, _: &mut World) {
                    self.initialized = true;
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut boxed = BoxedSystem::new(InitSystem { initialized: false });
            let mut world = World::new();

            boxed.initialize(&mut world);
            // Can verify no panics; internal state isn't accessible
        }

        #[test]
        fn test_debug() {
            let boxed = BoxedSystem::new(SimpleSystem { run_count: 0 });
            let debug = format!("{boxed:?}");

            assert!(debug.contains("BoxedSystem"));
            assert!(debug.contains("SimpleSystem"));
        }

        #[test]
        fn test_collection_of_different_systems() {
            let systems: Vec<BoxedSystem> = vec![
                BoxedSystem::new(SimpleSystem { run_count: 0 }),
                BoxedSystem::new(WriteSystem),
                BoxedSystem::new(ReadSystem),
            ];

            assert_eq!(systems.len(), 3);
            assert_eq!(systems[0].name(), "SimpleSystem");
            assert_eq!(systems[1].name(), "WriteSystem");
            assert_eq!(systems[2].name(), "ReadSystem");
        }

        #[test]
        fn test_run_multiple_systems() {
            let mut world = World::new();
            let mut systems: Vec<BoxedSystem> = vec![
                BoxedSystem::new(SimpleSystem { run_count: 0 }),
                BoxedSystem::new(WriteSystem),
            ];

            for system in &mut systems {
                system.run(&mut world);
            }
            // Verify no panics during execution
        }
    }

    // =========================================================================
    // IntoSystem Tests
    // =========================================================================

    mod into_system {
        use super::*;

        struct MySystem;

        impl System for MySystem {
            fn name(&self) -> &'static str {
                "MySystem"
            }

            fn run(&mut self, _world: &mut World) {}
        }

        #[test]
        fn test_system_into_boxed() {
            let boxed: BoxedSystem = MySystem.into_system();
            assert_eq!(boxed.name(), "MySystem");
        }

        #[test]
        fn test_into_system_preserves_behavior() {
            struct CounterSystem {
                count: u32,
            }
            impl System for CounterSystem {
                fn name(&self) -> &'static str {
                    "CounterSystem"
                }
                fn run(&mut self, _: &mut World) {
                    self.count += 1;
                }
            }

            let mut boxed = CounterSystem { count: 0 }.into_system();
            let mut world = World::new();

            boxed.run(&mut world);
            boxed.run(&mut world);
            // System runs without panics
        }

        #[test]
        fn test_into_system_preserves_access() {
            struct AccessSystem;
            impl System for AccessSystem {
                fn name(&self) -> &'static str {
                    "AccessSystem"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let boxed: BoxedSystem = AccessSystem.into_system();
            assert!(!boxed.is_read_only());
            assert!(boxed
                .component_access()
                .writes()
                .contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_multiple_into_system() {
            struct SystemA;
            impl System for SystemA {
                fn name(&self) -> &'static str {
                    "A"
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct SystemB;
            impl System for SystemB {
                fn name(&self) -> &'static str {
                    "B"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let systems: Vec<BoxedSystem> = vec![SystemA.into_system(), SystemB.into_system()];

            assert_eq!(systems[0].name(), "A");
            assert_eq!(systems[1].name(), "B");
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_system_modifies_world() {
            struct SpawnSystem;

            impl System for SpawnSystem {
                fn name(&self) -> &'static str {
                    "SpawnSystem"
                }

                fn run(&mut self, world: &mut World) {
                    world.spawn_empty();
                }
            }

            let mut world = World::new();
            let mut system = SpawnSystem;

            assert_eq!(world.entity_count(), 0);
            system.run(&mut world);
            assert_eq!(world.entity_count(), 1);
            system.run(&mut world);
            assert_eq!(world.entity_count(), 2);
        }

        #[test]
        fn test_system_adds_components() {
            struct AddPositionSystem {
                entities: Vec<crate::ecs::Entity>,
            }

            impl System for AddPositionSystem {
                fn name(&self) -> &'static str {
                    "AddPositionSystem"
                }

                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }

                fn run(&mut self, world: &mut World) {
                    for &entity in &self.entities {
                        world.insert(entity, Position { x: 0.0, y: 0.0 });
                    }
                }
            }

            let mut world = World::new();
            let e1 = world.spawn_empty();
            let e2 = world.spawn_empty();

            let mut system = AddPositionSystem {
                entities: vec![e1, e2],
            };
            system.run(&mut world);

            assert!(world.has::<Position>(e1));
            assert!(world.has::<Position>(e2));
        }

        #[test]
        fn test_conditional_system_execution() {
            struct CountingSystem {
                count: u32,
                max_runs: u32,
            }

            impl System for CountingSystem {
                fn name(&self) -> &'static str {
                    "CountingSystem"
                }

                fn should_run(&self, _world: &World) -> bool {
                    self.count < self.max_runs
                }

                fn run(&mut self, _world: &mut World) {
                    self.count += 1;
                }
            }

            let mut world = World::new();
            let mut system = CountingSystem {
                count: 0,
                max_runs: 3,
            };

            // Run only if should_run returns true
            for _ in 0..5 {
                if system.should_run(&world) {
                    system.run(&mut world);
                }
            }

            assert_eq!(system.count, 3);
        }

        #[test]
        fn test_boxed_system_pipeline() {
            struct IncrementCounter {
                counter: *mut u32,
            }
            unsafe impl Send for IncrementCounter {}

            impl System for IncrementCounter {
                fn name(&self) -> &'static str {
                    "IncrementCounter"
                }
                fn run(&mut self, _: &mut World) {
                    unsafe {
                        *self.counter += 1;
                    }
                }
            }

            let mut counter: u32 = 0;
            let counter_ptr = &mut counter as *mut u32;

            let mut systems: Vec<BoxedSystem> = vec![
                BoxedSystem::new(IncrementCounter {
                    counter: counter_ptr,
                }),
                BoxedSystem::new(IncrementCounter {
                    counter: counter_ptr,
                }),
                BoxedSystem::new(IncrementCounter {
                    counter: counter_ptr,
                }),
            ];

            let mut world = World::new();

            for system in &mut systems {
                system.run(&mut world);
            }

            assert_eq!(counter, 3);
        }

        #[test]
        fn test_system_access_conflict_detection() {
            struct PositionWriter;
            impl System for PositionWriter {
                fn name(&self) -> &'static str {
                    "PositionWriter"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct PositionReader;
            impl System for PositionReader {
                fn name(&self) -> &'static str {
                    "PositionReader"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_read(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct VelocityReader;
            impl System for VelocityReader {
                fn name(&self) -> &'static str {
                    "VelocityReader"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_read(ComponentId::of::<Velocity>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let writer = BoxedSystem::new(PositionWriter);
            let reader = BoxedSystem::new(PositionReader);
            let vel_reader = BoxedSystem::new(VelocityReader);

            // Writer conflicts with reader of same component
            assert!(writer.conflicts_with(&reader));
            assert!(reader.conflicts_with(&writer));

            // Writer doesn't conflict with reader of different component
            assert!(!writer.conflicts_with(&vel_reader));

            // Two readers don't conflict
            assert!(!reader.conflicts_with(&vel_reader));
        }
    }
}
