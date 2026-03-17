//! Provider trait abstractions for GoudEngine subsystems.
//!
//! This module defines the universal provider pattern used by all engine
//! subsystems. Providers are swappable implementations selected at engine
//! initialization time, enabling runtime backend selection and headless testing.
//!
//! # Architecture
//!
//! All provider traits are object-safe (no associated types, no generic methods)
//! and stored as `Box<dyn XxxProvider>`. Dynamic dispatch overhead is acceptable
//! because provider calls are coarse-grained (per-frame or per-batch).
//!
//! The `Provider` supertrait requires `Send + Sync + 'static` for all providers
//! except `WindowProvider`, which is `!Send + !Sync` because native windowing
//! backends require main-thread access.

pub mod audio;
pub mod diagnostics;
pub mod impls;
pub mod input;
pub mod input_types;
pub mod network;
pub mod network_types;
pub mod physics;
pub mod physics3d;
pub mod render;
pub mod types;
pub mod types3d;
pub mod window;

mod builder;
mod registry;

pub use builder::ProviderRegistryBuilder;
pub use diagnostics::{
    AudioDiagnosticsV1, DiagnosticsSource, InputDiagnosticsV1, NetworkDiagnosticsV1,
    Physics3DDiagnosticsV1, PhysicsDiagnosticsV1, RenderDiagnosticsV1, WindowDiagnosticsV1,
};
pub use registry::ProviderRegistry;

use crate::core::error::GoudResult;

/// Common supertrait for all providers (except `WindowProvider`).
///
/// Provides identity, versioning, and capability introspection. The
/// `Send + Sync + 'static` bounds allow providers to be stored in
/// `ProviderRegistry`, which may be accessed from worker threads
/// during asset streaming.
pub trait Provider: Send + Sync + 'static {
    /// Returns the human-readable name of this provider (e.g., "OpenGL").
    fn name(&self) -> &str;

    /// Returns the version string of this provider implementation.
    fn version(&self) -> &str;

    /// Returns provider capabilities as a type-erased `Any` value.
    ///
    /// For subsystem-specific code, prefer the typed accessor on the
    /// subsystem trait (e.g., `RenderProvider::render_capabilities()`).
    /// This generic method exists for code that operates on providers
    /// generically (e.g., logging all provider capabilities at startup).
    fn capabilities(&self) -> Box<dyn std::any::Any>;
}

/// Lifecycle management for providers.
///
/// All subsystem provider traits extend both `Provider` and `ProviderLifecycle`.
/// The lifecycle follows five phases:
///
/// 1. **Create** -- constructed with a config struct
/// 2. **Init** -- `init()` called during `GoudGame::new()`
/// 3. **Update** -- per-frame `update(delta)` for providers that need it
/// 4. **Shutdown** -- `shutdown()` called during `GoudGame::drop()`
/// 5. **Drop** -- provider dropped after shutdown
pub trait ProviderLifecycle {
    /// Initialize the provider. Called once during engine startup.
    ///
    /// Failure is fatal unless a fallback provider is configured.
    fn init(&mut self) -> GoudResult<()>;

    /// Per-frame update. Called once per frame for providers that need it
    /// (e.g., physics stepping, audio streaming).
    fn update(&mut self, delta: f32) -> GoudResult<()>;

    /// Shut down the provider. Called once during engine teardown.
    ///
    /// Must not fail. All GPU/OS resources must be released before `Drop`.
    fn shutdown(&mut self);
}
