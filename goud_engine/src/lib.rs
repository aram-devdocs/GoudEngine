#![warn(missing_docs)]
#![warn(rustdoc::all)]
#![allow(rustdoc::private_intra_doc_links)]

//! # Goud Engine Core
//!
//! This is the core library for the Goud Engine, a lightweight, cross-platform,
//! and data-oriented game engine written in Rust. It provides a flexible
//! Entity-Component-System (ECS) architecture, rendering abstractions, asset
//! management, and an FFI layer for scripting in other languages like C#.
//!
//! ## Key Modules
//!
//! - [`core`]: Foundational building blocks like error handling, generational
//!   handles, events, and math types.
//! - [`ecs`]: A full-featured, Bevy-inspired Entity-Component-System. This is
//!   the primary interface for game development.
//! - [`assets`]: A comprehensive asset management system with hot-reloading.
//! - [`ffi`]: The Foreign Function Interface for C# and other language bindings.
//! - [`libs`]: Low-level libraries, currently containing the graphics backend.

pub mod assets;
pub mod core;
pub mod ecs;
pub mod ffi;
/// Low-level libraries for graphics, platform, and other systems.
pub mod libs;
