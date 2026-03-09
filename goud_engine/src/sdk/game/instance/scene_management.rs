use crate::context_registry::scene::{SceneId, SceneManager};
use crate::core::error::GoudError;
use crate::ecs::World;
use crate::sdk::game_config::GameConfig;

use super::GoudGame;

impl GoudGame {
    // =========================================================================
    // Scene Management
    // =========================================================================

    /// Creates a new scene with the given name.
    pub fn create_scene(&mut self, name: &str) -> Result<SceneId, GoudError> {
        self.scene_manager.create_scene(name)
    }

    /// Destroys a scene. Cannot destroy the default scene.
    pub fn destroy_scene(&mut self, id: SceneId) -> Result<(), GoudError> {
        self.scene_manager.destroy_scene(id)
    }

    /// Returns a reference to a scene's world.
    pub fn scene(&self, id: SceneId) -> Option<&World> {
        self.scene_manager.get_scene(id)
    }

    /// Returns a mutable reference to a scene's world.
    pub fn scene_mut(&mut self, id: SceneId) -> Option<&mut World> {
        self.scene_manager.get_scene_mut(id)
    }

    /// Looks up a scene by name.
    pub fn scene_by_name(&self, name: &str) -> Option<SceneId> {
        self.scene_manager.get_scene_by_name(name)
    }

    /// Sets whether a scene is active.
    pub fn set_scene_active(&mut self, id: SceneId, active: bool) -> Result<(), GoudError> {
        self.scene_manager.set_active(id, active)
    }

    /// Returns a reference to the scene manager.
    #[inline]
    pub fn scene_manager(&self) -> &SceneManager {
        &self.scene_manager
    }

    /// Returns a mutable reference to the scene manager.
    #[inline]
    pub fn scene_manager_mut(&mut self) -> &mut SceneManager {
        &mut self.scene_manager
    }

    /// Returns the game configuration.
    #[inline]
    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    /// Returns the window title.
    #[inline]
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Returns the window dimensions.
    #[inline]
    pub fn window_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}
