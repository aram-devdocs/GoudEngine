//! Built-in plugins that ship with the engine.
//!
//! These plugins provide core engine functionality like transform propagation.

use super::plugin::{Plugin, PluginGroup};
use super::App;
use crate::ecs::schedule::CoreStage;
use crate::ecs::systems::TransformPropagationSystem;

/// Plugin that adds the transform propagation system to PostUpdate.
///
/// This plugin registers [`TransformPropagationSystem`] in the
/// [`CoreStage::PostUpdate`] stage, ensuring that global transforms are
/// recomputed after all user systems have run.
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::ecs::app::App;
/// use goud_engine::ecs::app::TransformPropagationPlugin;
///
/// let mut app = App::new();
/// app.add_plugin(TransformPropagationPlugin);
/// ```
#[derive(Debug, Default, Clone)]
pub struct TransformPropagationPlugin;

impl Plugin for TransformPropagationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CoreStage::PostUpdate, TransformPropagationSystem::new());
    }

    fn name(&self) -> &'static str {
        "TransformPropagationPlugin"
    }
}

/// A plugin group that adds all default engine plugins.
///
/// Currently includes:
/// - [`TransformPropagationPlugin`]: Automatic transform hierarchy propagation
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::ecs::app::App;
/// use goud_engine::ecs::DefaultPlugins;
///
/// let mut app = App::new_with_defaults();
/// // Or manually:
/// let mut app = App::new();
/// app.add_plugin_group(DefaultPlugins);
/// ```
#[derive(Debug, Default, Clone)]
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self, app: &mut App) {
        app.add_plugin(TransformPropagationPlugin);
    }
}
