#![warn(missing_docs)]
#![warn(rustdoc::all)]
#![allow(rustdoc::private_intra_doc_links)]
// Allow common lint warnings in test code
#![cfg_attr(test, allow(dead_code))]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(unused_unsafe))]
#![cfg_attr(test, allow(unused_assignments))]
#![cfg_attr(test, allow(clippy::useless_vec))]
#![cfg_attr(test, allow(clippy::unnecessary_unwrap))]
#![cfg_attr(test, allow(clippy::approx_constant))]
#![cfg_attr(test, allow(clippy::clone_on_copy))]
#![cfg_attr(test, allow(clippy::unnecessary_cast))]
#![cfg_attr(test, allow(clippy::unit_cmp))]
#![cfg_attr(test, allow(clippy::drop_non_drop))]
#![cfg_attr(test, allow(clippy::bool_assert_comparison))]
#![cfg_attr(test, allow(clippy::manual_range_contains))]
#![cfg_attr(test, allow(clippy::single_char_add_str))]
#![cfg_attr(test, allow(clippy::len_zero))]
#![cfg_attr(test, allow(clippy::cmp_owned))]
#![cfg_attr(test, allow(clippy::assertions_on_constants))]
#![cfg_attr(test, allow(clippy::default_constructed_unit_structs))]
#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

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
/// High-level Rust-native SDK for game development.
///
/// This module provides an ergonomic, zero-overhead API for building games
/// in pure Rust. Unlike the FFI layer, the SDK is designed for Rust developers
/// with full type safety and idiomatic APIs.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::{GoudGame, GameConfig};
/// use goud_engine::sdk::components::Transform2D;
/// use goud_engine::core::math::Vec2;
///
/// let mut game = GoudGame::new(GameConfig::default()).unwrap();
/// let player = game.spawn()
///     .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
///     .build();
/// ```
pub mod sdk;
