//! Runtime loop and diagnostic methods for [`GoudGame`].

use super::GoudGame;
#[cfg(feature = "native")]
use crate::assets::AssetServer;
use crate::context_registry::scene::SceneId;
use crate::core::error::GoudResult;
use crate::core::providers::ProviderRegistry;
use crate::ecs::World;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::StateOps;
use crate::sdk::debug_overlay::FpsStats;
use crate::sdk::engine_config::EngineConfig;
use crate::sdk::game_config::GameContext;
use crate::ui::UiManager;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
fn last_ui_command_counts() -> &'static Mutex<HashMap<usize, usize>> {
    static COUNTS: OnceLock<Mutex<HashMap<usize, usize>>> = OnceLock::new();
    COUNTS.get_or_init(|| Mutex::new(HashMap::new()))
}

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

        // Simple game loop (native builds drive this from the window backend)
        let frame_time = if self.config.target_fps > 0 {
            1.0 / self.config.target_fps as f32
        } else {
            1.0 / 60.0 // Default to 60 FPS for simulation
        };

        // For now, just run a few frames to demonstrate the API
        // Real implementation would integrate with windowing system
        while self.context.is_running() {
            let frame_time = self.prepare_runtime_frame(frame_time);
            self.context.update(frame_time);
            self.debug_overlay.update(frame_time);
            self.process_ui_frame();

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

            self.update_physics_debug_shapes();
            // Timing for render metrics
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

            // Safety: Limit iterations in tests/examples without actual window
            if self.context.frame_count() > 10000 {
                break;
            }
        }
    }

    /// Runs the game loop with separate fixed-rate and per-frame callbacks.
    ///
    /// `fixed_update` is called at a deterministic rate controlled by the
    /// configured fixed timestep (see [`GameConfig::with_fixed_timestep`]).
    /// `render_update` is called once per visual frame for rendering and
    /// input handling.
    ///
    /// Fixed timestep must be configured (non-zero) before calling this
    /// method; otherwise `render_update` is called once per frame with no
    /// fixed steps.
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

    /// Runs a single frame update for all active scenes.
    pub fn update_frame<F>(&mut self, delta_time: f32, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        let delta_time = self.prepare_runtime_frame(delta_time);
        self.context.update(delta_time);
        self.debug_overlay.update(delta_time);
        self.process_ui_frame();

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

        self.update_physics_debug_shapes();
        // Timing for render metrics
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

    fn render_ui_frame(&mut self) {
        #[cfg(feature = "native")]
        {
            if self.ui_render_system.is_none() {
                self.ui_render_system = Some(crate::rendering::UiRenderSystem::new());
            }
            if self.asset_server.is_none() {
                self.asset_server = Some(AssetServer::new());
            }

            let commands = self.ui_manager.build_render_commands();
            #[cfg(test)]
            self.record_last_ui_command_count(commands.len());
            let active_viewport = self.render_viewport();
            let viewport = active_viewport.logical_size();
            if let (Some(system), Some(asset_server), Some(backend)) = (
                self.ui_render_system.as_mut(),
                self.asset_server.as_mut(),
                self.render_backend.as_mut(),
            ) {
                crate::rendering::ensure_ui_asset_loaders(asset_server);
                backend.set_viewport(
                    active_viewport.x,
                    active_viewport.y,
                    active_viewport.width,
                    active_viewport.height,
                );
                if let Err(err) = system.run(&commands, asset_server, backend, viewport) {
                    log::warn!("UiRenderSystem frame failed: {err}");
                }
            }
        }

        #[cfg(not(feature = "native"))]
        {
            let _commands = self.ui_manager.build_render_commands();
            #[cfg(test)]
            self.record_last_ui_command_count(_commands.len());
        }
    }

    #[cfg(test)]
    fn record_last_ui_command_count(&self, count: usize) {
        let key = self as *const Self as usize;
        let mut counts = last_ui_command_counts()
            .lock()
            .expect("UI command count test registry should not be poisoned");
        counts.insert(key, count);
    }

    #[cfg(test)]
    pub(crate) fn last_ui_command_count(&self) -> usize {
        let key = self as *const Self as usize;
        let counts = last_ui_command_counts()
            .lock()
            .expect("UI command count test registry should not be poisoned");
        counts.get(&key).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::GoudResult;
    use crate::core::providers::impls::NullPhysicsProvider;
    use crate::core::providers::physics::PhysicsProvider;
    use crate::core::providers::types::{
        BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, ContactPair,
        DebugShape, JointDesc, JointHandle, PhysicsCapabilities, RaycastHit,
    };
    use crate::core::providers::{Provider, ProviderLifecycle};
    use crate::sdk::engine_config::EngineConfig;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingPhysicsProvider {
        inner: NullPhysicsProvider,
        debug_shapes_calls: Arc<AtomicUsize>,
    }

    impl CountingPhysicsProvider {
        fn new(debug_shapes_calls: Arc<AtomicUsize>) -> Self {
            Self {
                inner: NullPhysicsProvider::new(),
                debug_shapes_calls,
            }
        }
    }

    impl Provider for CountingPhysicsProvider {
        fn name(&self) -> &str {
            self.inner.name()
        }

        fn version(&self) -> &str {
            self.inner.version()
        }

        fn capabilities(&self) -> Box<dyn std::any::Any> {
            self.inner.capabilities()
        }
    }

    impl ProviderLifecycle for CountingPhysicsProvider {
        fn init(&mut self) -> GoudResult<()> {
            self.inner.init()
        }

        fn update(&mut self, delta: f32) -> GoudResult<()> {
            self.inner.update(delta)
        }

        fn shutdown(&mut self) {
            self.inner.shutdown();
        }
    }

    impl PhysicsProvider for CountingPhysicsProvider {
        fn physics_capabilities(&self) -> &PhysicsCapabilities {
            self.inner.physics_capabilities()
        }

        fn step(&mut self, delta: f32) -> GoudResult<()> {
            self.inner.step(delta)
        }

        fn set_gravity(&mut self, gravity: [f32; 2]) {
            self.inner.set_gravity(gravity);
        }

        fn gravity(&self) -> [f32; 2] {
            self.inner.gravity()
        }

        fn set_timestep(&mut self, dt: f32) {
            self.inner.set_timestep(dt);
        }

        fn timestep(&self) -> f32 {
            self.inner.timestep()
        }

        fn create_body(&mut self, desc: &BodyDesc) -> GoudResult<BodyHandle> {
            self.inner.create_body(desc)
        }

        fn destroy_body(&mut self, handle: BodyHandle) {
            self.inner.destroy_body(handle);
        }

        fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
            self.inner.body_position(handle)
        }

        fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 2]) -> GoudResult<()> {
            self.inner.set_body_position(handle, pos)
        }

        fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
            self.inner.body_velocity(handle)
        }

        fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 2]) -> GoudResult<()> {
            self.inner.set_body_velocity(handle, vel)
        }

        fn apply_force(&mut self, handle: BodyHandle, force: [f32; 2]) -> GoudResult<()> {
            self.inner.apply_force(handle, force)
        }

        fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 2]) -> GoudResult<()> {
            self.inner.apply_impulse(handle, impulse)
        }

        fn body_gravity_scale(&self, handle: BodyHandle) -> GoudResult<f32> {
            self.inner.body_gravity_scale(handle)
        }

        fn set_body_gravity_scale(&mut self, handle: BodyHandle, scale: f32) -> GoudResult<()> {
            self.inner.set_body_gravity_scale(handle, scale)
        }

        fn create_collider(
            &mut self,
            body: BodyHandle,
            desc: &ColliderDesc,
        ) -> GoudResult<ColliderHandle> {
            self.inner.create_collider(body, desc)
        }

        fn destroy_collider(&mut self, handle: ColliderHandle) {
            self.inner.destroy_collider(handle);
        }

        fn collider_friction(&self, handle: ColliderHandle) -> GoudResult<f32> {
            self.inner.collider_friction(handle)
        }

        fn set_collider_friction(
            &mut self,
            handle: ColliderHandle,
            friction: f32,
        ) -> GoudResult<()> {
            self.inner.set_collider_friction(handle, friction)
        }

        fn collider_restitution(&self, handle: ColliderHandle) -> GoudResult<f32> {
            self.inner.collider_restitution(handle)
        }

        fn set_collider_restitution(
            &mut self,
            handle: ColliderHandle,
            restitution: f32,
        ) -> GoudResult<()> {
            self.inner.set_collider_restitution(handle, restitution)
        }

        fn raycast(&self, origin: [f32; 2], dir: [f32; 2], max_dist: f32) -> Option<RaycastHit> {
            self.inner.raycast(origin, dir, max_dist)
        }

        fn overlap_circle(&self, center: [f32; 2], radius: f32) -> Vec<BodyHandle> {
            self.inner.overlap_circle(center, radius)
        }

        fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
            self.inner.drain_collision_events()
        }

        fn contact_pairs(&self) -> Vec<ContactPair> {
            self.inner.contact_pairs()
        }

        fn create_joint(&mut self, desc: &JointDesc) -> GoudResult<JointHandle> {
            self.inner.create_joint(desc)
        }

        fn destroy_joint(&mut self, handle: JointHandle) {
            self.inner.destroy_joint(handle);
        }

        fn debug_shapes(&self) -> Vec<DebugShape> {
            self.debug_shapes_calls.fetch_add(1, Ordering::SeqCst);
            self.inner.debug_shapes()
        }

        fn physics_diagnostics(&self) -> crate::core::providers::diagnostics::PhysicsDiagnosticsV1 {
            self.inner.physics_diagnostics()
        }
    }

    #[test]
    fn test_update_frame_skips_physics_debug_shapes_when_disabled() {
        let calls = Arc::new(AtomicUsize::new(0));
        let config = EngineConfig::new()
            .with_physics_provider(CountingPhysicsProvider::new(Arc::clone(&calls)))
            .with_physics_debug(false);
        let mut game = GoudGame::from_engine_config(config).unwrap();

        game.update_frame(0.016, |_ctx, _world| {});

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(game.physics_debug_shapes.is_empty());
    }

    #[test]
    fn test_update_frame_queries_physics_debug_shapes_when_enabled() {
        let calls = Arc::new(AtomicUsize::new(0));
        let config = EngineConfig::new()
            .with_physics_provider(CountingPhysicsProvider::new(Arc::clone(&calls)))
            .with_physics_debug(true);
        let mut game = GoudGame::from_engine_config(config).unwrap();

        game.update_frame(0.016, |_ctx, _world| {});

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
