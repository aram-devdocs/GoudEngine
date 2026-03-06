//! Core traits for the system parameter abstraction.
//!
//! Defines [`SystemParamState`], [`SystemParam`], and [`ReadOnlySystemParam`].

use crate::ecs::query::Access;
use crate::ecs::World;

// =============================================================================
// SystemParamState Trait
// =============================================================================

/// Trait for the cached state of a system parameter.
///
/// System parameters may need to cache information (like component IDs or
/// resource handles) for efficient repeated extraction. This trait defines
/// the state type and how to initialize it.
///
/// # Requirements
///
/// State types must be `Send + Sync` for parallel system execution.
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::World;
/// use goud_engine::ecs::system::SystemParamState;
///
/// // A simple state that caches nothing
/// impl SystemParamState for () {
///     fn init(_world: &mut World) -> Self {}
/// }
/// ```
pub trait SystemParamState: Send + Sync + Sized + 'static {
    /// Initializes the state from the world.
    ///
    /// Called once when the system is first registered. The returned state
    /// is cached and reused for all subsequent parameter extractions.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world (may be needed for registration)
    fn init(world: &mut World) -> Self;

    /// Applies any pending state changes.
    ///
    /// Called after system execution to apply deferred operations like
    /// command buffers. The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the world
    #[inline]
    fn apply(&mut self, _world: &mut World) {}
}

// Blanket implementation for unit type (no state needed)
impl SystemParamState for () {
    #[inline]
    fn init(_world: &mut World) -> Self {}
}

// =============================================================================
// SystemParam Trait
// =============================================================================

/// Trait for types that can be used as system function parameters.
///
/// `SystemParam` defines how a type is extracted from the [`World`] when a
/// function system runs. It provides:
///
/// 1. **State caching**: What data to cache between runs
/// 2. **Access tracking**: What components/resources are accessed
/// 3. **Parameter fetching**: How to get the actual value from World
///
/// # Design
///
/// The trait uses associated types and GATs (Generic Associated Types) to
/// express the complex lifetime relationships between the World borrow,
/// the parameter value, and the cached state.
///
/// # Implementing SystemParam
///
/// Most users won't implement this trait directly. Instead, use the built-in
/// parameter types (Query, Res, ResMut) or derive macros (future).
///
/// # Thread Safety
///
/// System parameters must properly track their access patterns via
/// `update_access`. This enables the scheduler to detect conflicts and
/// safely parallelize system execution.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::World;
/// use goud_engine::ecs::query::Access;
/// use goud_engine::ecs::system::{SystemParam, SystemParamState, ReadOnlySystemParam};
///
/// // A parameter that just provides the entity count
/// struct EntityCount(usize);
///
/// // State for EntityCount (none needed)
/// struct EntityCountState;
///
/// impl SystemParamState for EntityCountState {
///     fn init(_world: &mut World) -> Self {
///         EntityCountState
///     }
/// }
///
/// impl SystemParam for EntityCount {
///     type State = EntityCountState;
///     type Item<'w, 's> = EntityCount;
///
///     fn update_access(_state: &Self::State, _access: &mut Access) {
///         // No component access needed
///     }
///
///     fn get_param<'w, 's>(
///         _state: &'s mut Self::State,
///         world: &'w World,
///     ) -> Self::Item<'w, 's> {
///         EntityCount(world.entity_count())
///     }
/// }
///
/// // EntityCount is read-only
/// impl ReadOnlySystemParam for EntityCount {}
/// ```
pub trait SystemParam: Sized {
    /// The cached state for this parameter.
    ///
    /// State is initialized once via `SystemParamState::init` and reused
    /// for all subsequent parameter extractions.
    type State: SystemParamState;

    /// The type of value this parameter produces.
    ///
    /// This is a GAT with two lifetime parameters:
    /// - `'w`: The World borrow lifetime
    /// - `'s`: The State borrow lifetime
    ///
    /// These separate lifetimes allow for both:
    /// - References to World data (`'w`)
    /// - References to cached state (`'s`)
    type Item<'w, 's>;

    /// Updates the access pattern for this parameter.
    ///
    /// Called during system setup to build the complete access pattern.
    /// Implementations should add any component/resource reads or writes
    /// to the provided access tracker.
    ///
    /// # Arguments
    ///
    /// * `state` - The parameter's cached state
    /// * `access` - Access tracker to update
    fn update_access(state: &Self::State, access: &mut Access);

    /// Extracts the parameter value from the World.
    ///
    /// Called each time the system runs to get the actual parameter value.
    /// The returned value has lifetimes tied to both the World and the state.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable reference to cached state
    /// * `world` - Immutable reference to the World
    ///
    /// # Returns
    ///
    /// The extracted parameter value.
    fn get_param<'w, 's>(state: &'s mut Self::State, world: &'w World) -> Self::Item<'w, 's>;

    /// Extracts the parameter value from a mutable World reference.
    ///
    /// Some parameters (like `ResMut`) need mutable world access. This method
    /// provides that access. The default implementation calls `get_param`.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable reference to cached state
    /// * `world` - Mutable reference to the World
    ///
    /// # Returns
    ///
    /// The extracted parameter value.
    #[inline]
    fn get_param_mut<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        // Default: use immutable access
        Self::get_param(state, world)
    }
}

// =============================================================================
// ReadOnlySystemParam Marker Trait
// =============================================================================

/// Marker trait for system parameters that only read data.
///
/// `ReadOnlySystemParam` indicates that a parameter does not mutate the World.
/// This enables optimizations:
///
/// 1. **Parallel execution**: Multiple read-only systems can run concurrently
/// 2. **Shared borrows**: Can coexist with other readers
/// 3. **Cache friendliness**: Read-only access enables certain optimizations
///
/// # Safety
///
/// Implementors must ensure the parameter truly never mutates World state,
/// even when given mutable access.
///
/// # Built-in Implementations
///
/// - `Query<Q, F>` where Q: ReadOnlyWorldQuery
/// - `Res<T>` (immutable resource access)
/// - `&World` (immutable world access)
/// - Tuples of read-only parameters
///
/// # Example
///
/// ```
/// use goud_engine::ecs::system::ReadOnlySystemParam;
///
/// // Marker trait - just implement it for read-only types
/// // struct MyReadOnlyParam;
/// // impl ReadOnlySystemParam for MyReadOnlyParam {}
/// ```
pub trait ReadOnlySystemParam: SystemParam {}

// =============================================================================
// Unit Type Implementation
// =============================================================================

/// Unit type `()` as a system parameter represents no data.
///
/// Useful as a base case for tuple parameters or for systems that
/// don't need any parameters extracted from the World.
impl SystemParam for () {
    type State = ();
    type Item<'w, 's> = ();

    #[inline]
    fn update_access(_state: &Self::State, _access: &mut Access) {
        // No access needed
    }

    #[inline]
    fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {}
}

impl ReadOnlySystemParam for () {}
