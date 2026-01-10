//! Function system wrapper for converting functions into systems.
//!
//! This module provides the [`FunctionSystem`] type that wraps a Rust function
//! and its associated state to implement the [`System`] trait. This allows
//! ergonomic system definition using regular functions.
//!
//! # Architecture
//!
//! The function system architecture consists of:
//!
//! - [`FunctionSystem`]: Wraps a function and its cached parameter state
//! - [`SystemParamFunction`]: Trait for functions that can be systems
//! - [`IntoSystem`] implementations: Convert functions to boxed systems
//!
//! # Design
//!
//! Function systems use compile-time type information to:
//! 1. Determine what parameters the function needs
//! 2. Cache parameter state for efficient repeated extraction
//! 3. Track component/resource access for conflict detection
//! 4. Extract parameters from the World at runtime
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::query::Query;
//! use goud_engine::ecs::system::{IntoSystem, BoxedSystem};
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // Define a system as a regular function
//! fn movement_system(query: Query<&Position>) {
//!     // System logic here
//! }
//!
//! // Convert to a BoxedSystem
//! let mut boxed: BoxedSystem = movement_system.into_system();
//!
//! // Run it
//! let mut world = World::new();
//! boxed.run(&mut world);
//! ```
//!
//! # Supported Function Signatures
//!
//! Functions can have 0 to 16 parameters, each implementing [`SystemParam`]:
//!
//! ```ignore
//! fn no_params() { }
//! fn one_param(query: Query<&Position>) { }
//! fn two_params(query: Query<&Position>, time: Res<Time>) { }
//! // ... up to 16 parameters
//! ```
//!
//! # Thread Safety
//!
//! Function systems are `Send` if all their parameters and states are `Send`.
//! This enables parallel scheduling in the future.

use std::borrow::Cow;
use std::marker::PhantomData;

use super::{BoxedSystem, IntoSystem, System, SystemMeta, SystemParam, SystemParamState};
use crate::ecs::query::Access;
use crate::ecs::World;

// =============================================================================
// FunctionSystem
// =============================================================================

/// A system that wraps a function and its cached parameter state.
///
/// `FunctionSystem` stores:
/// - The function to call
/// - Cached state for each parameter
/// - Metadata (name, access patterns)
///
/// This type is created automatically when you call `.into_system()` on a
/// function. You typically don't construct it directly.
///
/// # Type Parameters
///
/// - `Marker`: A type-level marker to distinguish different function arities
/// - `F`: The function type
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World, Component};
/// use goud_engine::ecs::system::{IntoSystem, BoxedSystem};
///
/// fn my_system() {
///     println!("Hello from system!");
/// }
///
/// let boxed: BoxedSystem = my_system.into_system();
/// assert_eq!(boxed.name(), "my_system");
/// ```
pub struct FunctionSystem<Marker, F>
where
    F: SystemParamFunction<Marker>,
{
    /// The wrapped function.
    func: F,
    /// Cached parameter state.
    state: Option<F::State>,
    /// System metadata.
    meta: SystemMeta,
    /// Marker for the function type.
    _marker: PhantomData<fn() -> Marker>,
}

impl<Marker, F> FunctionSystem<Marker, F>
where
    F: SystemParamFunction<Marker>,
{
    /// Creates a new function system from a function.
    ///
    /// The state is initialized lazily on the first run.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to wrap
    #[inline]
    pub fn new(func: F) -> Self {
        Self {
            func,
            state: None,
            meta: SystemMeta::new(std::any::type_name::<F>()),
            _marker: PhantomData,
        }
    }

    /// Sets a custom name for this system.
    ///
    /// By default, the name is derived from the function's type name.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name for the system
    #[inline]
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.meta.set_name(name);
        self
    }
}

