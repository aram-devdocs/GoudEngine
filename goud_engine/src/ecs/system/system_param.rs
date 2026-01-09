//! System parameters for extracting data from the World.
//!
//! This module defines the [`SystemParam`] trait, which enables types to be
//! used as function system parameters. When a function is converted into a
//! system, its parameters are extracted from the World using this trait.
//!
//! # Architecture
//!
//! The system parameter architecture consists of:
//!
//! - [`SystemParam`]: Core trait for types that can be extracted from World
//! - [`SystemParamState`]: Cached state for efficient repeated extraction
//!
//! # Built-in Parameters
//!
//! The following types implement `SystemParam`:
//!
//! - `Query<Q, F>` - Queries for entities with specific components (Step 3.1.3)
//! - `Res<T>` - Immutable resource access (Step 3.1.4)
//! - `ResMut<T>` - Mutable resource access (Step 3.1.4)
//! - `World` (as `&World` or `&mut World`) - Direct world access
//! - Tuples of system parameters
//!
//! # Example
//!
//! ```ignore
//! // Future: Function systems (Step 3.1.5)
//! fn movement_system(query: Query<(&mut Position, &Velocity)>) {
//!     for (pos, vel) in query.iter_mut() {
//!         pos.x += vel.x;
//!         pos.y += vel.y;
//!     }
//! }
//!
//! // The function's parameters implement SystemParam, allowing automatic
//! // extraction from the World when the system runs.
//! ```
//!
//! # Safety
//!
//! System parameters track their access patterns to enable conflict detection.
//! The scheduler uses this information to determine which systems can run in
//! parallel.

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

// =============================================================================
// Tuple Implementations
// =============================================================================

/// Implements SystemParam for tuples of parameters.
///
/// This allows systems to accept multiple parameters:
/// ```ignore
/// fn my_system(query: Query<&Position>, res: Res<Time>) { ... }
/// ```
macro_rules! impl_system_param_tuple {
    ($($name:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(clippy::unused_unit)]
        impl<$($name: SystemParam),*> SystemParam for ($($name,)*) {
            type State = ($($name::State,)*);
            type Item<'w, 's> = ($($name::Item<'w, 's>,)*);

            #[inline]
            fn update_access(_state: &Self::State, _access: &mut Access) {
                let ($($name,)*) = _state;
                $($name::update_access($name, _access);)*
            }

            #[inline]
            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                let ($($name,)*) = _state;
                ($($name::get_param($name, _world),)*)
            }

            #[inline]
            fn get_param_mut<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w mut World,
            ) -> Self::Item<'w, 's> {
                let ($($name,)*) = _state;
                // Note: This requires unsafe in real implementation to split borrows
                // For now, we just use the immutable version
                ($($name::get_param($name, _world),)*)
            }
        }

        #[allow(non_snake_case)]
        impl<$($name: SystemParamState),*> SystemParamState for ($($name,)*) {
            #[inline]
            fn init(_world: &mut World) -> Self {
                ($($name::init(_world),)*)
            }

            #[inline]
            fn apply(&mut self, _world: &mut World) {
                let ($($name,)*) = self;
                $($name.apply(_world);)*
            }
        }

        #[allow(non_snake_case)]
        impl<$($name: ReadOnlySystemParam),*> ReadOnlySystemParam for ($($name,)*) {}
    };
}

// Implement for tuples up to 16 elements
// Note: () is implemented separately above, so we start with single-element tuples
impl_system_param_tuple!(A);
impl_system_param_tuple!(A, B);
impl_system_param_tuple!(A, B, C);
impl_system_param_tuple!(A, B, C, D);
impl_system_param_tuple!(A, B, C, D, E);
impl_system_param_tuple!(A, B, C, D, E, F);
impl_system_param_tuple!(A, B, C, D, E, F, G);
impl_system_param_tuple!(A, B, C, D, E, F, G, H);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_system_param_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// =============================================================================
// StaticSystemParam
// =============================================================================

/// A system parameter that provides access to its inner state directly.
///
/// `StaticSystemParam` is useful when you want to store data in the parameter
/// state and access it during system execution. Unlike `Local<T>`, this
/// stores the data in the system's state rather than per-system instance.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::World;
/// use goud_engine::ecs::query::Access;
/// use goud_engine::ecs::system::{SystemParam, SystemParamState, StaticSystemParam};
///
/// // State that tracks how many times the system ran
/// #[derive(Default)]
/// struct RunCountState {
///     count: u32,
/// }
///
/// impl SystemParamState for RunCountState {
///     fn init(_world: &mut World) -> Self {
///         Self::default()
///     }
/// }
///
/// // The parameter type
/// type RunCount = StaticSystemParam<RunCountState>;
/// ```
#[derive(Debug)]
pub struct StaticSystemParam<S: SystemParamState> {
    _marker: std::marker::PhantomData<fn() -> S>,
}

