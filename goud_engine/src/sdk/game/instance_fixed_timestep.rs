//! Fixed timestep game loop methods for [`GoudGame`].

use super::GoudGame;
use crate::context_registry::scene::SceneId;
use crate::ecs::World;
use crate::sdk::game_config::GameContext;

impl GoudGame {
    /// Runs the game loop with separate fixed-rate and per-frame callbacks.
    ///
    /// `fixed_update` is called at a deterministic rate controlled by the
    /// configured fixed timestep (see [`GameConfig::with_fixed_timestep`]).
    /// `render_update` is called once per visual frame for rendering and
    /// input handling.
    pub fn run_with_fixed_update<F, G>(&mut self, mut fixed_update: F, mut render_update: G)
    where
        F: FnMut(&mut GameContext, &mut World),
        G: FnMut(&mut GameContext, &mut World),
    {
        self.initialized = true;

        let frame_time = if self.config.target_fps > 0 {
            1.0 / self.config.target_fps as f32
        } else {
            1.0 / 60.0
        };

        while self.context.is_running() {
            let frame_time = self.prepare_runtime_frame(frame_time);
            self.context.update(frame_time);
            self.debug_overlay.update(frame_time);
            self.process_ui_frame();

            // Fixed timestep accumulator loop
            if self.context.is_fixed_timestep_enabled() {
                self.context.begin_frame_accumulator(frame_time);
                while self.context.consume_fixed_step() {
                    let active: Vec<SceneId> = self.scene_manager.active_scenes().to_vec();
                    for scene_id in active {
                        if let Some(world) = self.scene_manager.get_scene_mut(scene_id) {
                            fixed_update(&mut self.context, world);
                        }
                    }
                }
                self.context.finish_accumulator();
            }

            // Per-frame render update
            let active: Vec<SceneId> = self.scene_manager.active_scenes().to_vec();
            for scene_id in active {
                if let Some(world) = self.scene_manager.get_scene_mut(scene_id) {
                    render_update(&mut self.context, world);
                }
            }

            #[cfg(feature = "native")]
            if let Some(am) = &mut self.audio_manager {
                am.cleanup_finished();
            }

            if let Some(complete) = self.scene_manager.tick_transition(frame_time) {
                self.last_transition_complete = Some(complete);
            }

            self.update_physics_debug_shapes();
            #[cfg(feature = "native")]
            let render_start = std::time::Instant::now();
            self.render_ui_frame();
            #[cfg(feature = "native")]
            {
                let ui_ms = render_start.elapsed().as_secs_f32() * 1000.0;
                self.render_metrics.ui_render_ms = ui_ms;
                self.render_metrics.total_render_ms = ui_ms;
            }
            self.finish_runtime_frame();

            if self.context.frame_count() > 10000 {
                break;
            }
        }
    }

    /// Sets the fixed timestep at runtime (e.g. from FFI).
    pub fn set_fixed_timestep(&mut self, step: f32) {
        self.context
            .configure_fixed_timestep(step, self.config.max_fixed_steps_per_frame);
        self.config.fixed_timestep = step.max(0.0);
    }

    /// Sets the maximum fixed steps per frame at runtime (e.g. from FFI).
    pub fn set_max_fixed_steps(&mut self, max: u32) {
        let max = max.max(1);
        self.context
            .configure_fixed_timestep(self.config.fixed_timestep, max);
        self.config.max_fixed_steps_per_frame = max;
    }
}
