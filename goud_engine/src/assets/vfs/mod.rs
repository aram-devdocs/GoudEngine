//! Virtual filesystem abstraction for the asset system.
//!
//! Provides the [`VirtualFs`] trait and concrete implementations so the
//! [`AssetServer`](super::AssetServer) can load assets from different
//! storage backends (OS filesystem, archives, embedded resources).

mod archive_fs;
mod os_fs;
mod trait_def;

#[cfg(test)]
mod tests;

pub use archive_fs::ArchiveFs;
pub use os_fs::OsFs;
pub use trait_def::VirtualFs;