/// State for StaticSystemParam - just wraps the inner state
impl<S: SystemParamState> SystemParamState for StaticSystemParamState<S> {
    #[inline]
    fn init(world: &mut World) -> Self {
        Self {
            inner: S::init(world),
        }
    }

    #[inline]
    fn apply(&mut self, world: &mut World) {
        self.inner.apply(world);
    }
}

/// State wrapper for StaticSystemParam
#[derive(Debug)]
pub struct StaticSystemParamState<S> {
    inner: S,
}

impl<S> StaticSystemParamState<S> {
    /// Returns a reference to the inner state.
    #[inline]
    pub fn get(&self) -> &S {
        &self.inner
    }

    /// Returns a mutable reference to the inner state.
    #[inline]
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: SystemParamState + 'static> SystemParam for StaticSystemParam<S> {
    type State = StaticSystemParamState<S>;
    type Item<'w, 's> = &'s mut S;

    #[inline]
    fn update_access(_state: &Self::State, _access: &mut Access) {
        // Static state doesn't access world components
    }

    #[inline]
    fn get_param<'w, 's>(state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        state.get_mut()
    }
}

// =============================================================================
// ParamSet
// =============================================================================

/// A parameter that allows disjoint access to multiple queries with conflicting access.
///
/// Normally, two queries that access the same component cannot be used together
/// because of Rust's aliasing rules. `ParamSet` solves this by ensuring only one
/// query can be accessed at a time.
///
/// # Example
///
/// ```ignore
/// // Future: When Query is implemented
/// fn conflicting_access(
///     // These would conflict: both access Position
///     // query1: Query<&mut Position>,
///     // query2: Query<&Position, With<Player>>,
///
///     // Solution: use ParamSet
///     mut set: ParamSet<(Query<&mut Position>, Query<&Position, With<Player>>)>,
/// ) {
///     // Access one at a time
///     for pos in set.p0().iter_mut() {
///         pos.x += 1.0;
///     }
///     // p0 is dropped before p1 is accessed
///     for pos in set.p1().iter() {
///         println!("Player position: {:?}", pos);
///     }
/// }
/// ```
///
/// # Note
///
/// This is a placeholder for the full ParamSet implementation, which requires
/// Query to be implemented first (Step 3.1.3).
#[derive(Debug)]
pub struct ParamSet<T> {
    _marker: std::marker::PhantomData<T>,
}

// ParamSet implementations will be added when Query is implemented (Step 3.1.3)

// =============================================================================
// Res<T> - Immutable Resource SystemParam
// =============================================================================

use crate::ecs::resource::{Res, ResMut, Resource, ResourceId};

/// State for `Res<T>` system parameter.
///
/// Caches the resource ID for efficient access and conflict detection.
#[derive(Debug, Clone)]
pub struct ResState<T: Resource> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Resource> SystemParamState for ResState<T> {
    fn init(_world: &mut World) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// `Res<T>` as a system parameter for immutable resource access.
///
/// # Example
///
/// ```ignore
/// fn print_time(time: Res<Time>) {
///     println!("Delta: {}", time.delta);
/// }
/// ```
///
/// # Panics
///
/// When used as a system parameter, `Res<T>` will panic if the resource
/// does not exist in the world. Use `Option<Res<T>>` for optional access.
impl<T: Resource> SystemParam for Res<'_, T> {
    type State = ResState<T>;
    type Item<'w, 's> = Res<'w, T>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        // Register read access to the resource
        access.add_resource_read(ResourceId::of::<T>());
    }

    fn get_param<'w, 's>(_state: &'s mut Self::State, world: &'w World) -> Self::Item<'w, 's> {
        world
            .resource::<T>()
            .expect("Resource does not exist. Use Option<Res<T>> for optional access.")
    }

    fn get_param_mut<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        // Immutable resource access, so we just use get_param
        Self::get_param(state, world)
    }
}

/// `Res<T>` is read-only.
impl<T: Resource> ReadOnlySystemParam for Res<'_, T> {}

// =============================================================================
// ResMut<T> - Mutable Resource SystemParam
// =============================================================================

/// State for `ResMut<T>` system parameter.
///
/// Caches the resource ID for efficient access and conflict detection.
#[derive(Debug, Clone)]
pub struct ResMutState<T: Resource> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Resource> SystemParamState for ResMutState<T> {
    fn init(_world: &mut World) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// `ResMut<T>` as a system parameter for mutable resource access.
///
/// # Example
///
/// ```ignore
/// fn update_time(mut time: ResMut<Time>) {
///     time.total += time.delta;
/// }
/// ```
///
/// # Panics
///
/// When used as a system parameter, `ResMut<T>` will panic if the resource
/// does not exist in the world. Use `Option<ResMut<T>>` for optional access.
impl<T: Resource> SystemParam for ResMut<'_, T> {
    type State = ResMutState<T>;
    type Item<'w, 's> = ResMut<'w, T>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        // Register write access to the resource
        access.add_resource_write(ResourceId::of::<T>());
    }

    fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        // ResMut requires mutable world access, so this panics
        panic!("ResMut<T> requires mutable world access. Use get_param_mut instead.")
    }

    fn get_param_mut<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        world
            .resource_mut::<T>()
            .expect("Resource does not exist. Use Option<ResMut<T>> for optional access.")
    }
}

