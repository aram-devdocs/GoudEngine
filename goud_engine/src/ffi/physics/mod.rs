//! Physics FFI exports for 2D and 3D physics providers.
//!
//! This module provides C-compatible functions for creating, configuring, and
//! stepping physics simulations via the Rapier2D and Rapier3D backends.
//!
//! Physics providers are stored in a global registry keyed by context ID.
//! The caller must first create a provider with `goud_physics_create` or
//! `goud_physics3d_create` before calling other physics functions.

#[cfg(feature = "rapier2d")]
mod physics2d;
#[cfg(feature = "rapier2d")]
mod physics2d_common;
#[cfg(feature = "rapier2d")]
mod physics2d_events;
#[cfg(feature = "rapier2d")]
mod physics2d_ex;
#[cfg(feature = "rapier2d")]
mod physics2d_material;
#[cfg(feature = "rapier2d")]
mod physics2d_state;
#[cfg(feature = "rapier3d")]
mod physics3d;
#[cfg(feature = "rapier3d")]
mod physics3d_material;

#[cfg(feature = "rapier2d")]
// Keep both singular and plural collision-event symbols exported for backward compatibility:
// older SDKs use singular names while newer APIs use pluralized names.
pub use physics2d_events::{
    goud_physics_collision_event_count, goud_physics_collision_event_read,
    goud_physics_collision_events_count, goud_physics_collision_events_read,
    goud_physics_set_collision_callback,
};
#[cfg(feature = "rapier2d")]
pub use physics2d_ex::goud_physics_raycast_ex;
#[cfg(feature = "rapier2d")]
pub use physics2d_state::CollisionCallback;

// =============================================================================
// Global Physics Provider Registries (2D)
// =============================================================================

#[cfg(feature = "rapier2d")]
mod registry_2d {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    use crate::ffi::context::GoudContextId;

    /// Registry mapping context IDs to their 2D physics providers.
    pub(super) struct PhysicsRegistry2D {
        pub providers:
            HashMap<GoudContextId, Box<dyn crate::core::providers::physics::PhysicsProvider>>,
    }

    /// Returns the global 2D physics registry (thread-safe).
    pub(super) fn get() -> &'static Mutex<PhysicsRegistry2D> {
        static REGISTRY: OnceLock<Mutex<PhysicsRegistry2D>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Mutex::new(PhysicsRegistry2D {
                providers: HashMap::new(),
            })
        })
    }
}

// =============================================================================
// Global Physics Provider Registries (3D)
// =============================================================================

#[cfg(feature = "rapier3d")]
mod registry_3d {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    use crate::ffi::context::GoudContextId;

    /// Registry mapping context IDs to their 3D physics providers.
    pub(super) struct PhysicsRegistry3D {
        pub providers:
            HashMap<GoudContextId, Box<dyn crate::core::providers::physics3d::PhysicsProvider3D>>,
    }

    /// Returns the global 3D physics registry (thread-safe).
    pub(super) fn get() -> &'static Mutex<PhysicsRegistry3D> {
        static REGISTRY: OnceLock<Mutex<PhysicsRegistry3D>> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            Mutex::new(PhysicsRegistry3D {
                providers: HashMap::new(),
            })
        })
    }
}

#[cfg(feature = "rapier2d")]
use registry_2d::get as get_physics_registry_2d;
#[cfg(feature = "rapier3d")]
use registry_3d::get as get_physics_registry_3d;
