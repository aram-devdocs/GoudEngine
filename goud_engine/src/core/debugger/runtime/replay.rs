use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::super::types::CapabilityStateV1;
use super::state::{set_route_capability, set_service_state, RouteState, SyntheticInputEventV1};

const REPLAY_MANIFEST_VERSION: u32 = 1;
const REPLAY_FORMAT: &str = "goud.replay.v1";
const REPLAY_DETERMINISM_NOTE: &str =
    "normalized input timing only; physics and render output can still diverge by platform/frame pacing";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct RecordedInputEventV1 {
    pub frame_index: u64,
    pub event: SyntheticInputEventV1,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(super) struct ReplayExportEnvelopeV1 {
    pub manifest_json: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
struct ReplayManifestV1 {
    manifest_version: u32,
    format: String,
    event_count: usize,
    determinism: String,
}

#[derive(Debug, Clone, Default)]
pub(super) struct RouteReplayState {
    pub(super) recording_active: bool,
    pub(super) recorded_events: Vec<RecordedInputEventV1>,
    pub(super) replay_events_by_frame: BTreeMap<u64, VecDeque<SyntheticInputEventV1>>,
    pub(super) replay_source_event_count: usize,
}

impl RouteReplayState {
    pub(super) fn is_replay_active(&self) -> bool {
        !self.replay_events_by_frame.is_empty()
    }

    fn queued_replay_event_count(&self) -> usize {
        self.replay_events_by_frame
            .values()
            .map(VecDeque::len)
            .sum()
    }
}

fn replay_mode(state: &RouteReplayState) -> &'static str {
    if state.recording_active {
        "recording"
    } else if state.is_replay_active() {
        "replay"
    } else {
        "idle"
    }
}

fn replay_detail(state: &RouteReplayState) -> String {
    if state.recording_active {
        return format!(
            "recording {} normalized events for next-frame playback; {REPLAY_DETERMINISM_NOTE}",
            state.recorded_events.len()
        );
    }
    if state.is_replay_active() {
        return format!(
            "queued {} replay events across {} target frames; {REPLAY_DETERMINISM_NOTE}",
            state.queued_replay_event_count(),
            state.replay_events_by_frame.len(),
        );
    }
    format!("idle; {REPLAY_DETERMINISM_NOTE}")
}

fn sync_replay_health(route: &mut RouteState) {
    let detail = replay_detail(&route.replay);
    set_route_capability(route, "replay", CapabilityStateV1::Ready);
    set_service_state(
        &mut route.snapshot,
        "replay",
        CapabilityStateV1::Ready,
        Some(detail),
    );
}

pub(super) fn replay_status_json(route: &RouteState) -> Value {
    json!({
        "mode": replay_mode(&route.replay),
        "detail": replay_detail(&route.replay),
        "recorded_event_count": route.replay.recorded_events.len(),
        "queued_replay_frames": route.replay.replay_events_by_frame.len(),
        "queued_replay_events": route.replay.queued_replay_event_count(),
    })
}

pub(super) fn start_recording(route: &mut RouteState) {
    route.replay.recording_active = true;
    route.replay.recorded_events.clear();
    route.replay.replay_events_by_frame.clear();
    route.replay.replay_source_event_count = 0;
    sync_replay_health(route);
}

pub(super) fn stop_recording(route: &mut RouteState) -> ReplayExportEnvelopeV1 {
    route.replay.recording_active = false;

    let manifest = ReplayManifestV1 {
        manifest_version: REPLAY_MANIFEST_VERSION,
        format: REPLAY_FORMAT.to_string(),
        event_count: route.replay.recorded_events.len(),
        determinism: REPLAY_DETERMINISM_NOTE.to_string(),
    };
    let manifest_json = serde_json::to_string(&manifest)
        .unwrap_or_else(|_| "{\"manifest_version\":1,\"format\":\"goud.replay.v1\"}".to_string());
    let data = serde_json::to_vec(&route.replay.recorded_events).unwrap_or_else(|_| b"[]".to_vec());
    sync_replay_health(route);
    ReplayExportEnvelopeV1 {
        manifest_json,
        data,
    }
}

pub(super) fn start_replay(route: &mut RouteState, data: &[u8]) -> Result<(), String> {
    let events: Vec<RecordedInputEventV1> =
        serde_json::from_slice(data).map_err(|err| format!("invalid replay data: {err}"))?;

    let mut grouped: BTreeMap<u64, VecDeque<SyntheticInputEventV1>> = BTreeMap::new();
    for recorded in events {
        grouped
            .entry(recorded.frame_index)
            .or_default()
            .push_back(recorded.event);
    }

    route.replay.recording_active = false;
    route.replay.replay_source_event_count = grouped.values().map(VecDeque::len).sum();
    route.replay.replay_events_by_frame = grouped;
    sync_replay_health(route);
    Ok(())
}

pub(super) fn stop_replay(route: &mut RouteState) {
    route.replay.replay_events_by_frame.clear();
    route.replay.replay_source_event_count = 0;
    sync_replay_health(route);
}

pub(super) fn take_replay_events_for_frame(
    route: &mut RouteState,
    target_frame_index: u64,
) -> Vec<SyntheticInputEventV1> {
    let events: Vec<SyntheticInputEventV1> = route
        .replay
        .replay_events_by_frame
        .remove(&target_frame_index)
        .map(|queue| queue.into_iter().collect())
        .unwrap_or_default();
    if !events.is_empty() || !route.replay.is_replay_active() {
        sync_replay_health(route);
    }
    events
}

pub(super) fn record_normalized_input_event(route: &mut RouteState, event: SyntheticInputEventV1) {
    if !route.replay.recording_active {
        return;
    }

    route.replay.recorded_events.push(RecordedInputEventV1 {
        frame_index: route.snapshot.frame.index.saturating_add(1),
        event,
    });
    sync_replay_health(route);
}