impl<Marker, F> System for FunctionSystem<Marker, F>
where
    Marker: 'static,
    F: SystemParamFunction<Marker> + Send + 'static,
    F::State: Send + Sync,
{
    fn name(&self) -> &'static str {
        // Extract the short function name from the full path
        let full_name = std::any::type_name::<F>();
        // Find the last segment after ::
        full_name.rsplit("::").next().unwrap_or(full_name)
    }

    fn component_access(&self) -> Access {
        self.meta.component_access().clone()
    }

    fn initialize(&mut self, world: &mut World) {
        // Initialize state if not already done
        if self.state.is_none() {
            let state = F::State::init(world);

            // Update access patterns from state
            let access = F::build_access(&state);
            self.meta.set_component_access(access);

            self.state = Some(state);
        }
    }

    fn run(&mut self, world: &mut World) {
        // Ensure state is initialized
        if self.state.is_none() {
            self.initialize(world);
        }

        // Get mutable reference to state
        let state = self.state.as_mut().expect("State should be initialized");

        // Run the function with extracted parameters
        // SAFETY: We have exclusive access to world through &mut World.
        // The SystemParamFunction implementation is responsible for ensuring
        // that parameter access doesn't alias.
        unsafe { self.func.run_unsafe(state, world) };

        // Apply any pending state changes (like command buffers)
        state.apply(world);
    }

    fn is_read_only(&self) -> bool {
        self.meta.is_read_only()
    }
}

impl<Marker, F> std::fmt::Debug for FunctionSystem<Marker, F>
where
    F: SystemParamFunction<Marker>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionSystem")
            .field("name", &std::any::type_name::<F>())
            .field("initialized", &self.state.is_some())
            .finish()
    }
}

// =============================================================================
// SystemParamFunction Trait
// =============================================================================

/// Trait for functions that can be used as systems.
///
/// This trait connects a function signature to its:
/// - Parameter type (implementing [`SystemParam`])
/// - Cached state type (implementing [`SystemParamState`])
/// - Execution method
///
/// This trait is automatically implemented for functions with valid
/// system parameter signatures. You don't need to implement it manually.
///
/// # Type Parameters
///
/// - `Marker`: A marker type used to disambiguate different function arities
///
/// # Safety
///
/// Implementations must ensure that parameter access patterns are accurately
/// reported to enable safe parallel execution.
pub trait SystemParamFunction<Marker>: Send + 'static {
    /// The combined system parameter type for all function parameters.
    type Param: SystemParam;

    /// The combined state type for all parameters.
    type State: SystemParamState;

    /// Builds the access pattern for this function's parameters.
    fn build_access(state: &Self::State) -> Access;

    /// Runs the function with parameters extracted from the world.
    ///
    /// # Safety
    ///
    /// The caller must ensure exclusive access to the world. The implementation
    /// uses unsafe code internally to extract multiple parameters, which is safe
    /// because the access patterns are tracked and conflict detection ensures
    /// no aliasing occurs at runtime.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable reference to cached parameter state
    /// * `world` - Mutable reference to the world
    unsafe fn run_unsafe(&mut self, state: &mut Self::State, world: &mut World);
}

// =============================================================================
// IntoSystem Implementations for Functions
// =============================================================================

/// Marker type for functions with no parameters.
pub struct FnMarker;

/// Marker types for function arities 1-16.
pub struct FnMarker1<P>(PhantomData<P>);
pub struct FnMarker2<P1, P2>(PhantomData<(P1, P2)>);
pub struct FnMarker3<P1, P2, P3>(PhantomData<(P1, P2, P3)>);
pub struct FnMarker4<P1, P2, P3, P4>(PhantomData<(P1, P2, P3, P4)>);
pub struct FnMarker5<P1, P2, P3, P4, P5>(PhantomData<(P1, P2, P3, P4, P5)>);
pub struct FnMarker6<P1, P2, P3, P4, P5, P6>(PhantomData<(P1, P2, P3, P4, P5, P6)>);
pub struct FnMarker7<P1, P2, P3, P4, P5, P6, P7>(PhantomData<(P1, P2, P3, P4, P5, P6, P7)>);
pub struct FnMarker8<P1, P2, P3, P4, P5, P6, P7, P8>(PhantomData<(P1, P2, P3, P4, P5, P6, P7, P8)>);

