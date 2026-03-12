use std::collections::VecDeque;

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
