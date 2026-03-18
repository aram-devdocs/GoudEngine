//! # Rust Native SDK for GoudEngine
//!
//! This module provides a high-level, ergonomic Rust API for game development.
//! Unlike the FFI layer which is designed for cross-language bindings, this SDK
//! is pure Rust with zero FFI overhead - ideal for Rust-native game development.
//!
//! ## Architecture Philosophy
//!
//! The SDK follows a "Rust-first" design principle:
//!
//! - **All game logic lives in Rust**: Components, systems, and game behavior
//!   are implemented in Rust and exposed through this SDK
//! - **Zero-overhead abstractions**: No FFI marshalling, no runtime type checks
//! - **Type-safe**: Full Rust type safety with compile-time guarantees
//! - **Ergonomic**: Builder patterns, fluent APIs, and sensible defaults
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use goud_engine::sdk::{GoudGame, GameConfig};
//! use goud_engine::sdk::components::{Transform2D, Sprite};
//! use goud_engine::core::math::Vec2;
//!
//! fn main() {
//!     // Create game instance
//!     let mut game = GoudGame::new(GameConfig {
//!         title: "My Game".to_string(),
//!         width: 800,
//!         height: 600,
//!         ..Default::default()
//!     }).expect("Failed to create game");
//!
//!     // Spawn entities with fluent builder API
//!     let player = game.spawn()
//!         .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
//!         .with(Sprite::default())
//!         .build();
//!
//!     // Run game loop
//!     game.run(|ctx| {
//!         // Update logic here
//!         ctx.delta_time(); // Get frame delta
//!         ctx.input();      // Access input state
//!     });
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`components`](crate::sdk::components): Re-exports of all ECS components
//!   (Transform2D, Sprite, etc.)
//! - [`GoudGame`](crate::sdk::GoudGame): High-level game abstraction managing world,
//!   window, and loop
//! - [`EntityBuilder`](crate::sdk::EntityBuilder): Fluent builder for spawning
//!   entities with components
//! - [`GameContext`](crate::sdk::GameContext): Runtime context passed to update
//!   callbacks
//!
//! ## Comparison with FFI Layer
//!
//! | Feature | SDK (this module) | FFI Layer |
//! |---------|-------------------|-----------|
//! | Target | Rust games | C#/Python/etc |
//! | Overhead | Zero | Marshalling cost |
//! | Type Safety | Compile-time | Runtime checks |
//! | API Style | Idiomatic Rust | C-compatible |

pub mod collision;
pub mod color;
pub mod component_ops;
pub mod components;
pub mod components_sprite;
pub mod components_transform2d;
pub mod context;
pub mod debug_overlay;
pub mod engine_config;
pub mod entity;
pub mod entity_builder;
pub mod game;
pub mod game_config;
#[cfg(feature = "native")]
pub mod input;
pub mod network_debug_overlay;
#[cfg(feature = "native")]
pub mod rendering;
#[cfg(feature = "native")]
pub mod rendering_3d;
pub mod scene;
#[cfg(feature = "native")]
pub mod texture;
#[cfg(feature = "native")]
pub mod window;

// Re-export commonly used types for convenience
pub use crate::core::error::{GoudError, GoudResult};
pub use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};
pub use crate::ecs::{Component, Entity, EntityAllocator, SparseSet, World};

// Re-export SDK types from sub-modules so public API paths are preserved
pub use crate::core::debugger::DebuggerConfig;
pub use crate::libs::graphics::renderer3d::AntiAliasingMode;
pub use context::{Context, ContextConfig};
pub use engine_config::EngineConfig;
pub use entity_builder::EntityBuilder;
pub use game::GoudGame;
pub use game_config::{GameConfig, GameContext, RenderBackendKind, WindowBackendKind};
#[cfg(feature = "native")]
pub use rendering::Renderer2D;
pub use scene::{SceneId, SceneManager};

// Re-export components module contents at sdk level for convenience
// Note: We explicitly re-export to avoid shadowing issues
pub use components::{
    // Propagation
    propagate_transforms,
    propagate_transforms_2d,
    // Audio
    AttenuationModel,
    AudioChannel,
    AudioSource,
    // Hierarchy
    Children,
    // Physics
    Collider,
    ColliderShape,
    // Transforms
    GlobalTransform,
    GlobalTransform2D,
    Mat3x3,
    Name,
    Parent,
    RigidBody,
    RigidBodyType,
    // Rendering
    Sprite,
    Transform,
    Transform2D,
};

// =============================================================================
// Prelude - Convenient imports
// =============================================================================

/// Prelude module for convenient imports.
///
/// ```rust
/// use goud_engine::sdk::prelude::*;
/// ```
pub mod prelude {
    // Math types
    pub use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};

    // ECS core
    pub use crate::ecs::{Component, Entity, World};

    // SDK types
    #[cfg(feature = "native")]
    pub use super::Renderer2D;
    pub use super::{
        Context, ContextConfig, DebuggerConfig, EngineConfig, EntityBuilder, GameConfig,
        GameContext, GoudError, GoudGame, GoudResult, RenderBackendKind, SceneId, SceneManager,
        WindowBackendKind,
    };

    // Components - explicitly list to avoid shadowing
    pub use super::components::{
        // Propagation
        propagate_transforms,
        propagate_transforms_2d,
        // Audio
        AttenuationModel,
        AudioChannel,
        AudioSource,
        // Hierarchy
        Children,
        // Physics
        Collider,
        ColliderShape,
        // Transforms
        GlobalTransform,
        GlobalTransform2D,
        Mat3x3,
        Name,
        Parent,
        RigidBody,
        RigidBodyType,
        // Rendering
        Sprite,
        Transform,
        Transform2D,
    };
}
