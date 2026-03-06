//! Access tracking and conflict detection for the query system.
//!
//! Types in this module describe which components and resources a query or
//! system accesses, and provide conflict detection to enforce Rust's aliasing
//! rules across concurrent system execution.

mod conflict;
mod types;

pub use conflict::{AccessConflict, ConflictInfo, NonSendConflictInfo, ResourceConflictInfo};
pub use types::{Access, AccessType};