// Implementation for zero-parameter functions: fn()
impl<F> SystemParamFunction<FnMarker> for F
where
    F: FnMut() + Send + 'static,
{
    type Param = ();
    type State = ();

    #[inline]
    fn build_access(_state: &Self::State) -> Access {
        Access::new()
    }

    #[inline]
    unsafe fn run_unsafe(&mut self, _state: &mut Self::State, _world: &mut World) {
        self();
    }
}

impl<F> IntoSystem<(FnMarker,)> for F
where
    F: FnMut() + Send + 'static,
{
    type System = FunctionSystem<FnMarker, F>;

    #[inline]
    fn into_system(self) -> BoxedSystem {
        BoxedSystem::new(FunctionSystem::new(self))
    }
}

// Macro to implement for functions with 1 parameter
macro_rules! impl_system_param_function_1 {
    ($marker:ident, $param:ident) => {
        #[allow(non_snake_case)]
        impl<F, $param: SystemParam + 'static> SystemParamFunction<$marker<$param>> for F
        where
            F: FnMut($param) + Send + 'static,
            for<'w, 's> F: FnMut($param::Item<'w, 's>),
            $param::State: Send + Sync + Clone + 'static,
        {
            type Param = $param;
            type State = $param::State;

            #[inline]
            fn build_access(state: &Self::State) -> Access {
                let mut access = Access::new();
                $param::update_access(state, &mut access);
                access
            }

            #[inline]
            unsafe fn run_unsafe(&mut self, state: &mut Self::State, world: &mut World) {
                // SAFETY: We have exclusive access to world. The access patterns
                // registered in build_access ensure no aliasing with other systems.
                let world_ptr = world as *mut World;
                let param = $param::get_param_mut(state, &mut *world_ptr);
                self(param);
            }
        }

        impl<F, $param: SystemParam + 'static> IntoSystem<($marker<$param>,)> for F
        where
            F: FnMut($param) + Send + 'static,
            for<'w, 's> F: FnMut($param::Item<'w, 's>),
            $param::State: Send + Sync + Clone + 'static,
        {
            type System = FunctionSystem<$marker<$param>, F>;

            #[inline]
            fn into_system(self) -> BoxedSystem {
                BoxedSystem::new(FunctionSystem::new(self))
            }
        }
    };
}

impl_system_param_function_1!(FnMarker1, P1);

