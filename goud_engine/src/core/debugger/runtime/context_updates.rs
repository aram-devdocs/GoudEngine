use super::super::snapshot::MemorySummaryV1;
use super::super::types::CapabilityStateV1;
use crate::core::context_id::GoudContextId;

use super::state::{
    memory_category_mut, recalculate_memory_totals, set_route_capability, set_service_state,
    with_route_state_mut_by_context, RuntimeFpsStats,
};

/// Stores the current FPS overlay statistics for a route-backed context.
pub fn update_fps_stats_for_context(
    context_id: GoudContextId,
    current_fps: f32,
    min_fps: f32,
    max_fps: f32,
    avg_fps: f32,
    frame_time_ms: f32,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.fps_stats =
            RuntimeFpsStats::from_values(current_fps, min_fps, max_fps, avg_fps, frame_time_ms);
        true
    })
    .unwrap_or(false)
}

/// Returns the current FPS overlay statistics for a route-backed context.
pub fn fps_stats_for_context(context_id: GoudContextId) -> Option<[f32; 5]> {
    with_route_state_mut_by_context(context_id, |route| route.fps_stats.as_array())
}

/// Sets the currently selected entity for one context.
pub fn set_selected_entity_for_context(context_id: GoudContextId, entity_id: Option<u64>) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        if !route.snapshot.scene.active_scene.is_empty() {
            route.snapshot.selection.scene_id = route.snapshot.scene.active_scene.clone();
        }
        route.snapshot.selection.entity_id = entity_id;
        true
    })
    .unwrap_or(false)
}

/// Appends render stats to the current frame totals for one context.
pub fn update_render_stats_for_context(
    context_id: GoudContextId,
    draw_calls: u32,
    triangles: u32,
    texture_binds: u32,
    shader_binds: u32,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.snapshot.stats.render.draw_calls = route
            .snapshot
            .stats
            .render
            .draw_calls
            .saturating_add(draw_calls);
        route.snapshot.stats.render.triangles = route
            .snapshot
            .stats
            .render
            .triangles
            .saturating_add(triangles);
        route.snapshot.stats.render.texture_binds = route
            .snapshot
            .stats
            .render
            .texture_binds
            .saturating_add(texture_binds);
        route.snapshot.stats.render.shader_binds = route
            .snapshot
            .stats
            .render
            .shader_binds
            .saturating_add(shader_binds);
        set_route_capability(route, "render_stats", CapabilityStateV1::Ready);
        set_service_state(
            &mut route.snapshot,
            "renderer",
            CapabilityStateV1::Ready,
            None,
        );
        true
    })
    .unwrap_or(false)
}

/// Updates one tracked memory category for the given context.
pub fn update_memory_category_for_context(
    context_id: GoudContextId,
    category: &str,
    current_bytes: u64,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        let Some(category_stats) =
            memory_category_mut(&mut route.snapshot.memory_summary, category)
        else {
            return false;
        };
        category_stats.current_bytes = current_bytes;
        category_stats.peak_bytes = category_stats.peak_bytes.max(current_bytes);
        recalculate_memory_totals(&mut route.snapshot.memory_summary);
        route.snapshot.stats.memory.tracked_bytes =
            route.snapshot.memory_summary.total_current_bytes;
        route.snapshot.stats.memory.peak_bytes = route.snapshot.memory_summary.total_peak_bytes;
        set_route_capability(route, "memory_stats", CapabilityStateV1::Ready);
        set_service_state(
            &mut route.snapshot,
            "memory",
            CapabilityStateV1::Ready,
            None,
        );
        true
    })
    .unwrap_or(false)
}

/// Returns the memory summary for one context, if registered.
pub fn get_memory_summary_for_context(context_id: GoudContextId) -> Option<MemorySummaryV1> {
    with_route_state_mut_by_context(context_id, |route| route.snapshot.memory_summary)
}

/// Stores the latest network snapshot totals for a context.
pub fn set_snapshot_network_stats_for_context(
    context_id: GoudContextId,
    bytes_sent: u64,
    bytes_received: u64,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.snapshot.stats.network.bytes_sent = bytes_sent;
        route.snapshot.stats.network.bytes_received = bytes_received;
        set_service_state(
            &mut route.snapshot,
            "network",
            CapabilityStateV1::Ready,
            None,
        );
        true
    })
    .unwrap_or(false)
}

/// Updates one service entry on the context snapshot.
pub fn set_service_state_for_context(
    context_id: GoudContextId,
    name: &str,
    state: CapabilityStateV1,
    detail: Option<String>,
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        set_service_state(&mut route.snapshot, name, state, detail);
        true
    })
    .unwrap_or(false)
}
