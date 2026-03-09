//! Accessor and scene/world utility methods for [`GoudGame`].

use crate::context_registry::scene::{SceneId, SceneManager};
use crate::core::error::GoudError;
use crate::core::providers::ProviderRegistry;
use crate::ecs::{Component, Entity, World};
use crate::sdk::debug_overlay::FpsStats;
use crate::sdk::entity_builder::EntityBuilder;
use crate::sdk::game_config::GameConfig;
use crate::ui::UiManager;

use super::GoudGame;

impl GoudGame {
    // =========================================================================
    // Default-scene World Access (backward-compatible)
    // =========================================================================

    /// Returns a reference to the default scene's ECS world.
    #[inline]
    pub fn world(&self) -> &World {
        self.scene_manager
            .get_scene(self.scene_manager.default_scene())
            .expect("default scene must exist")
    }

    /// Returns a mutable reference to the default scene's ECS world.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        let default = self.scene_manager.default_scene();
        self.scene_manager
            .get_scene_mut(default)
            .expect("default scene must exist")
    }

    /// Creates an entity builder for fluent entity creation (default scene).
    #[inline]
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let default = self.scene_manager.default_scene();
        let world = self
            .scene_manager
            .get_scene_mut(default)
            .expect("default scene must exist");
        EntityBuilder::new(world)
    }

    /// Spawns an empty entity with no components (default scene).
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        self.world_mut().spawn_empty()
    }

    /// Spawns multiple empty entities at once (default scene).
    #[inline]
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        self.world_mut().spawn_batch(count)
    }

    /// Despawns an entity and removes all its components (default scene).
    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.world_mut().despawn(entity)
    }

    /// Gets a reference to a component on an entity (default scene).
    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.world().get::<T>(entity)
    }

    /// Gets a mutable reference to a component on an entity (default scene).
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.world_mut().get_mut::<T>(entity)
    }

    /// Adds or replaces a component on an entity (default scene).
    #[inline]
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        self.world_mut().insert(entity, component);
    }

    /// Removes a component from an entity (default scene).
    #[inline]
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.world_mut().remove::<T>(entity)
    }

    /// Checks if an entity has a specific component (default scene).
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.world().has::<T>(entity)
    }

    /// Returns the number of entities in the default scene.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.world().entity_count()
    }

    /// Checks if an entity is alive (default scene).
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world().is_alive(entity)
    }

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

    /// Returns the current FPS statistics from the debug overlay.
    #[inline]
    pub fn fps_stats(&self) -> FpsStats {
        self.debug_overlay.stats()
    }

    /// Enables or disables the FPS stats overlay.
    #[inline]
    pub fn set_fps_overlay_enabled(&mut self, enabled: bool) {
        self.debug_overlay.set_enabled(enabled);
    }

    /// Returns the current frame count.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.context.frame_count()
    }

    /// Returns the total time elapsed since game start.
    #[inline]
    pub fn total_time(&self) -> f32 {
        self.context.total_time()
    }

    /// Returns the current FPS.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.context.fps()
    }

    /// Returns true if the game has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns a reference to the provider registry.
    #[inline]
    pub fn providers(&self) -> &ProviderRegistry {
        &self.providers
    }

    /// Returns a reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager(&self) -> Option<&crate::assets::AudioManager> {
        self.audio_manager.as_ref()
    }

    /// Returns a mutable reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager_mut(&mut self) -> Option<&mut crate::assets::AudioManager> {
        self.audio_manager.as_mut()
    }

    /// Returns a reference to the UI manager.
    #[inline]
    pub fn ui_manager(&self) -> &UiManager {
        &self.ui_manager
    }

    /// Returns a mutable reference to the UI manager.
    #[inline]
    pub fn ui_manager_mut(&mut self) -> &mut UiManager {
        &mut self.ui_manager
    }
}
