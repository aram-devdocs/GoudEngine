//! Process-local debugger runtime, snapshot schema, and route discovery types.
//!
//! This module exposes the public debugger surface used by the Rust SDK, FFI,
//! and desktop SDK bindings.

#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

mod config;
pub mod log_capture;
mod runtime;
mod snapshot;
mod types;

#[doc(inline)]
pub use config::ContextConfig;
#[doc(inline)]
pub use config::DebuggerConfig;
#[doc(inline)]
pub use log_capture::LogCaptureSink;
#[doc(inline)]
pub use log_capture::LogEntryV1;
#[doc(inline)]
pub use runtime::active_route_count;
#[doc(inline)]
pub use runtime::begin_frame;
#[doc(inline)]
pub use runtime::capture_frame_for_route;
#[doc(inline)]
pub use runtime::clear_debug_draw_transient_for_route;
#[doc(inline)]
pub use runtime::control_state_for_route;
#[doc(inline)]
pub use runtime::current_manifest;
#[doc(inline)]
pub use runtime::current_route;
#[doc(inline)]
pub use runtime::debug_draw_payload_for_route;
#[doc(inline)]
pub use runtime::debug_draw_shapes_2d_for_context;
#[doc(inline)]
pub use runtime::debug_draw_shapes_2d_for_route;
#[doc(inline)]
pub use runtime::dispatch_request_json_for_route;
#[doc(inline)]
pub use runtime::end_frame;
#[doc(inline)]
pub use runtime::fps_stats_for_context;
#[doc(inline)]
pub use runtime::get_memory_summary_for_context;
#[doc(inline)]
pub use runtime::profiler_enabled_for_context;
#[doc(inline)]
pub use runtime::profiler_enabled_for_route;
#[doc(inline)]
pub use runtime::record_phase_duration;
#[doc(inline)]
pub use runtime::record_synthetic_input_for_current_route;
#[doc(inline)]
pub use runtime::register_capture_hook_for_route;
#[doc(inline)]
pub use runtime::register_context;
#[doc(inline)]
pub use runtime::register_snapshot_refresh_hook_for_route;
#[doc(inline)]
pub use runtime::replace_provider_debug_draw_2d_for_context;
#[doc(inline)]
pub use runtime::replace_provider_debug_draw_2d_for_route;
#[doc(inline)]
pub use runtime::replace_provider_debug_draw_3d_for_context;
#[doc(inline)]
pub use runtime::replace_provider_debug_draw_3d_for_route;
#[doc(inline)]
pub use runtime::route_for_context;
#[doc(inline)]
pub use runtime::scoped_route;
#[doc(inline)]
pub use runtime::set_profiling_enabled_for_context;
#[doc(inline)]
pub use runtime::set_selected_entity_for_context;
#[doc(inline)]
pub use runtime::set_service_state_for_context;
#[doc(inline)]
pub use runtime::set_snapshot_network_stats_for_context;
#[doc(inline)]
pub use runtime::set_system_sample;
#[doc(inline)]
pub use runtime::snapshot_for_context;
#[doc(inline)]
pub use runtime::snapshot_for_route;
#[doc(inline)]
pub use runtime::snapshot_json_for_context;
#[doc(inline)]
pub use runtime::take_frame_control_for_route;
#[doc(inline)]
pub use runtime::unregister_capture_hook_for_route;
#[doc(inline)]
pub use runtime::unregister_context;
#[doc(inline)]
pub use runtime::unregister_snapshot_refresh_hook_for_route;
#[doc(inline)]
pub use runtime::update_fps_stats_for_context;
#[doc(inline)]
pub use runtime::update_memory_category_for_context;
#[doc(inline)]
pub use runtime::update_render_stats_for_context;
#[doc(inline)]
pub use runtime::with_snapshot_mut;
#[doc(inline)]
pub use runtime::AttachAcceptedV1;
#[doc(inline)]
pub use runtime::AttachHelloV1;
#[doc(inline)]
pub use runtime::DebugDrawPayloadV1;
#[doc(inline)]
pub use runtime::DebugDrawShape2DV1;
#[doc(inline)]
pub use runtime::DebugDrawShape3DV1;
#[doc(inline)]
pub use runtime::FrameControlPlanV1;
#[doc(inline)]
pub use runtime::RawFramebufferReadbackV1;
#[doc(inline)]
pub use runtime::RouteControlStateV1;
#[doc(inline)]
pub use runtime::SyntheticInputEventV1;
#[doc(inline)]
pub use snapshot::default_capabilities;
#[doc(inline)]
pub use snapshot::default_services;
#[doc(inline)]
pub use snapshot::DebuggerSnapshotV1;
#[doc(inline)]
pub use snapshot::DebuggerStateV1;
#[doc(inline)]
pub use snapshot::DiagnosticsStateV1;
#[doc(inline)]
pub use snapshot::EntityStateV1;
#[doc(inline)]
pub use snapshot::FrameStateV1;
#[doc(inline)]
pub use snapshot::LocalEndpointV1;
#[doc(inline)]
pub use snapshot::MemoryCategoryStatsV1;
#[doc(inline)]
pub use snapshot::MemoryStatsV1;
#[doc(inline)]
pub use snapshot::MemorySummaryV1;
#[doc(inline)]
pub use snapshot::NetworkStatsV1;
#[doc(inline)]
pub use snapshot::ProfilerSampleV1;
#[doc(inline)]
pub use snapshot::RenderStatsV1;
#[doc(inline)]
pub use snapshot::RouteSummaryV1;
#[doc(inline)]
pub use snapshot::RuntimeManifestV1;
#[doc(inline)]
pub use snapshot::SceneStateV1;
#[doc(inline)]
pub use snapshot::SelectionStateV1;
#[doc(inline)]
pub use snapshot::ServiceHealthV1;
#[doc(inline)]
pub use snapshot::SnapshotStatsV1;
#[doc(inline)]
pub use types::CapabilityStateV1;
#[doc(inline)]
pub use types::RuntimeRouteId;
#[doc(inline)]
pub use types::RuntimeSurfaceKind;
#[doc(inline)]
pub use types::REQUIRED_SERVICE_OWNERS;
#[doc(inline)]
pub use types::ROUTE_CAPABILITY_KEYS;

#[cfg(test)]
pub(crate) use runtime::attach_hello_for_tests;
#[cfg(test)]
pub(crate) use runtime::attach_request_json_for_tests;
#[cfg(test)]
pub(crate) use runtime::attach_session_heartbeat_for_tests;
#[cfg(test)]
pub(crate) use runtime::manifest_artifact_path_for_tests;
#[cfg(test)]
pub(crate) use runtime::reset_for_tests;
#[cfg(test)]
pub(crate) use runtime::stop_attach_server_for_tests;

#[cfg(test)]
pub(crate) fn test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

#[cfg(test)]
mod tests;
