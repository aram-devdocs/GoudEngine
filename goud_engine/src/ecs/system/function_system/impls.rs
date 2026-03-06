//! Macro-generated [`SystemParamFunction`] and [`IntoSystem`] implementations
//! for functions with 1 to 8 parameters.
//!
//! These implementations allow regular Rust functions with up to 8 system
//! parameters to be converted into boxed systems via [`IntoSystem`].
//!
//! # Safety
//!
//! Multi-parameter implementations use raw pointer aliasing to work around
//! Rust's borrow checker when extracting multiple parameters from the same
//! `World`. This is safe because:
//! 1. We hold exclusive `&mut World` access.
//! 2. Access tracking ensures no two parameters alias the same component
//!    storage.
//! 3. Each `SystemParam` implementation is responsible for accessing
//!    disjoint data.

use std::marker::PhantomData;

use crate::ecs::query::Access;
use crate::ecs::system::{BoxedSystem, IntoSystem, SystemParam};
use crate::ecs::World;

use super::core::{FunctionSystem, SystemParamFunction};

// =============================================================================
// Marker types for function arities 1–8
// =============================================================================

/// Marker type for functions with 1 parameter.
pub struct FnMarker1<P>(PhantomData<P>);
/// Marker type for functions with 2 parameters.
pub struct FnMarker2<P1, P2>(PhantomData<(P1, P2)>);
/// Marker type for functions with 3 parameters.
pub struct FnMarker3<P1, P2, P3>(PhantomData<(P1, P2, P3)>);
/// Marker type for functions with 4 parameters.
pub struct FnMarker4<P1, P2, P3, P4>(PhantomData<(P1, P2, P3, P4)>);
/// Marker type for functions with 5 parameters.
pub struct FnMarker5<P1, P2, P3, P4, P5>(PhantomData<(P1, P2, P3, P4, P5)>);
/// Marker type for functions with 6 parameters.
pub struct FnMarker6<P1, P2, P3, P4, P5, P6>(PhantomData<(P1, P2, P3, P4, P5, P6)>);
/// Marker type for functions with 7 parameters.
pub struct FnMarker7<P1, P2, P3, P4, P5, P6, P7>(PhantomData<(P1, P2, P3, P4, P5, P6, P7)>);
/// Marker type for functions with 8 parameters.
pub struct FnMarker8<P1, P2, P3, P4, P5, P6, P7, P8>(PhantomData<(P1, P2, P3, P4, P5, P6, P7, P8)>);

// =============================================================================
// 1-parameter macro
// =============================================================================

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

// =============================================================================
// 2–8 parameter macro
// =============================================================================

// Macro to implement for functions with 2+ parameters.
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

// Implement for 2–8 parameters with unique state variable names
impl_system_param_function_multi!(FnMarker2, P1, P2; s1 s2);
impl_system_param_function_multi!(FnMarker3, P1, P2, P3; s1 s2 s3);
impl_system_param_function_multi!(FnMarker4, P1, P2, P3, P4; s1 s2 s3 s4);
impl_system_param_function_multi!(FnMarker5, P1, P2, P3, P4, P5; s1 s2 s3 s4 s5);
impl_system_param_function_multi!(FnMarker6, P1, P2, P3, P4, P5, P6; s1 s2 s3 s4 s5 s6);
impl_system_param_function_multi!(FnMarker7, P1, P2, P3, P4, P5, P6, P7; s1 s2 s3 s4 s5 s6 s7);
impl_system_param_function_multi!(FnMarker8, P1, P2, P3, P4, P5, P6, P7, P8; s1 s2 s3 s4 s5 s6 s7 s8);