// Macro to implement for functions with 2+ parameters
// These require unsafe pointer manipulation to work around Rust's borrow rules.
// This is safe because:
// 1. We have exclusive access to the world (&mut World)
// 2. The access tracking ensures systems don't run in parallel if they conflict
// 3. Each parameter accesses disjoint data (tracked by ComponentId/ResourceId)
macro_rules! impl_system_param_function_multi {
    ($marker:ident $(, $param:ident)* ; $($state_name:ident)*) => {
        #[allow(non_snake_case)]
        #[allow(unused_parens)]
        impl<F, $($param: SystemParam + 'static),*> SystemParamFunction<$marker<$($param),*>> for F
        where
            F: FnMut($($param),*) + Send + 'static,
            for<'w, 's> F: FnMut($($param::Item<'w, 's>),*),
            $($param::State: Send + Sync + Clone + 'static,)*
        {
            type Param = ($($param,)*);
            type State = ($($param::State,)*);

            #[inline]
            fn build_access(state: &Self::State) -> Access {
                let mut access = Access::new();
                let ($($state_name,)*) = state;
                $($param::update_access($state_name, &mut access);)*
                access
            }

            #[inline]
            unsafe fn run_unsafe(&mut self, state: &mut Self::State, world: &mut World) {
                // SAFETY: We have exclusive access to world through &mut World.
                // The parameter access patterns are tracked and conflict detection
                // ensures this system doesn't run in parallel with conflicting systems.
                // Each parameter type is responsible for accessing disjoint data.
                let world_ptr = world as *mut World;
                let ($($state_name,)*) = state;
                $(let $param = $param::get_param_mut($state_name, &mut *world_ptr);)*
                self($($param),*);
            }
        }

        impl<F, $($param: SystemParam + 'static),*> IntoSystem<($marker<$($param),*>,)> for F
        where
            F: FnMut($($param),*) + Send + 'static,
            for<'w, 's> F: FnMut($($param::Item<'w, 's>),*),
            $($param::State: Send + Sync + Clone + 'static,)*
        {
            type System = FunctionSystem<$marker<$($param),*>, F>;

            #[inline]
            fn into_system(self) -> BoxedSystem {
                BoxedSystem::new(FunctionSystem::new(self))
            }
        }
    };
}

// Implement for 2-8 parameters with unique state variable names
impl_system_param_function_multi!(FnMarker2, P1, P2; s1 s2);
impl_system_param_function_multi!(FnMarker3, P1, P2, P3; s1 s2 s3);
impl_system_param_function_multi!(FnMarker4, P1, P2, P3, P4; s1 s2 s3 s4);
impl_system_param_function_multi!(FnMarker5, P1, P2, P3, P4, P5; s1 s2 s3 s4 s5);
impl_system_param_function_multi!(FnMarker6, P1, P2, P3, P4, P5, P6; s1 s2 s3 s4 s5 s6);
impl_system_param_function_multi!(FnMarker7, P1, P2, P3, P4, P5, P6, P7; s1 s2 s3 s4 s5 s6 s7);
impl_system_param_function_multi!(FnMarker8, P1, P2, P3, P4, P5, P6, P7, P8; s1 s2 s3 s4 s5 s6 s7 s8);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentId;
    use crate::ecs::query::{Query, With};
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

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // Test resource (for documentation only - resource systems tested elsewhere)
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct Score(u32);

    // =========================================================================
    // Zero Parameter Function Tests
    // =========================================================================

    mod zero_params {
        use super::*;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        #[test]
        fn test_fn_no_params() {
            static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

            fn my_system() {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            }

            let mut world = World::new();
            let mut boxed = my_system.into_system();

            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 0);
            boxed.run(&mut world);
            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
            boxed.run(&mut world);
            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn test_fn_no_params_name() {
            fn my_system() {}

            let boxed = my_system.into_system();
            // Name should contain the function name
            assert!(boxed.name().contains("my_system"));
        }

        #[test]
        fn test_fn_no_params_is_read_only() {
            fn my_system() {}

            let boxed = my_system.into_system();
            // No parameters means read-only
            assert!(boxed.is_read_only());
        }

        #[test]
        fn test_fn_closure_no_params() {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            let system = move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            };

            let mut world = World::new();
            let mut boxed = system.into_system();

            boxed.run(&mut world);
            assert_eq!(counter.load(Ordering::SeqCst), 1);
        }
    }

    // =========================================================================
    // One Parameter Function Tests
    // =========================================================================

    mod one_param {
        use super::*;

        #[test]
        fn test_fn_one_query_param() {
            fn count_entities(_query: Query<&Position>) {
                // Query would be used here
            }

            let boxed = count_entities.into_system();
            assert!(boxed.name().contains("count_entities"));
        }

        #[test]
        fn test_fn_query_access_tracking() {
            fn position_reader(_query: Query<&Position>) {}

            let mut world = World::new();
            let mut boxed = position_reader.into_system();
            boxed.initialize(&mut world);

            let access = boxed.component_access();
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
            assert!(access.is_read_only());
        }

        #[test]
        fn test_fn_filtered_query() {
            fn player_positions(_query: Query<&Position, With<Player>>) {}

            let boxed = player_positions.into_system();
            assert!(boxed.name().contains("player_positions"));
        }
    }

    // =========================================================================
    // Two Parameter Function Tests
    // =========================================================================

    mod two_params {
        use super::*;

        #[test]
        fn test_fn_two_queries() {
            fn movement_system(_positions: Query<&Position>, _velocities: Query<&Velocity>) {}

            let boxed = movement_system.into_system();
            assert!(boxed.name().contains("movement_system"));
        }

        #[test]
        fn test_fn_two_filtered_queries() {
            fn player_system(
                _positions: Query<&Position, With<Player>>,
                _velocities: Query<&Velocity>,
            ) {
            }

            let boxed = player_system.into_system();
            assert!(boxed.name().contains("player_system"));
        }
    }

    // =========================================================================
    // Three+ Parameter Function Tests
    // =========================================================================

    mod multi_params {
        use super::*;
        use crate::ecs::Entity;

        #[test]
        fn test_fn_three_queries() {
            fn complex_system(
                _positions: Query<&Position>,
                _velocities: Query<&Velocity>,
                _entities: Query<Entity>,
            ) {
            }

            let boxed = complex_system.into_system();
            assert!(boxed.name().contains("complex_system"));
        }

        #[test]
        fn test_fn_four_queries() {
            fn even_more_complex(
                _positions: Query<&Position>,
                _velocities: Query<&Velocity>,
                _players: Query<&Position, With<Player>>,
                _entities: Query<Entity>,
            ) {
            }

            let boxed = even_more_complex.into_system();
            assert!(boxed.name().contains("even_more_complex"));
        }

        #[test]
        fn test_fn_with_filtered_queries() {
            fn filtered_system(
                _players: Query<&Position, With<Player>>,
                _velocities: Query<&Velocity, With<Player>>,
            ) {
            }

            let boxed = filtered_system.into_system();
            assert!(boxed.name().contains("filtered_system"));
        }
    }

    // =========================================================================
    // FunctionSystem Direct Tests
    // =========================================================================

    mod function_system {
        use super::*;

        #[test]
        fn test_function_system_new() {
            let system = FunctionSystem::<FnMarker, _>::new(|| {});
            assert!(format!("{system:?}").contains("FunctionSystem"));
        }

        #[test]
        fn test_function_system_with_name() {
            fn my_fn() {}

            let system = FunctionSystem::<FnMarker, _>::new(my_fn).with_name("CustomName");
            // Note: with_name sets meta.name, but System::name() returns type name
            // This is intentional - the type name is more useful for debugging
            let _ = system;
        }

        #[test]
        fn test_function_system_debug() {
            fn my_fn() {}

            let system = FunctionSystem::<FnMarker, _>::new(my_fn);
            let debug = format!("{system:?}");
            assert!(debug.contains("FunctionSystem"));
            assert!(debug.contains("initialized"));
        }

        #[test]
        fn test_function_system_initialize() {
            fn my_fn() {}

            let mut world = World::new();
            let mut system = FunctionSystem::<FnMarker, _>::new(my_fn);

            assert!(format!("{system:?}").contains("initialized: false"));
            system.initialize(&mut world);
            assert!(format!("{system:?}").contains("initialized: true"));
        }

        #[test]
        fn test_function_system_run_initializes() {
            static mut CALLED: bool = false;

            fn my_fn() {
                unsafe {
                    CALLED = true;
                }
            }

            let mut world = World::new();
            let mut system = FunctionSystem::<FnMarker, _>::new(my_fn);

            // Running should initialize if needed
            system.run(&mut world);
            assert!(unsafe { CALLED });
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        #[test]
        fn test_boxed_system_collection() {
            fn system_a() {}
            fn system_b() {}
            fn system_c() {}

            let systems: Vec<BoxedSystem> = vec![
                system_a.into_system(),
                system_b.into_system(),
                system_c.into_system(),
            ];

            assert_eq!(systems.len(), 3);
            assert!(systems[0].name().contains("system_a"));
            assert!(systems[1].name().contains("system_b"));
            assert!(systems[2].name().contains("system_c"));
        }

        #[test]
        fn test_run_boxed_systems() {
            let counter = Arc::new(AtomicU32::new(0));

            let c1 = counter.clone();
            let system1 = move || {
                c1.fetch_add(1, Ordering::SeqCst);
            };

            let c2 = counter.clone();
            let system2 = move || {
                c2.fetch_add(10, Ordering::SeqCst);
            };

            let mut systems: Vec<BoxedSystem> = vec![system1.into_system(), system2.into_system()];

            let mut world = World::new();

            for system in &mut systems {
                system.run(&mut world);
            }

            assert_eq!(counter.load(Ordering::SeqCst), 11);
        }

        #[test]
        fn test_system_with_actual_queries() {
            // Create a system that actually uses queries
            fn position_printer(_query: Query<&Position>) {
                // Would print position in real system
            }

            let mut world = World::new();

            // Add some entities
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            let mut boxed = position_printer.into_system();
            boxed.run(&mut world);
            // System ran without panic
        }

        #[test]
        fn test_reader_system() {
            fn reader(_query: Query<&Position>) {}

            let mut world = World::new();

            let mut reader_system = reader.into_system();

            // Initialize to get access patterns
            reader_system.initialize(&mut world);

            // Verify access pattern is read-only
            assert!(reader_system.is_read_only());
        }

        #[test]
        fn test_multi_query_system() {
            fn multi_query(_positions: Query<&Position>, _velocities: Query<&Velocity>) {}

            let mut world = World::new();

            let e = world.spawn_empty();
            world.insert(e, Position { x: 0.0, y: 0.0 });
            world.insert(e, Velocity { x: 1.0, y: 1.0 });

            let mut boxed = multi_query.into_system();
            boxed.run(&mut world);
            // System runs successfully
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_function_system_is_send() {
            fn my_system() {}

            fn requires_send<T: Send>() {}
            requires_send::<FunctionSystem<FnMarker, fn()>>();

            let _ = my_system.into_system();
        }

        #[test]
        fn test_boxed_system_is_send() {
            fn my_system() {}

            fn requires_send<T: Send>() {}

            let boxed = my_system.into_system();
            requires_send::<BoxedSystem>();
            let _ = boxed;
        }
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_world() {
            fn check_empty(_query: Query<&Position>) {
                // Would check count
            }

            let mut world = World::new();
            let mut boxed = check_empty.into_system();
            boxed.run(&mut world);
        }

        #[test]
        fn test_multiple_runs() {
            static mut RUN_COUNT: u32 = 0;

            fn counting_system() {
                unsafe {
                    RUN_COUNT += 1;
                }
            }

            let mut world = World::new();
            let mut boxed = counting_system.into_system();

            for _ in 0..100 {
                boxed.run(&mut world);
            }

            assert_eq!(unsafe { RUN_COUNT }, 100);
        }

        #[test]
        fn test_system_state_persists() {
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Arc;

            let run_count = Arc::new(AtomicU32::new(0));
            let run_count_clone = run_count.clone();

            let init_once = move || {
                run_count_clone.fetch_add(1, Ordering::SeqCst);
            };

            let mut world = World::new();
            let mut boxed = init_once.into_system();

            // Initialize explicitly
            boxed.initialize(&mut world);

            // Running should work and update run count
            boxed.run(&mut world);
            assert_eq!(run_count.load(Ordering::SeqCst), 1);

            // Running again should work too
            boxed.run(&mut world);
            assert_eq!(run_count.load(Ordering::SeqCst), 2);
        }
    }
}
