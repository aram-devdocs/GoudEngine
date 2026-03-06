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

mod param_set;
mod resource_params;
mod static_param;
#[cfg(test)]
mod tests;
mod traits;
mod tuple_impl;

pub use param_set::ParamSet;
pub use resource_params::{ResMutState, ResState};
pub use static_param::{StaticSystemParam, StaticSystemParamState};
pub use traits::{ReadOnlySystemParam, SystemParam, SystemParamState};
