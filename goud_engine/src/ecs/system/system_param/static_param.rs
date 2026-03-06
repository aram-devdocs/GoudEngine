//! [`StaticSystemParam`] and [`StaticSystemParamState`] — state-backed system parameters.

use crate::ecs::query::Access;
use crate::ecs::World;

use super::traits::{SystemParam, SystemParamState};

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

/// State wrapper for [`StaticSystemParam`].
#[derive(Debug)]
pub struct StaticSystemParamState<S> {
    inner: S,
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
