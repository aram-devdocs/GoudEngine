use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::state::{lock_runtime, RouteState};
use crate::core::debugger::snapshot::{
    FrameStateV1, MemorySummaryV1, ProfilerSampleV1, SnapshotStatsV1,
};
use crate::core::debugger::RuntimeRouteId;

const MAX_METRICS_FRAMES: usize = 300;
const MAX_METRICS_EVENTS: usize = 32;
const METRICS_TRACE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct MetricsTraceFrameV1 {
    pub frame: FrameStateV1,
    pub stats: SnapshotStatsV1,
    pub memory_summary: MemorySummaryV1,
    pub profiler_samples: Vec<ProfilerSampleV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MetricsTraceEventV1 {
    pub event: String,
    pub frame_index: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct MetricsTraceExportV1 {
    pub version: u32,
    pub route: RuntimeRouteId,
    pub frames: Vec<MetricsTraceFrameV1>,
    pub events: Vec<MetricsTraceEventV1>,
}

impl MetricsTraceExportV1 {
    pub(super) fn empty(route: RuntimeRouteId) -> Self {
        Self {
            version: METRICS_TRACE_VERSION,
            route,
            frames: Vec::new(),
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct RouteMetricsState {
    frames: VecDeque<MetricsTraceFrameV1>,
    events: VecDeque<MetricsTraceEventV1>,
}

impl RouteMetricsState {
    fn push_frame(&mut self, frame: MetricsTraceFrameV1) {
        self.frames.push_back(frame);
        while self.frames.len() > MAX_METRICS_FRAMES {
            self.frames.pop_front();
        }
    }

    fn push_event(&mut self, event: MetricsTraceEventV1) {
        self.events.push_back(event);
        while self.events.len() > MAX_METRICS_EVENTS {
            self.events.pop_front();
        }
    }
}

const MAX_DIAGNOSTICS_RECORDING_FRAMES: usize = 3600;

/// One frame captured during a diagnostics recording session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DiagnosticsRecordingFrameV1 {
    pub frame: FrameStateV1,
    pub stats: SnapshotStatsV1,
    pub memory_summary: MemorySummaryV1,
    pub provider_diagnostics: BTreeMap<String, serde_json::Value>,
}

/// Recording session state on RouteState.
#[derive(Debug, Clone, Default)]
pub(super) struct RouteDiagnosticsRecordingState {
    pub active: bool,
    pub max_frames: usize,
    pub duration_seconds: f32,
    pub start_total_seconds: f64,
    pub frames: Vec<DiagnosticsRecordingFrameV1>,
    pub recording_id: String,
}

/// One aggregated slice in the export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DiagnosticsSliceV1 {
    pub slice_index: u32,
    pub frame_range: [u64; 2],
    pub time_range: [f64; 2],
    pub frame_count: u32,
    pub avg_delta_seconds: f32,
    pub avg_fps: f32,
    pub stats_avg: SnapshotStatsV1,
    pub provider_diagnostics_avg: BTreeMap<String, serde_json::Value>,
}

/// Full recording export with slices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct DiagnosticsRecordingExportV1 {
    pub version: u32,
    pub route: RuntimeRouteId,
    pub recording_id: String,
    pub total_frames: u32,
    pub total_duration_seconds: f64,
    pub requested_slices: u32,
    pub slices: Vec<DiagnosticsSliceV1>,
}

pub(super) fn record_metrics_frame(route: &mut RouteState) {
    let frame_index = route.snapshot.frame.index;
    let frame = MetricsTraceFrameV1 {
        frame: route.snapshot.frame.clone(),
        stats: route.snapshot.stats.clone(),
        memory_summary: route.snapshot.memory_summary,
        profiler_samples: route.snapshot.profiler_samples.clone(),
    };
    route.metrics.push_frame(frame);
    route.metrics.push_event(MetricsTraceEventV1 {
        event: "frame_recorded".to_string(),
        frame_index,
    });
}

pub(super) fn metrics_trace_json_for_route(route_id: &RuntimeRouteId) -> Option<String> {
    let guard = lock_runtime();
    let runtime = guard.as_ref()?;
    let route = runtime.routes.get(&route_id.context_id)?;
    let export = MetricsTraceExportV1 {
        version: METRICS_TRACE_VERSION,
        route: route.snapshot.route_id.clone(),
        frames: route.metrics.frames.iter().cloned().collect(),
        events: route.metrics.events.iter().cloned().collect(),
    };
    serde_json::to_string(&export).ok()
}

pub(super) fn empty_metrics_export(route_id: &RuntimeRouteId) -> MetricsTraceExportV1 {
    MetricsTraceExportV1::empty(route_id.clone())
}

pub(super) fn start_diagnostics_recording(
    route: &mut RouteState,
    duration_seconds: f32,
    max_frames: usize,
) -> String {
    let recording_id = format!(
        "diag-rec-{}-{}",
        route.snapshot.route_id.context_id, route.snapshot.frame.index
    );
    route.diagnostics_recording = RouteDiagnosticsRecordingState {
        active: true,
        max_frames: max_frames.min(MAX_DIAGNOSTICS_RECORDING_FRAMES),
        duration_seconds,
        start_total_seconds: route.snapshot.frame.total_seconds,
        frames: Vec::new(),
        recording_id: recording_id.clone(),
    };
    recording_id
}

pub(super) fn stop_diagnostics_recording(route: &mut RouteState) -> Option<String> {
    if !route.diagnostics_recording.active {
        return None;
    }
    route.diagnostics_recording.active = false;
    Some(route.diagnostics_recording.recording_id.clone())
}

pub(super) fn record_diagnostics_frame(route: &mut RouteState) {
    if !route.diagnostics_recording.active {
        return;
    }

    // Auto-stop if duration exceeded
    if route.diagnostics_recording.duration_seconds > 0.0 {
        let elapsed =
            route.snapshot.frame.total_seconds - route.diagnostics_recording.start_total_seconds;
        if elapsed >= route.diagnostics_recording.duration_seconds as f64 {
            route.diagnostics_recording.active = false;
            return;
        }
    }

    // Auto-stop if max frames reached
    if route.diagnostics_recording.frames.len() >= route.diagnostics_recording.max_frames {
        route.diagnostics_recording.active = false;
        return;
    }

    route
        .diagnostics_recording
        .frames
        .push(DiagnosticsRecordingFrameV1 {
            frame: route.snapshot.frame.clone(),
            stats: route.snapshot.stats.clone(),
            memory_summary: route.snapshot.memory_summary,
            provider_diagnostics: route.snapshot.provider_diagnostics.clone(),
        });
}

pub(super) fn export_diagnostics_recording_sliced(
    route: &RouteState,
    slice_count: u32,
) -> Option<DiagnosticsRecordingExportV1> {
    let recording = &route.diagnostics_recording;
    if recording.frames.is_empty() {
        return None;
    }

    let total_frames = recording.frames.len();
    let slice_count = (slice_count as usize).clamp(1, total_frames.min(1000));

    let mut slices = Vec::with_capacity(slice_count);
    for i in 0..slice_count {
        let start = i * total_frames / slice_count;
        let end = (i + 1) * total_frames / slice_count;
        let group = &recording.frames[start..end];
        if group.is_empty() {
            continue;
        }

        let frame_count = group.len() as u32;
        let avg_delta: f32 =
            group.iter().map(|f| f.frame.delta_seconds).sum::<f32>() / frame_count as f32;

        let stats_avg = average_stats(group);
        let provider_diagnostics_avg = average_provider_diagnostics(group);

        slices.push(DiagnosticsSliceV1 {
            slice_index: i as u32,
            frame_range: [
                group.first().unwrap().frame.index,
                group.last().unwrap().frame.index,
            ],
            time_range: [
                group.first().unwrap().frame.total_seconds,
                group.last().unwrap().frame.total_seconds,
            ],
            frame_count,
            avg_delta_seconds: avg_delta,
            avg_fps: if avg_delta > 0.0 {
                1.0 / avg_delta
            } else {
                0.0
            },
            stats_avg,
            provider_diagnostics_avg,
        });
    }

    let first_frame = &recording.frames.first().unwrap().frame;
    let last_frame = &recording.frames.last().unwrap().frame;
    let total_duration = last_frame.total_seconds - first_frame.total_seconds;

    Some(DiagnosticsRecordingExportV1 {
        version: 1,
        route: route.snapshot.route_id.clone(),
        recording_id: recording.recording_id.clone(),
        total_frames: total_frames as u32,
        total_duration_seconds: total_duration,
        requested_slices: slices.len() as u32,
        slices,
    })
}

pub(super) fn diagnostics_recording_status(route: &RouteState) -> serde_json::Value {
    serde_json::json!({
        "active": route.diagnostics_recording.active,
        "frame_count": route.diagnostics_recording.frames.len(),
        "recording_id": route.diagnostics_recording.recording_id,
    })
}

fn average_stats(group: &[DiagnosticsRecordingFrameV1]) -> SnapshotStatsV1 {
    let n = group.len() as u32;
    if n == 0 {
        return SnapshotStatsV1::default();
    }
    let mut result = SnapshotStatsV1::default();
    for frame in group {
        result.render.draw_calls += frame.stats.render.draw_calls;
        result.render.triangles += frame.stats.render.triangles;
        result.render.texture_binds += frame.stats.render.texture_binds;
        result.render.shader_binds += frame.stats.render.shader_binds;
    }
    result.render.draw_calls /= n;
    result.render.triangles /= n;
    result.render.texture_binds /= n;
    result.render.shader_binds /= n;

    // Use last frame's memory stats (not averaged)
    if let Some(last) = group.last() {
        result.memory = last.stats.memory;
        result.network = last.stats.network;
    }
    result
}

fn average_provider_diagnostics(
    group: &[DiagnosticsRecordingFrameV1],
) -> BTreeMap<String, serde_json::Value> {
    if group.is_empty() {
        return BTreeMap::new();
    }

    // Collect all keys from the group
    let mut all_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for frame in group {
        for key in frame.provider_diagnostics.keys() {
            all_keys.insert(key.clone());
        }
    }

    let mut result = BTreeMap::new();
    for key in &all_keys {
        let values: Vec<&serde_json::Value> = group
            .iter()
            .filter_map(|f| f.provider_diagnostics.get(key))
            .collect();
        if values.is_empty() {
            continue;
        }
        // Try to average numeric leaf values; for non-numeric, take last
        if let Some(averaged) = try_average_json_objects(&values) {
            result.insert(key.clone(), averaged);
        } else if let Some(last) = values.last() {
            result.insert(key.clone(), (*last).clone());
        }
    }
    result
}

fn try_average_json_objects(values: &[&serde_json::Value]) -> Option<serde_json::Value> {
    // If all values are objects, try to average their numeric fields
    if values.iter().all(|v| v.is_object()) {
        let mut result = serde_json::Map::new();
        let mut all_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for v in values {
            if let serde_json::Value::Object(obj) = v {
                for key in obj.keys() {
                    all_keys.insert(key.clone());
                }
            }
        }
        for key in &all_keys {
            let field_values: Vec<&serde_json::Value> =
                values.iter().filter_map(|v| v.get(key)).collect();
            if field_values.is_empty() {
                continue;
            }
            // Try numeric averaging
            let numeric_values: Vec<f64> = field_values.iter().filter_map(|v| v.as_f64()).collect();
            if numeric_values.len() == field_values.len() && !numeric_values.is_empty() {
                let avg = numeric_values.iter().sum::<f64>() / numeric_values.len() as f64;
                result.insert(key.clone(), serde_json::json!(avg));
            } else {
                // Non-numeric: take last value
                if let Some(last) = field_values.last() {
                    result.insert(key.clone(), (*last).clone());
                }
            }
        }
        Some(serde_json::Value::Object(result))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::debugger::types::RuntimeSurfaceKind;

    fn test_route_state() -> RouteState {
        let route_id = RuntimeRouteId {
            process_nonce: 1,
            context_id: 1,
            surface_kind: RuntimeSurfaceKind::HeadlessContext,
        };
        let config = crate::core::debugger::config::DebuggerConfig::default();
        super::super::state::initialize_route_state(
            route_id,
            RuntimeSurfaceKind::HeadlessContext,
            &config,
        )
    }

    fn push_test_frame(route: &mut RouteState, index: u64, total_seconds: f64) {
        route.snapshot.frame.index = index;
        route.snapshot.frame.delta_seconds = 1.0 / 60.0;
        route.snapshot.frame.total_seconds = total_seconds;
        route.snapshot.stats.render.draw_calls = 10;
        route.snapshot.stats.render.triangles = 100;
        route.snapshot.provider_diagnostics.insert(
            "render".to_string(),
            serde_json::json!({"draw_calls": 10, "triangles": 100}),
        );
        record_diagnostics_frame(route);
    }

    #[test]
    fn test_diagnostics_recording_captures_frames() {
        let mut route = test_route_state();
        let recording_id = start_diagnostics_recording(&mut route, 0.0, 3600);
        assert!(!recording_id.is_empty());
        assert!(route.diagnostics_recording.active);

        for i in 0..60 {
            push_test_frame(&mut route, i, i as f64 / 60.0);
        }
        assert_eq!(route.diagnostics_recording.frames.len(), 60);

        stop_diagnostics_recording(&mut route);
        assert!(!route.diagnostics_recording.active);

        let export = export_diagnostics_recording_sliced(&route, 10).unwrap();
        assert_eq!(export.slices.len(), 10);
        assert_eq!(export.total_frames, 60);
    }

    #[test]
    fn test_diagnostics_recording_auto_stops_on_duration() {
        let mut route = test_route_state();
        start_diagnostics_recording(&mut route, 1.0, 3600);

        // Push frames within duration
        for i in 0..30 {
            push_test_frame(&mut route, i, i as f64 / 60.0);
        }
        assert!(route.diagnostics_recording.active);

        // Push a frame past duration
        push_test_frame(&mut route, 60, 1.5);
        assert!(!route.diagnostics_recording.active);
        // The frame that caused the stop is NOT recorded
        assert_eq!(route.diagnostics_recording.frames.len(), 30);
    }

    #[test]
    fn test_diagnostics_recording_caps_at_max_frames() {
        let mut route = test_route_state();
        start_diagnostics_recording(&mut route, 0.0, 20);

        for i in 0..50 {
            push_test_frame(&mut route, i, i as f64 / 60.0);
        }
        // Should have auto-stopped at 20
        assert!(!route.diagnostics_recording.active);
        assert_eq!(route.diagnostics_recording.frames.len(), 20);
    }

    #[test]
    fn test_slicing_math_distributes_evenly() {
        let mut route = test_route_state();
        start_diagnostics_recording(&mut route, 0.0, 3600);

        for i in 0..100 {
            push_test_frame(&mut route, i, i as f64 / 60.0);
        }
        stop_diagnostics_recording(&mut route);

        let export = export_diagnostics_recording_sliced(&route, 10).unwrap();
        assert_eq!(export.slices.len(), 10);
        for slice in &export.slices {
            assert_eq!(slice.frame_count, 10);
        }
    }

    #[test]
    fn test_slicing_handles_remainder() {
        let mut route = test_route_state();
        start_diagnostics_recording(&mut route, 0.0, 3600);

        for i in 0..103 {
            push_test_frame(&mut route, i, i as f64 / 60.0);
        }
        stop_diagnostics_recording(&mut route);

        let export = export_diagnostics_recording_sliced(&route, 10).unwrap();
        assert_eq!(export.slices.len(), 10);
        let total: u32 = export.slices.iter().map(|s| s.frame_count).sum();
        assert_eq!(total, 103);
        // Each slice should have 10 or 11 frames
        for slice in &export.slices {
            assert!(slice.frame_count >= 10 && slice.frame_count <= 11);
        }
    }
}
