//! Plugin and PluginGroup traits for modular app composition.
//!
//! Plugins encapsulate reusable bundles of systems, resources, and configuration
//! that can be added to an [`App`].

use std::any::TypeId;

use super::App;

/// A modular unit of app configuration.
///
/// Plugins add systems, resources, and other configuration to an [`App`].
/// They enable reusable, composable game engine features.
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::ecs::app::{App, Plugin};
///
/// struct PhysicsPlugin;
///
/// impl Plugin for PhysicsPlugin {
///     fn build(&self, app: &mut App) {
///         // Add physics systems, resources, etc.
///     }
/// }
/// ```
pub trait Plugin: Send + Sync + 'static {
    /// Configures the app with this plugin's systems and resources.
    fn build(&self, app: &mut App);

    /// Returns the name of this plugin for debugging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Returns the TypeIds of plugins this plugin depends on.
    ///
    /// Dependencies must be added before this plugin. The default
    /// implementation returns an empty list (no dependencies).
    fn dependencies(&self) -> Vec<TypeId> {
        Vec::new()
    }
}

/// A group of plugins that can be added to an [`App`] together.
///
/// Plugin groups provide a convenient way to add multiple related plugins
/// at once.
pub trait PluginGroup {
    /// Adds all plugins in this group to the app.
    fn build(self, app: &mut App);
}