// ResMut is NOT ReadOnlySystemParam - intentionally omitted

// =============================================================================
// NonSend<T> - Immutable Non-Send Resource SystemParam
// =============================================================================

use crate::ecs::resource::{NonSend, NonSendMut, NonSendResource, NonSendResourceId};

/// State for `NonSend<T>` system parameter.
///
/// Caches the non-send resource ID for efficient access and conflict detection.
///
/// Note: Uses `PhantomData<fn() -> T>` to be Send+Sync regardless of T's bounds.
/// This is safe because the state only stores type information for conflict detection,
/// not the actual resource data.
#[derive(Debug, Clone)]
pub struct NonSendState<T: NonSendResource> {
    // Use fn() -> T to be Send + Sync regardless of T's bounds
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: NonSendResource> SystemParamState for NonSendState<T> {
    fn init(_world: &mut World) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// `NonSend<T>` as a system parameter for immutable non-send resource access.
///
/// # Thread Safety
///
/// Systems using `NonSend<T>` are constrained to run on the main thread.
/// This is enforced by the scheduler.
///
/// # Example
///
/// ```ignore
/// fn print_window(window: NonSend<WindowHandle>) {
///     println!("Window ID: {}", window.id);
/// }
/// ```
///
/// # Panics
///
/// When used as a system parameter, `NonSend<T>` will panic if the non-send
/// resource does not exist in the world.
impl<T: NonSendResource> SystemParam for NonSend<'_, T> {
    type State = NonSendState<T>;
    type Item<'w, 's> = NonSend<'w, T>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        // Register read access to the non-send resource
        access.add_non_send_read(NonSendResourceId::of::<T>());
    }

    fn get_param<'w, 's>(_state: &'s mut Self::State, world: &'w World) -> Self::Item<'w, 's> {
        world
            .non_send_resource::<T>()
            .expect("Non-send resource does not exist. Use Option<NonSend<T>> for optional access.")
    }

    fn get_param_mut<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        // Immutable non-send resource access, so we just use get_param
        Self::get_param(state, world)
    }
}

/// `NonSend<T>` is read-only.
impl<T: NonSendResource> ReadOnlySystemParam for NonSend<'_, T> {}

// =============================================================================
// NonSendMut<T> - Mutable Non-Send Resource SystemParam
// =============================================================================

/// State for `NonSendMut<T>` system parameter.
///
/// Caches the non-send resource ID for efficient access and conflict detection.
///
/// Note: Uses `PhantomData<fn() -> T>` to be Send+Sync regardless of T's bounds.
/// This is safe because the state only stores type information for conflict detection,
/// not the actual resource data.
#[derive(Debug, Clone)]
pub struct NonSendMutState<T: NonSendResource> {
    // Use fn() -> T to be Send + Sync regardless of T's bounds
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: NonSendResource> SystemParamState for NonSendMutState<T> {
    fn init(_world: &mut World) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// `NonSendMut<T>` as a system parameter for mutable non-send resource access.
///
/// # Thread Safety
///
/// Systems using `NonSendMut<T>` are constrained to run on the main thread.
/// This is enforced by the scheduler.
///
/// # Example
///
/// ```ignore
/// fn update_window(mut window: NonSendMut<WindowHandle>) {
///     window.update_title("New Title");
/// }
/// ```
///
/// # Panics
///
/// When used as a system parameter, `NonSendMut<T>` will panic if the non-send
/// resource does not exist in the world.
impl<T: NonSendResource> SystemParam for NonSendMut<'_, T> {
    type State = NonSendMutState<T>;
    type Item<'w, 's> = NonSendMut<'w, T>;

    fn update_access(_state: &Self::State, access: &mut Access) {
        // Register write access to the non-send resource
        access.add_non_send_write(NonSendResourceId::of::<T>());
    }

    fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
        // NonSendMut requires mutable world access, so this panics
        panic!("NonSendMut<T> requires mutable world access. Use get_param_mut instead.")
    }

    fn get_param_mut<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut World,
    ) -> Self::Item<'w, 's> {
        world.non_send_resource_mut::<T>().expect(
            "Non-send resource does not exist. Use Option<NonSendMut<T>> for optional access.",
        )
    }
}

