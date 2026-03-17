//! Debugger runtime frame lifecycle, synthetic input injection, and snapshot
//! refresh for [`GoudGame`].

use std::sync::atomic::{AtomicU32, Ordering};

use crate::context_registry::scene;
use crate::core::context_id::GoudContextId;
use crate::core::debugger::{self, RuntimeRouteId, RuntimeSurfaceKind, SyntheticInputEventV1};
use crate::sdk::game_config::GameConfig;

use super::GoudGame;

impl GoudGame {
    pub(crate) fn next_debugger_context_id() -> GoudContextId {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1_000_000);
        GoudContextId::new(NEXT_ID.fetch_add(1, Ordering::Relaxed), 1)
    }

    pub(crate) fn register_debugger_route(
        config: &GameConfig,
        surface: RuntimeSurfaceKind,
    ) -> Option<RuntimeRouteId> {
        config.debugger.enabled.then(|| {
            debugger::register_context(Self::next_debugger_context_id(), surface, &config.debugger)
        })
    }

    #[cfg(feature = "native")]
    pub(crate) fn apply_synthetic_inputs(&mut self, events: &[SyntheticInputEventV1]) {
        use crate::core::providers::input_types::{KeyCode as Key, MouseButton};

        for event in events {
            match (
                event.device.as_str(),
                event.action.as_str(),
                event.key.as_deref(),
                event.button.as_deref(),
            ) {
                ("keyboard", "press", Some(key), _) => {
                    if let Some(key) = Key::from_debugger_name(key) {
                        self.input_manager.press_key(key);
                    } else {
                        log::warn!("Ignoring unsupported debugger synthetic key '{key}'");
                    }
                }
                ("keyboard", "release", Some(key), _) => {
                    if let Some(key) = Key::from_debugger_name(key) {
                        self.input_manager.release_key(key);
                    } else {
                        log::warn!("Ignoring unsupported debugger synthetic key '{key}'");
                    }
                }
                ("mouse", "press", _, Some(button)) => {
                    if let Some(button) = MouseButton::from_debugger_name(button) {
                        self.input_manager.press_mouse_button(button);
                    } else {
                        log::warn!("Ignoring unsupported debugger mouse button '{button}'");
                    }
                }
                ("mouse", "release", _, Some(button)) => {
                    if let Some(button) = MouseButton::from_debugger_name(button) {
                        self.input_manager.release_mouse_button(button);
                    } else {
                        log::warn!("Ignoring unsupported debugger mouse button '{button}'");
                    }
                }
                _ => {}
            }
        }
    }

    #[cfg(not(feature = "native"))]
    pub(crate) fn apply_synthetic_inputs(&mut self, _events: &[SyntheticInputEventV1]) {}

    pub(crate) fn prepare_runtime_frame(&mut self, raw_delta_seconds: f32) -> f32 {
        let Some(route_id) = self.debugger_route.clone() else {
            self.runtime_debug_draw_enabled = false;
            return raw_delta_seconds;
        };

        let frame_plan = debugger::take_frame_control_for_route(&route_id, raw_delta_seconds)
            .unwrap_or_default();
        self.runtime_debug_draw_enabled = frame_plan.debug_draw_enabled;
        self.apply_synthetic_inputs(&frame_plan.synthetic_inputs);

        let (next_index, total_seconds) = debugger::snapshot_for_route(&route_id)
            .map(|snapshot| {
                (
                    snapshot.frame.index.saturating_add(1),
                    snapshot.frame.total_seconds + frame_plan.effective_delta_seconds as f64,
                )
            })
            .unwrap_or((1, frame_plan.effective_delta_seconds as f64));
        debugger::begin_frame(
            &route_id,
            next_index,
            frame_plan.effective_delta_seconds,
            total_seconds,
        );
        frame_plan.effective_delta_seconds
    }

    pub(crate) fn finish_runtime_frame(&mut self) {
        let route_id = match self.debugger_route.as_ref() {
            Some(r) => r.clone(),
            None => return,
        };

        // 1. Collect ALL diagnostics from providers (type-safe, automatic).
        let mut all_diag = self.providers.collect_provider_diagnostics();

        // 2. Collect non-provider diagnostics.
        #[cfg(feature = "native")]
        {
            use crate::core::providers::diagnostics::DiagnosticsSource;

            if let Some(ref batch) = self.sprite_batch {
                all_diag.insert("sprite_batch".into(), batch.collect_diagnostics());
            }
            if let Some(ref text_batch) = self.text_batch {
                all_diag.insert("text_batch".into(), text_batch.collect_diagnostics());
            }
            if let Some(ref ui_render) = self.ui_render_system {
                all_diag.insert("ui_render".into(), ui_render.collect_diagnostics());
            }
            if let Some(ref asset_server) = self.asset_server {
                all_diag.insert("assets".into(), asset_server.collect_diagnostics());
            }
        }

        // 3. Push to snapshot.
        debugger::with_snapshot_mut(&route_id, |snapshot| {
            // Backward compat: populate legacy stats.render from provider diagnostics.
            if let Some(render_val) = all_diag.get("render") {
                if let Ok(rd) = serde_json::from_value::<
                    crate::core::providers::diagnostics::RenderDiagnosticsV1,
                >(render_val.clone())
                {
                    snapshot.stats.render = crate::core::debugger::RenderStatsV1 {
                        draw_calls: rd.draw_calls,
                        triangles: rd.triangles,
                        texture_binds: rd.texture_binds,
                        shader_binds: rd.shader_binds,
                    };
                }
            }
            // Also populate from sprite batch for backward compat.
            if let Some(sb_val) = all_diag.get("sprite_batch") {
                if let (Some(sprites), Some(batches)) = (
                    sb_val.get("sprite_count").and_then(|v| v.as_u64()),
                    sb_val.get("batch_count").and_then(|v| v.as_u64()),
                ) {
                    snapshot.stats.render.draw_calls = batches as u32;
                    snapshot.stats.render.triangles = (sprites * 2) as u32;
                    snapshot.stats.render.texture_binds = batches as u32;
                    snapshot.stats.render.shader_binds = 1;
                }
            }
            snapshot.provider_diagnostics = all_diag;
        });

        // 4. Entity/scene data.
        self.refresh_debugger_snapshot_data(&route_id);
        debugger::end_frame(&route_id);
    }

    /// Populates the debugger snapshot with current entity and scene data.
    fn refresh_debugger_snapshot_data(&self, route_id: &RuntimeRouteId) {
        let active_scene_name = self
            .scene_manager
            .get_scene_name(self.scene_manager.default_scene())
            .unwrap_or("default")
            .to_string();

        let selected_entity = debugger::snapshot_for_route(route_id).and_then(|snapshot| {
            snapshot.selection.entity_id.map(|entity_id| {
                let scene_id = if snapshot.selection.scene_id.is_empty() {
                    active_scene_name.clone()
                } else {
                    snapshot.selection.scene_id.clone()
                };
                (scene_id, entity_id)
            })
        });

        let mut entities = Vec::new();
        let mut entity_count = 0usize;
        for scene_id in self.scene_manager.active_scenes() {
            let Some(world) = self.scene_manager.get_scene(*scene_id) else {
                continue;
            };
            entity_count = entity_count.saturating_add(world.entity_count());
            let scene_name = self
                .scene_manager
                .get_scene_name(*scene_id)
                .unwrap_or("unknown")
                .to_string();
            let scene_entities = scene::collect_debugger_entities(
                world,
                scene_name,
                selected_entity
                    .as_ref()
                    .map(|(scene_id, entity_id)| (scene_id.as_str(), *entity_id)),
            );
            entities.extend(scene_entities);
        }

        let _ = debugger::with_snapshot_mut(route_id, |snapshot| {
            snapshot.scene.active_scene = active_scene_name.clone();
            snapshot.scene.entity_count = entity_count as u32;
            if snapshot.selection.entity_id.is_none() || snapshot.selection.scene_id.is_empty() {
                snapshot.selection.scene_id = active_scene_name;
            }
            snapshot.entities = entities;
        });
    }

    /// Updates cached physics debug shapes according to runtime config.
    ///
    /// When disabled, this avoids querying the physics provider entirely.
    pub(crate) fn update_physics_debug_shapes(&mut self) {
        if !self.config.physics_debug.enabled && !self.runtime_debug_draw_enabled {
            self.physics_debug_shapes.clear();
            return;
        }

        self.physics_debug_shapes = self.providers.physics.debug_shapes();
    }
}
