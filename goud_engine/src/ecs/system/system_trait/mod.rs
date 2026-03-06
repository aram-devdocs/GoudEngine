//! Core System trait and related types.
//!
//! This module defines the foundational [`System`] trait that all systems implement,
//! along with supporting types for system identification and metadata.

mod core;
mod system_id;
mod system_meta;

#[cfg(test)]
mod tests;

pub use core::{BoxedSystem, IntoSystem, System};
pub use system_id::SystemId;
pub use system_meta::SystemMeta;
