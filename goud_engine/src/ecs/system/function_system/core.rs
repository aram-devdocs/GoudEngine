//! Core types for the function system: [`FunctionSystem`] and [`SystemParamFunction`].
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

use std::borrow::Cow;
use std::marker::PhantomData;

use crate::ecs::query::Access;
use crate::ecs::system::{
    BoxedSystem, IntoSystem, System, SystemMeta, SystemParam, SystemParamState,
};
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
// IntoSystem impl for zero-parameter functions
// =============================================================================

/// Marker type for functions with no parameters.
pub struct FnMarker;

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
