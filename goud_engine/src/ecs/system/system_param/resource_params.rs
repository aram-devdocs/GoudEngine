//! [`SystemParam`] implementations for resource access types.
//!
//! Covers [`Res`], [`ResMut`], [`NonSend`], and [`NonSendMut`].

use crate::ecs::query::Access;
use crate::ecs::resource::{NonSend, NonSendMut, NonSendResource, NonSendResourceId};
use crate::ecs::resource::{Res, ResMut, Resource, ResourceId};
use crate::ecs::World;

use super::traits::{ReadOnlySystemParam, SystemParam, SystemParamState};

// =============================================================================
// Res<T> - Immutable Resource SystemParam
// =============================================================================

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
