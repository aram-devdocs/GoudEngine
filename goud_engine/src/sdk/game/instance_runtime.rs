//! Runtime loop and diagnostic methods for [`GoudGame`].

use super::GoudGame;
use crate::context_registry::scene::SceneId;
use crate::core::error::GoudResult;
use crate::core::providers::ProviderRegistry;
use crate::ecs::World;
use crate::sdk::debug_overlay::FpsStats;
use crate::sdk::engine_config::EngineConfig;
use crate::sdk::game_config::GameContext;
use crate::ui::UiManager;

impl GoudGame {
    /// Runs the game loop with the given update callback.
    ///
    /// The callback receives the default scene's world for backward
    /// compatibility. Use [`scene_manager_mut`](Self::scene_manager_mut)
    /// inside the callback for multi-scene access.
    pub fn run<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        self.initialized = true;

        // Simple game loop (actual implementation would use GLFW/window events)
        let frame_time = if self.config.target_fps > 0 {
            1.0 / self.config.target_fps as f32
        } else {
            1.0 / 60.0 // Default to 60 FPS for simulation
        };

        // For now, just run a few frames to demonstrate the API
        // Real implementation would integrate with windowing system
        while self.context.is_running() {
            self.context.update(frame_time);
            self.debug_overlay.update(frame_time);

            // Update all active scenes each frame.
            let active: Vec<SceneId> = self.scene_manager.active_scenes().to_vec();
            for scene_id in active {
                if let Some(world) = self.scene_manager.get_scene_mut(scene_id) {
                    update(&mut self.context, world);
                }
            }

            // Clean up finished audio players
            #[cfg(feature = "native")]
            if let Some(am) = &mut self.audio_manager {
                am.cleanup_finished();
            }

            // Advance any in-progress scene transition.
            if let Some(complete) = self.scene_manager.tick_transition(frame_time) {
                self.last_transition_complete = Some(complete);
            }

            // Update UI manager after scene updates.
            self.ui_manager.update();

            // Render UI manager after updates (before buffer swap).
            self.ui_manager.render();

            // Safety: Limit iterations in tests/examples without actual window
            if self.context.frame_count() > 10000 {
                break;
            }
        }
    }

    /// Runs a single frame update for all active scenes.
    pub fn update_frame<F>(&mut self, delta_time: f32, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        self.context.update(delta_time);
        self.debug_overlay.update(delta_time);

        let active: Vec<SceneId> = self.scene_manager.active_scenes().to_vec();
        for scene_id in active {
            if let Some(world) = self.scene_manager.get_scene_mut(scene_id) {
                update(&mut self.context, world);
            }
        }

        // Clean up finished audio players
        #[cfg(feature = "native")]
        if let Some(am) = &mut self.audio_manager {
            am.cleanup_finished();
        }

        // Advance any in-progress scene transition.
        if let Some(complete) = self.scene_manager.tick_transition(delta_time) {
            self.last_transition_complete = Some(complete);
        }

        // Update UI manager after scene updates.
        self.ui_manager.update();

        // Render UI manager after updates (before buffer swap).
        self.ui_manager.render();
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

    /// Creates a headless game from an [`EngineConfig`] builder.
    pub fn from_engine_config(config: EngineConfig) -> GoudResult<Self> {
        let (game_config, providers) = config.build();
        let mut game = Self::new(game_config)?;
        game.providers = providers;
        Ok(game)
    }

    /// Creates a windowed game from an [`EngineConfig`] builder.
    #[cfg(feature = "native")]
    pub fn from_engine_config_with_platform(config: EngineConfig) -> GoudResult<Self> {
        let (game_config, providers) = config.build();
        let mut game = Self::with_platform(game_config)?;
        game.providers = providers;
        Ok(game)
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
