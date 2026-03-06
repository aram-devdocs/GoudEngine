//! Tuple implementations of [`SystemParam`] and [`SystemParamState`].
//!
//! Allows system functions to accept multiple parameters as tuples.

use crate::ecs::query::Access;
use crate::ecs::World;

use super::traits::{ReadOnlySystemParam, SystemParam, SystemParamState};

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
// Note: () is implemented separately in traits.rs, so we start with single-element tuples
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
