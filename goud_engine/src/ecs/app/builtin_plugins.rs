//! Built-in plugins that ship with the engine.
//!
//! These plugins provide core engine functionality like transform propagation.

use super::plugin::Plugin;
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