// NonSendMut is NOT ReadOnlySystemParam - intentionally omitted

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
    // SystemParamState Tests
    // =========================================================================

    mod system_param_state {
        use super::*;

        #[test]
        fn test_unit_state_init() {
            let mut world = World::new();
            let _state: () = <()>::init(&mut world);
            // Unit state is just ()
        }

        #[test]
        fn test_unit_state_apply() {
            let mut world = World::new();
            let mut state: () = <()>::init(&mut world);
            state.apply(&mut world);
            // Should not panic
        }

        #[test]
        fn test_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<()>();
        }

        #[test]
        fn test_tuple_state_init() {
            let mut world = World::new();
            let _state: ((), ()) = <((), ())>::init(&mut world);
        }

        #[test]
        fn test_tuple_state_apply() {
            let mut world = World::new();
            let mut state: ((), (), ()) = <((), (), ())>::init(&mut world);
            state.apply(&mut world);
        }
    }

    // =========================================================================
    // SystemParam Tests
    // =========================================================================

    mod system_param {
        use super::*;

        #[test]
        fn test_unit_param_update_access() {
            let state = ();
            let mut access = Access::new();
            <()>::update_access(&state, &mut access);

            // Unit param has no access
            assert!(access.is_read_only());
            assert_eq!(access.writes().len(), 0);
        }

        #[test]
        fn test_unit_param_get_param() {
            let world = World::new();
            let mut state = ();
            let _result: () = <()>::get_param(&mut state, &world);
        }

        #[test]
        fn test_unit_param_get_param_mut() {
            let mut world = World::new();
            let mut state = ();
            let _result: () = <()>::get_param_mut(&mut state, &mut world);
        }

        #[test]
        fn test_unit_implements_system_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<()>();
        }

        #[test]
        fn test_unit_implements_read_only() {
            fn requires_read_only<T: ReadOnlySystemParam>() {}
            requires_read_only::<()>();
        }
    }

    // =========================================================================
    // Tuple SystemParam Tests
    // =========================================================================

    mod tuple_param {
        use super::*;

        #[test]
        fn test_single_tuple_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<((),)>();
        }

        #[test]
        fn test_double_tuple_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<((), ())>();
        }

        #[test]
        fn test_triple_tuple_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<((), (), ())>();
        }

        #[test]
        fn test_tuple_get_param() {
            let world = World::new();
            let mut state: ((), ()) = ((), ());
            let (a, b): ((), ()) = <((), ())>::get_param(&mut state, &world);
            // Both should be ()
            assert_eq!(a, ());
            assert_eq!(b, ());
        }

        #[test]
        fn test_tuple_update_access() {
            let state: ((), ()) = ((), ());
            let mut access = Access::new();
            <((), ())>::update_access(&state, &mut access);

            // Empty tuples have no access
            assert!(access.is_read_only());
        }

        #[test]
        fn test_nested_tuple_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<(((), ()), ())>();
        }

        #[test]
        fn test_large_tuple_param() {
            fn requires_system_param<T: SystemParam>() {}
            // Test up to 16 elements
            requires_system_param::<(
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
                (),
            )>();
        }

        #[test]
        fn test_tuple_read_only() {
            fn requires_read_only<T: ReadOnlySystemParam>() {}
            requires_read_only::<((), ())>();
            requires_read_only::<((), (), ())>();
        }
    }

    // =========================================================================
    // Custom SystemParam Tests
    // =========================================================================

    mod custom_param {
        use super::*;

        // A custom system parameter that provides entity count
        struct EntityCount(usize);

        struct EntityCountState;

        impl SystemParamState for EntityCountState {
            fn init(_world: &mut World) -> Self {
                EntityCountState
            }
        }

        impl SystemParam for EntityCount {
            type State = EntityCountState;
            type Item<'w, 's> = EntityCount;

            fn update_access(_state: &Self::State, _access: &mut Access) {
                // No component access needed
            }

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                world: &'w World,
            ) -> Self::Item<'w, 's> {
                EntityCount(world.entity_count())
            }
        }

        impl ReadOnlySystemParam for EntityCount {}

        #[test]
        fn test_custom_param_implements_trait() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<EntityCount>();
        }

        #[test]
        fn test_custom_param_state_init() {
            let mut world = World::new();
            let _state = EntityCountState::init(&mut world);
        }

        #[test]
        fn test_custom_param_get_param() {
            let mut world = World::new();
            world.spawn_empty();
            world.spawn_empty();
            world.spawn_empty();

            let mut state = EntityCountState;
            let count = EntityCount::get_param(&mut state, &world);

            assert_eq!(count.0, 3);
        }

        #[test]
        fn test_custom_param_access() {
            let state = EntityCountState;
            let mut access = Access::new();
            EntityCount::update_access(&state, &mut access);

            // EntityCount has no component access
            assert!(access.is_read_only());
        }

        #[test]
        fn test_custom_param_read_only() {
            fn requires_read_only<T: ReadOnlySystemParam>() {}
            requires_read_only::<EntityCount>();
        }
    }

    // =========================================================================
    // Custom Param with Component Access Tests
    // =========================================================================

    mod param_with_access {
        use super::*;

        // A custom parameter that reads Position
        struct PositionReader;

        struct PositionReaderState {
            component_id: ComponentId,
        }

        impl SystemParamState for PositionReaderState {
            fn init(_world: &mut World) -> Self {
                Self {
                    component_id: ComponentId::of::<Position>(),
                }
            }
        }

        impl SystemParam for PositionReader {
            type State = PositionReaderState;
            type Item<'w, 's> = PositionReader;

            fn update_access(state: &Self::State, access: &mut Access) {
                access.add_read(state.component_id);
            }

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                PositionReader
            }
        }

        impl ReadOnlySystemParam for PositionReader {}

        // A custom parameter that writes Position
        struct PositionWriter;

        struct PositionWriterState {
            component_id: ComponentId,
        }

        impl SystemParamState for PositionWriterState {
            fn init(_world: &mut World) -> Self {
                Self {
                    component_id: ComponentId::of::<Position>(),
                }
            }
        }

        impl SystemParam for PositionWriter {
            type State = PositionWriterState;
            type Item<'w, 's> = PositionWriter;

            fn update_access(state: &Self::State, access: &mut Access) {
                access.add_write(state.component_id);
            }

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                PositionWriter
            }
        }

        // Note: PositionWriter is NOT ReadOnlySystemParam

        #[test]
        fn test_reader_is_read_only() {
            let mut world = World::new();
            let state = PositionReaderState::init(&mut world);
            let mut access = Access::new();
            PositionReader::update_access(&state, &mut access);

            assert!(access.is_read_only());
        }

        #[test]
        fn test_writer_is_not_read_only() {
            let mut world = World::new();
            let state = PositionWriterState::init(&mut world);
            let mut access = Access::new();
            PositionWriter::update_access(&state, &mut access);

            assert!(!access.is_read_only());
        }

        #[test]
        fn test_reader_writer_conflict() {
            let mut world = World::new();

            let reader_state = PositionReaderState::init(&mut world);
            let mut reader_access = Access::new();
            PositionReader::update_access(&reader_state, &mut reader_access);

            let writer_state = PositionWriterState::init(&mut world);
            let mut writer_access = Access::new();
            PositionWriter::update_access(&writer_state, &mut writer_access);

            // Reader and writer of same component conflict
            assert!(reader_access.conflicts_with(&writer_access));
            assert!(writer_access.conflicts_with(&reader_access));
        }

        #[test]
        fn test_readers_dont_conflict() {
            let mut world = World::new();

            let state1 = PositionReaderState::init(&mut world);
            let mut access1 = Access::new();
            PositionReader::update_access(&state1, &mut access1);

            let state2 = PositionReaderState::init(&mut world);
            let mut access2 = Access::new();
            PositionReader::update_access(&state2, &mut access2);

            // Two readers don't conflict
            assert!(!access1.conflicts_with(&access2));
        }
    }

    // =========================================================================
    // StaticSystemParam Tests
    // =========================================================================

    mod static_param {
        use super::*;

        #[derive(Debug, Default)]
        struct CounterState {
            count: u32,
        }

        impl SystemParamState for CounterState {
            fn init(_world: &mut World) -> Self {
                Self::default()
            }
        }

        #[test]
        fn test_static_param_state_init() {
            let mut world = World::new();
            let state: StaticSystemParamState<CounterState> =
                StaticSystemParamState::<CounterState>::init(&mut world);

            assert_eq!(state.get().count, 0);
        }

        #[test]
        fn test_static_param_state_get() {
            let mut world = World::new();
            let state: StaticSystemParamState<CounterState> =
                StaticSystemParamState::<CounterState>::init(&mut world);

            assert_eq!(state.get().count, 0);
        }

        #[test]
        fn test_static_param_state_get_mut() {
            let mut world = World::new();
            let mut state: StaticSystemParamState<CounterState> =
                StaticSystemParamState::<CounterState>::init(&mut world);

            state.get_mut().count = 42;
            assert_eq!(state.get().count, 42);
        }

        #[test]
        fn test_static_param_get_param() {
            let mut world = World::new();
            let mut state: StaticSystemParamState<CounterState> =
                StaticSystemParamState::<CounterState>::init(&mut world);

            let counter: &mut CounterState =
                StaticSystemParam::<CounterState>::get_param(&mut state, &world);

            counter.count = 100;
            assert_eq!(state.get().count, 100);
        }

        #[test]
        fn test_static_param_no_access() {
            let mut world = World::new();
            let state: StaticSystemParamState<CounterState> =
                StaticSystemParamState::<CounterState>::init(&mut world);

            let mut access = Access::new();
            StaticSystemParam::<CounterState>::update_access(&state, &mut access);

            // Static state doesn't access world components
            assert!(access.is_read_only());
            assert_eq!(access.writes().len(), 0);
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_param_with_entity_spawn() {
            // Custom param that spawns entities
            struct EntitySpawner;

            struct EntitySpawnerState;

            impl SystemParamState for EntitySpawnerState {
                fn init(_world: &mut World) -> Self {
                    EntitySpawnerState
                }
            }

            impl SystemParam for EntitySpawner {
                type State = EntitySpawnerState;
                type Item<'w, 's> = EntitySpawner;

                fn update_access(_state: &Self::State, _access: &mut Access) {}

                fn get_param<'w, 's>(
                    _state: &'s mut Self::State,
                    _world: &'w World,
                ) -> Self::Item<'w, 's> {
                    EntitySpawner
                }
            }

            let mut world = World::new();
            let mut state = EntitySpawnerState::init(&mut world);

            // Get param multiple times
            let _spawner1 = EntitySpawner::get_param(&mut state, &world);
            let _spawner2 = EntitySpawner::get_param(&mut state, &world);
        }

        #[test]
        fn test_combined_access_tracking() {
            // Test that tuple params combine access correctly
            struct ReaderA;
            struct WriterB;

            struct ReaderAState(ComponentId);
            struct WriterBState(ComponentId);

            impl SystemParamState for ReaderAState {
                fn init(_world: &mut World) -> Self {
                    Self(ComponentId::of::<Position>())
                }
            }

            impl SystemParamState for WriterBState {
                fn init(_world: &mut World) -> Self {
                    Self(ComponentId::of::<Velocity>())
                }
            }

            impl SystemParam for ReaderA {
                type State = ReaderAState;
                type Item<'w, 's> = ReaderA;

                fn update_access(state: &Self::State, access: &mut Access) {
                    access.add_read(state.0);
                }

                fn get_param<'w, 's>(
                    _state: &'s mut Self::State,
                    _world: &'w World,
                ) -> Self::Item<'w, 's> {
                    ReaderA
                }
            }

            impl SystemParam for WriterB {
                type State = WriterBState;
                type Item<'w, 's> = WriterB;

                fn update_access(state: &Self::State, access: &mut Access) {
                    access.add_write(state.0);
                }

                fn get_param<'w, 's>(
                    _state: &'s mut Self::State,
                    _world: &'w World,
                ) -> Self::Item<'w, 's> {
                    WriterB
                }
            }

            let mut world = World::new();
            let state: (ReaderAState, WriterBState) =
                <(ReaderAState, WriterBState)>::init(&mut world);

            let mut access = Access::new();
            <(ReaderA, WriterB)>::update_access(&state, &mut access);

            // Combined access should have Position read, Velocity write
            assert!(!access.is_read_only()); // Has a write
            assert!(access.writes().contains(&ComponentId::of::<Velocity>()));
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_system_param_state_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<()>();
        }

        #[test]
        fn test_system_param_state_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<()>();
        }

        #[test]
        fn test_tuple_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<((), ())>();
            requires_send_sync::<((), (), ())>();
        }

        #[test]
        fn test_static_param_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}

            #[derive(Debug)]
            struct SendSyncState;
            impl SystemParamState for SendSyncState {
                fn init(_world: &mut World) -> Self {
                    SendSyncState
                }
            }

            requires_send_sync::<StaticSystemParamState<SendSyncState>>();
        }
    }

    // =========================================================================
    // Res<T> SystemParam Tests
    // =========================================================================

    mod res_param {
        use super::*;
        use crate::ecs::resource::{Res, ResourceId};

        // Test resource
        #[derive(Debug)]
        struct Time {
            delta: f32,
            total: f32,
        }

        #[derive(Debug)]
        struct Score(u32);

        #[test]
        fn test_res_state_init() {
            let mut world = World::new();
            let _state: ResState<Time> = ResState::init(&mut world);
        }

        #[test]
        fn test_res_update_access() {
            let mut world = World::new();
            let state: ResState<Time> = ResState::init(&mut world);

            let mut access = Access::new();
            Res::<Time>::update_access(&state, &mut access);

            // Should have read access to the resource
            assert!(access
                .resource_reads()
                .any(|&id| id == ResourceId::of::<Time>()));
            assert!(access.is_read_only());
        }

        #[test]
        fn test_res_get_param() {
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 1.0,
            });

            let mut state: ResState<Time> = ResState::init(&mut world);
            let time: Res<Time> = Res::get_param(&mut state, &world);

            assert_eq!(time.delta, 0.016);
            assert_eq!(time.total, 1.0);
        }

        #[test]
        fn test_res_get_param_mut() {
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 1.0,
            });

            let mut state: ResState<Time> = ResState::init(&mut world);
            let time: Res<Time> = Res::get_param_mut(&mut state, &mut world);

            assert_eq!(time.delta, 0.016);
        }

        #[test]
        #[should_panic(expected = "Resource does not exist")]
        fn test_res_get_param_missing_resource() {
            let mut world = World::new();
            // Don't insert the resource

            let mut state: ResState<Time> = ResState::init(&mut world);
            let _time: Res<Time> = Res::get_param(&mut state, &world);
        }

        #[test]
        fn test_res_implements_system_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<Res<Time>>();
            requires_system_param::<Res<Score>>();
        }

        #[test]
        fn test_res_implements_read_only() {
            fn requires_read_only<T: ReadOnlySystemParam>() {}
            requires_read_only::<Res<Time>>();
            requires_read_only::<Res<Score>>();
        }

        #[test]
        fn test_res_multiple_resources() {
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 0.0,
            });
            world.insert_resource(Score(100));

            let mut time_state: ResState<Time> = ResState::init(&mut world);
            let mut score_state: ResState<Score> = ResState::init(&mut world);

            let time: Res<Time> = Res::get_param(&mut time_state, &world);
            let score: Res<Score> = Res::get_param(&mut score_state, &world);

            assert_eq!(time.delta, 0.016);
            assert_eq!(score.0, 100);
        }

        #[test]
        fn test_res_access_no_conflict() {
            let mut world = World::new();

            let state1: ResState<Time> = ResState::init(&mut world);
            let state2: ResState<Time> = ResState::init(&mut world);

            let mut access1 = Access::new();
            Res::<Time>::update_access(&state1, &mut access1);

            let mut access2 = Access::new();
            Res::<Time>::update_access(&state2, &mut access2);

            // Two reads of the same resource don't conflict
            assert!(!access1.conflicts_with(&access2));
        }
    }

    // =========================================================================
    // ResMut<T> SystemParam Tests
    // =========================================================================

    mod res_mut_param {
        use super::*;
        use crate::ecs::resource::{ResMut, ResourceId};

        // Test resource
        #[derive(Debug)]
        struct Time {
            delta: f32,
            total: f32,
        }

        #[derive(Debug)]
        struct Score(u32);

        #[test]
        fn test_res_mut_state_init() {
            let mut world = World::new();
            let _state: ResMutState<Time> = ResMutState::init(&mut world);
        }

        #[test]
        fn test_res_mut_update_access() {
            let mut world = World::new();
            let state: ResMutState<Time> = ResMutState::init(&mut world);

            let mut access = Access::new();
            ResMut::<Time>::update_access(&state, &mut access);

            // Should have write access to the resource
            assert!(access.resource_writes().contains(&ResourceId::of::<Time>()));
            assert!(!access.is_read_only());
        }

        #[test]
        fn test_res_mut_get_param_mut() {
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 1.0,
            });

            let mut state: ResMutState<Time> = ResMutState::init(&mut world);
            let mut time: ResMut<Time> = ResMut::get_param_mut(&mut state, &mut world);

            assert_eq!(time.delta, 0.016);
            time.total += time.delta;
            assert_eq!(time.total, 1.016);
        }

        #[test]
        #[should_panic(expected = "ResMut<T> requires mutable world access")]
        fn test_res_mut_get_param_panics() {
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 1.0,
            });

            let mut state: ResMutState<Time> = ResMutState::init(&mut world);
            // This should panic - ResMut requires get_param_mut
            let _time: ResMut<Time> = ResMut::get_param(&mut state, &world);
        }

        #[test]
        #[should_panic(expected = "Resource does not exist")]
        fn test_res_mut_get_param_mut_missing_resource() {
            let mut world = World::new();
            // Don't insert the resource

            let mut state: ResMutState<Time> = ResMutState::init(&mut world);
            let _time: ResMut<Time> = ResMut::get_param_mut(&mut state, &mut world);
        }

        #[test]
        fn test_res_mut_implements_system_param() {
            fn requires_system_param<T: SystemParam>() {}
            requires_system_param::<ResMut<Time>>();
            requires_system_param::<ResMut<Score>>();
        }

        #[test]
        fn test_res_mut_not_read_only() {
            // ResMut should NOT implement ReadOnlySystemParam
            // We can't directly test for "does not implement", but we verify
            // through the access pattern
            let mut world = World::new();
            let state: ResMutState<Time> = ResMutState::init(&mut world);

            let mut access = Access::new();
            ResMut::<Time>::update_access(&state, &mut access);

            assert!(!access.is_read_only());
        }

        #[test]
        fn test_res_mut_modify_resource() {
            let mut world = World::new();
            world.insert_resource(Score(100));

            {
                let mut state: ResMutState<Score> = ResMutState::init(&mut world);
                let mut score: ResMut<Score> = ResMut::get_param_mut(&mut state, &mut world);
                score.0 += 50;
            }

            // Verify the change persisted
            assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
        }
    }

    // =========================================================================
    // Res/ResMut Conflict Tests
    // =========================================================================

    mod res_conflict {
        use super::*;
        use crate::ecs::resource::{Res, ResMut, ResourceId};

        #[derive(Debug)]
        struct Time {
            delta: f32,
        }

        #[derive(Debug)]
        struct Score(u32);

        #[test]
        fn test_res_res_no_conflict() {
            let mut world = World::new();

            let state1: ResState<Time> = ResState::init(&mut world);
            let state2: ResState<Time> = ResState::init(&mut world);

            let mut access1 = Access::new();
            Res::<Time>::update_access(&state1, &mut access1);

            let mut access2 = Access::new();
            Res::<Time>::update_access(&state2, &mut access2);

            // Two reads don't conflict
            assert!(!access1.conflicts_with(&access2));
        }

        #[test]
        fn test_res_res_mut_conflict() {
            let mut world = World::new();

            let res_state: ResState<Time> = ResState::init(&mut world);
            let res_mut_state: ResMutState<Time> = ResMutState::init(&mut world);

            let mut read_access = Access::new();
            Res::<Time>::update_access(&res_state, &mut read_access);

            let mut write_access = Access::new();
            ResMut::<Time>::update_access(&res_mut_state, &mut write_access);

            // Read and write conflict
            assert!(read_access.conflicts_with(&write_access));
            assert!(write_access.conflicts_with(&read_access));
        }

        #[test]
        fn test_res_mut_res_mut_conflict() {
            let mut world = World::new();

            let state1: ResMutState<Time> = ResMutState::init(&mut world);
            let state2: ResMutState<Time> = ResMutState::init(&mut world);

            let mut access1 = Access::new();
            ResMut::<Time>::update_access(&state1, &mut access1);

            let mut access2 = Access::new();
            ResMut::<Time>::update_access(&state2, &mut access2);

            // Two writes conflict
            assert!(access1.conflicts_with(&access2));
        }

        #[test]
        fn test_different_resources_no_conflict() {
            let mut world = World::new();

            let time_state: ResState<Time> = ResState::init(&mut world);
            let score_state: ResMutState<Score> = ResMutState::init(&mut world);

            let mut time_access = Access::new();
            Res::<Time>::update_access(&time_state, &mut time_access);

            let mut score_access = Access::new();
            ResMut::<Score>::update_access(&score_state, &mut score_access);

            // Different resources don't conflict
            assert!(!time_access.conflicts_with(&score_access));
        }

        #[test]
        fn test_combined_access() {
            let mut world = World::new();

            let time_state: ResState<Time> = ResState::init(&mut world);
            let score_state: ResMutState<Score> = ResMutState::init(&mut world);

            let mut access = Access::new();
            Res::<Time>::update_access(&time_state, &mut access);
            ResMut::<Score>::update_access(&score_state, &mut access);

            // Should have read on Time, write on Score
            assert!(access
                .resource_reads()
                .any(|&id| id == ResourceId::of::<Time>()));
            assert!(access
                .resource_writes()
                .contains(&ResourceId::of::<Score>()));
            assert!(!access.is_read_only()); // Has a write
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod res_integration {
        use super::*;
        use crate::ecs::resource::ResMut;

        #[derive(Debug, Clone)]
        struct Time {
            delta: f32,
            total: f32,
        }

        #[derive(Debug, Clone)]
        struct Score(u32);

        #[test]
        fn test_res_with_component_query() {
            // This test verifies Res can coexist with component queries
            let mut world = World::new();
            world.insert_resource(Time {
                delta: 0.016,
                total: 0.0,
            });

            // Spawn some entities
            let e = world.spawn_empty();
            world.insert(e, Position { x: 0.0, y: 0.0 });

            // Access both resource and component
            let time = world.get_resource::<Time>().unwrap();
            let pos = world.get::<Position>(e).unwrap();

            assert_eq!(time.delta, 0.016);
            assert_eq!(pos.x, 0.0);
        }

        #[test]
        fn test_res_mut_modifies_world() {
            let mut world = World::new();
            world.insert_resource(Score(0));

            // Simulate multiple system runs
            for _ in 0..10 {
                let mut state: ResMutState<Score> = ResMutState::init(&mut world);
                let mut score = ResMut::get_param_mut(&mut state, &mut world);
                score.0 += 10;
            }

            assert_eq!(world.get_resource::<Score>().unwrap().0, 100);
        }

        #[test]
        fn test_res_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<ResState<Time>>();
            requires_send_sync::<ResMutState<Score>>();
        }

        #[test]
        fn test_res_state_is_clone() {
            let mut world = World::new();
            let state: ResState<Time> = ResState::init(&mut world);
            let _cloned = state.clone();
        }
    }
}
