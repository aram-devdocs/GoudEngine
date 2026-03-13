//! Window provider trait definition.
//!
//! The `WindowProvider` trait abstracts the windowing backend, enabling
//! runtime selection between GLFW, winit, or null (headless).
//!
//! # Thread Safety
//!
//! `WindowProvider` is intentionally NOT `Send + Sync`. GLFW requires all
//! window calls on the main thread. The engine enforces this by storing
//! `WindowProvider` outside `ProviderRegistry`, directly in `GoudGame`.

use super::diagnostics::WindowDiagnosticsV1;
use crate::core::error::GoudResult;

/// Trait for windowing backends.
///
/// This trait does NOT extend `Provider` (which requires `Send + Sync`)
/// because GLFW and similar windowing libraries require main-thread access.
/// It includes its own `init()`/`shutdown()` methods matching the
/// `ProviderLifecycle` contract.
///
/// The trait is object-safe and stored as `Box<dyn WindowProvider>`.
pub trait WindowProvider: 'static {
    /// Returns the human-readable name of this window provider.
    fn name(&self) -> &str;

    /// Initialize the window provider and create the window.
    fn init(&mut self) -> GoudResult<()>;

    /// Shut down the window provider and destroy the window.
    ///
    /// Must not fail. All OS resources must be released.
    fn shutdown(&mut self);

    /// Returns true if the window has been requested to close.
    fn should_close(&self) -> bool;

    /// Set the close request flag on the window.
    fn set_should_close(&mut self, value: bool);

    /// Poll the OS event queue for window and input events.
    fn poll_events(&mut self);

    /// Swap the front and back buffers (present the frame).
    fn swap_buffers(&mut self);

    /// Get the window size in screen coordinates as (width, height).
    fn get_size(&self) -> (u32, u32);

    /// Get the framebuffer size in pixels as (width, height).
    ///
    /// May differ from `get_size()` on high-DPI displays.
    fn get_framebuffer_size(&self) -> (u32, u32);

    // -------------------------------------------------------------------------
    // Diagnostics
    // -------------------------------------------------------------------------

    /// Returns a snapshot of window diagnostics.
    fn window_diagnostics(&self) -> WindowDiagnosticsV1;
}
