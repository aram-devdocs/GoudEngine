use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourceTemplatesResult,
    ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
    ServerCapabilities, ServerInfo,
};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde_json::{json, Value};

use crate::attach_client::{AttachClient, AttachError, AttachedRoute};
use crate::discovery::{self, ArtifactKind, DiscoveredContext};
use crate::prompts;
use crate::resources;

mod types;

use self::types::{
    structured_response, AttachContextParams, BridgeResponse, GetDiagnosticsRecordingParams,
    GetLogsParams, GetSubsystemDiagnosticsParams, InjectInputParams, InspectEntityParams,
    McpDebuggerStepKind, RecordDiagnosticsParams, SetPausedParams, SetTimeScaleParams,
    StartReplayParams, StepParams,
};

struct BridgeState {
    runtime_root: std::path::PathBuf,
    attached: Option<AttachedRoute>,
}

#[derive(Clone)]
pub struct GoudEngineMcpServer {
    state: Arc<Mutex<BridgeState>>,
    tool_router: ToolRouter<Self>,
}

impl GoudEngineMcpServer {
    pub fn new() -> Self {
        Self::with_runtime_root(discovery::default_runtime_root())
    }

    pub fn with_runtime_root(runtime_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            state: Arc::new(Mutex::new(BridgeState {
                runtime_root: runtime_root.into(),
                attached: None,
            })),
            tool_router: Self::tool_router(),
        }
    }

    fn state(&self) -> Result<MutexGuard<'_, BridgeState>, McpError> {
        self.state
            .lock()
            .map_err(|_| McpError::internal_error("bridge state lock poisoned", None))
    }

    fn current_attached_summary(state: &BridgeState) -> Option<Value> {
        let attached = state.attached.as_ref()?;
        Some(json!({
            "pid": attached.manifest.pid,
            "process_nonce": attached.manifest.process_nonce,
            "published_at_unix_ms": attached.manifest.published_at_unix_ms,
            "endpoint": attached.manifest.endpoint,
            "route": attached.route,
            "session": attached.accepted,
        }))
    }

    fn request_attached(&self, request: Value) -> Result<Value, McpError> {
        let mut state = self.state()?;
        let attached = state
            .attached
            .as_mut()
            .ok_or_else(|| McpError::invalid_request("no context is attached", None))?;

        match attached.client.request(&request) {
            Ok(result) => Ok(result),
            Err(error) => {
                if should_detach(&error) {
                    state.attached = None;
                }
                Err(map_attach_error(error))
            }
        }
    }

    fn with_selection_restored(&self, entity_id: u64) -> Result<Value, McpError> {
        let original_snapshot = self.request_attached(json!({ "verb": "get_snapshot" }))?;
        let original_entity = original_snapshot
            .get("selection")
            .and_then(|selection| selection.get("entity_id"))
            .and_then(Value::as_u64);

        self.request_attached(json!({
            "verb": "set_selected_entity",
            "entity_id": entity_id,
        }))?;
        let detailed_snapshot = self.request_attached(json!({ "verb": "get_snapshot" }))?;

        if let Some(previous) = original_entity {
            let _ = self.request_attached(json!({
                "verb": "set_selected_entity",
                "entity_id": previous,
            }));
        } else {
            let _ = self.request_attached(json!({ "verb": "clear_selected_entity" }));
        }

        Ok(detailed_snapshot)
    }

    fn replay_bytes(&self, params: StartReplayParams) -> Result<Vec<u8>, McpError> {
        if let Some(artifact_id) = params.artifact_id {
            let state = self.state()?;
            return discovery::recording_bytes(&state.runtime_root, &artifact_id)
                .map_err(|err| McpError::invalid_request(err.to_string(), None));
        }

        if let Some(resource_uri) = params.resource_uri {
            let artifact_id =
                discovery::artifact_id_from_uri(&resource_uri, ArtifactKind::Recording)
                    .ok_or_else(|| {
                        McpError::invalid_request("invalid recording resource URI", None)
                    })?;
            let state = self.state()?;
            return discovery::recording_bytes(&state.runtime_root, &artifact_id)
                .map_err(|err| McpError::invalid_request(err.to_string(), None));
        }

        if let Some(data_base64) = params.data_base64 {
            return BASE64_STANDARD.decode(data_base64).map_err(|err| {
                McpError::invalid_request(format!("invalid replay base64: {err}"), None)
            });
        }

        Err(McpError::invalid_request(
            "start_replay requires artifactId, resourceUri, or dataBase64",
            None,
        ))
    }
}

#[tool_router(router = tool_router)]
impl GoudEngineMcpServer {
    #[tool(
        name = "goudengine.list_contexts",
        description = "Discover debugger contexts published by local GoudEngine processes."
    )]
    pub async fn list_contexts(&self) -> Result<Json<BridgeResponse>, McpError> {
        let state = self.state()?;
        let contexts = discovery::discover_contexts(&state.runtime_root);
        structured_response(json!({
            "contexts": contexts,
            "attached_context": Self::current_attached_summary(&state),
        }))
    }

    #[tool(
        name = "goudengine.attach_context",
        description = "Attach the MCP bridge to one route-scoped debugger context."
    )]
    pub async fn attach_context(
        &self,
        Parameters(params): Parameters<AttachContextParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let mut state = self.state()?;
        let context =
            discovery::find_context(&state.runtime_root, params.context_id, params.process_nonce)
                .map_err(|err| McpError::invalid_request(err.to_string(), None))?;
        let manifest = goud_engine::core::debugger::RuntimeManifestV1 {
            manifest_version: 1,
            pid: context.pid,
            process_nonce: context.process_nonce,
            executable: context.executable.clone(),
            endpoint: context.endpoint.clone(),
            routes: vec![context.route.clone()],
            published_at_unix_ms: context.published_at_unix_ms,
        };
        let attached =
            AttachClient::connect(manifest, context.route.clone()).map_err(map_attach_error)?;
        let response = json!({
            "context": context,
            "session": attached.accepted,
        });
        state.attached = Some(attached);
        structured_response(response)
    }

    #[tool(
        name = "goudengine.get_snapshot",
        description = "Fetch the current debugger snapshot for the attached route."
    )]
    pub async fn get_snapshot(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({ "verb": "get_snapshot" }))?)
    }

    #[tool(
        name = "goudengine.inspect_entity",
        description = "Select one entity, fetch its expanded debugger snapshot entry, then restore the previous selection."
    )]
    pub async fn inspect_entity(
        &self,
        Parameters(params): Parameters<InspectEntityParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let snapshot = self.with_selection_restored(params.entity_id)?;
        let entity = snapshot
            .get("entities")
            .and_then(Value::as_array)
            .and_then(|entities| {
                entities.iter().find(|entity| {
                    entity.get("entity_id").and_then(Value::as_u64) == Some(params.entity_id)
                })
            })
            .cloned()
            .ok_or_else(|| {
                McpError::invalid_request("entity was not present in the debugger snapshot", None)
            })?;
        structured_response(json!({
            "entity": entity,
            "snapshot": snapshot,
        }))
    }

    #[tool(
        name = "goudengine.get_metrics_trace",
        description = "Export the current versioned debugger metrics trace for the attached route."
    )]
    pub async fn get_metrics_trace(&self) -> Result<Json<BridgeResponse>, McpError> {
        let mut result = self.request_attached(json!({ "verb": "get_metrics_trace" }))?;
        resources::add_resource_uri(ArtifactKind::Metrics, &mut result);
        structured_response(result)
    }

    #[tool(
        name = "goudengine.capture_frame",
        description = "Capture the current framebuffer plus debugger metadata attachments for the attached route."
    )]
    pub async fn capture_frame(&self) -> Result<Json<BridgeResponse>, McpError> {
        let mut result = self.request_attached(json!({ "verb": "capture_frame" }))?;
        resources::add_resource_uri(ArtifactKind::Capture, &mut result);
        structured_response(result)
    }

    #[tool(
        name = "goudengine.start_recording",
        description = "Start normalized input recording for the attached route."
    )]
    pub async fn start_recording(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({ "verb": "start_recording" }))?)
    }

    #[tool(
        name = "goudengine.stop_recording",
        description = "Stop recording and export the replay artifact for the attached route."
    )]
    pub async fn stop_recording(&self) -> Result<Json<BridgeResponse>, McpError> {
        let mut result = self.request_attached(json!({ "verb": "stop_recording" }))?;
        resources::add_resource_uri(ArtifactKind::Recording, &mut result);
        structured_response(result)
    }

    #[tool(
        name = "goudengine.start_replay",
        description = "Start replay for the attached route from a stored recording artifact or base64 payload."
    )]
    pub async fn start_replay(
        &self,
        Parameters(params): Parameters<StartReplayParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let bytes = self.replay_bytes(params)?;
        structured_response(self.request_attached(json!({
            "verb": "start_replay",
            "data": bytes,
        }))?)
    }

    #[tool(
        name = "goudengine.stop_replay",
        description = "Stop replay for the attached route."
    )]
    pub async fn stop_replay(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({ "verb": "stop_replay" }))?)
    }

    #[tool(
        name = "goudengine.set_paused",
        description = "Pause or resume the attached route."
    )]
    pub async fn set_paused(
        &self,
        Parameters(params): Parameters<SetPausedParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({
            "verb": "set_paused",
            "paused": params.paused,
        }))?)
    }

    #[tool(
        name = "goudengine.step",
        description = "Spend frame or tick debugger step budget on the attached route."
    )]
    pub async fn step(
        &self,
        Parameters(params): Parameters<StepParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let request = match params.kind {
            McpDebuggerStepKind::Frame => json!({ "verb": "step", "frames": params.count }),
            McpDebuggerStepKind::Tick => {
                json!({ "verb": "step", "frames": 0, "ticks": params.count })
            }
        };
        structured_response(self.request_attached(request)?)
    }

    #[tool(
        name = "goudengine.set_time_scale",
        description = "Set the debugger-owned time scale on the attached route."
    )]
    pub async fn set_time_scale(
        &self,
        Parameters(params): Parameters<SetTimeScaleParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({
            "verb": "set_time_scale",
            "time_scale": params.scale,
        }))?)
    }

    #[tool(
        name = "goudengine.inject_input",
        description = "Inject one or more normalized debugger input events into the attached route."
    )]
    pub async fn inject_input(
        &self,
        Parameters(params): Parameters<InjectInputParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let events: Vec<Value> = params
            .events
            .into_iter()
            .map(|event| {
                json!({
                    "device": event.device,
                    "action": event.action,
                    "key": event.key,
                    "button": event.button,
                    "position": event.position,
                    "delta": event.delta,
                })
            })
            .collect();
        structured_response(self.request_attached(json!({
            "verb": "inject_input",
            "events": events,
        }))?)
    }

    #[tool(
        name = "goudengine.get_diagnostics",
        description = "Return the full provider diagnostics map from the current snapshot for the attached route."
    )]
    pub async fn get_diagnostics(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({ "verb": "get_diagnostics" }))?)
    }

    #[tool(
        name = "goudengine.get_subsystem_diagnostics",
        description = "Return diagnostics for a single subsystem key (e.g. render, physics_2d, audio, input, sprite_batch, assets, window)."
    )]
    pub async fn get_subsystem_diagnostics(
        &self,
        Parameters(params): Parameters<GetSubsystemDiagnosticsParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({
            "verb": "get_diagnostics_for",
            "key": params.key,
        }))?)
    }

    #[tool(
        name = "goudengine.get_logs",
        description = "Return recent engine log entries, optionally filtered to entries since a given frame number."
    )]
    pub async fn get_logs(
        &self,
        Parameters(params): Parameters<GetLogsParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        let mut request = json!({ "verb": "get_logs" });
        if let Some(since_frame) = params.since_frame {
            request["since_frame"] = json!(since_frame);
        }
        structured_response(self.request_attached(request)?)
    }

    #[tool(
        name = "goudengine.get_scene_hierarchy",
        description = "Return entities with parent/child relationships for the attached route."
    )]
    pub async fn get_scene_hierarchy(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({ "verb": "get_scene_hierarchy" }))?)
    }

    #[tool(
        name = "goudengine.record_diagnostics",
        description = "Record diagnostics for a specified duration then return time-sliced aggregated results. Blocks until recording completes."
    )]
    pub async fn record_diagnostics(
        &self,
        Parameters(params): Parameters<RecordDiagnosticsParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        // Start recording
        let start_result = self.request_attached(json!({
            "verb": "start_diagnostics_recording",
            "duration_seconds": params.duration_seconds,
        }))?;

        // Wait for the recording duration
        let duration = if params.duration_seconds > 0.0 {
            params.duration_seconds
        } else {
            return Err(McpError::invalid_request(
                "record_diagnostics requires a positive duration_seconds",
                None,
            ));
        };
        tokio::time::sleep(std::time::Duration::from_secs_f32(duration + 0.1)).await;

        // Stop recording
        let _ = self.request_attached(json!({ "verb": "stop_diagnostics_recording" }))?;

        // Get the sliced export
        let export = self.request_attached(json!({
            "verb": "get_diagnostics_recording",
            "slice_count": params.slice_count,
        }))?;

        structured_response(json!({
            "start": start_result,
            "export": export,
        }))
    }

    #[tool(
        name = "goudengine.get_diagnostics_recording",
        description = "Retrieve a previously recorded diagnostics session as time-sliced aggregated data. Use after manual start/stop workflow."
    )]
    pub async fn get_diagnostics_recording(
        &self,
        Parameters(params): Parameters<GetDiagnosticsRecordingParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(self.request_attached(json!({
            "verb": "get_diagnostics_recording",
            "slice_count": params.slice_count,
        }))?)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GoudEngineMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .enable_resources()
                .build(),
        )
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::get_prompt_result(&request.name).ok_or_else(|| {
            McpError::invalid_request(format!("unknown prompt: {}", request.name), None)
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: prompts::static_prompts(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: resources::static_resources(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: resources::resource_templates(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let state = self.state()?;
        let contents = resources::read_resource(&request.uri, &state.runtime_root)
            .map_err(|err| McpError::resource_not_found(err.to_string(), None))?;
        Ok(ReadResourceResult::new(contents))
    }
}

fn should_detach(error: &AttachError) -> bool {
    matches!(
        error,
        AttachError::Io(_) | AttachError::Handshake(_) | AttachError::Protocol(_)
    ) || matches!(error, AttachError::Debugger { code, .. } if code == "route_not_found" || code == "attach_disabled")
}

fn map_attach_error(error: AttachError) -> McpError {
    match error {
        AttachError::Debugger {
            code,
            message,
            data,
        } => match code.as_str() {
            "unsupported" | "protocol_error" => McpError::invalid_request(message, data),
            "route_not_found" | "attach_disabled" => McpError::invalid_request(message, data),
            "capture_failed" => McpError::internal_error(message, data),
            _ => McpError::internal_error(message, data),
        },
        other => McpError::internal_error(other.to_string(), None),
    }
}

#[cfg(test)]
mod tests;

impl From<DiscoveredContext> for Value {
    fn from(context: DiscoveredContext) -> Self {
        json!(context)
    }
}
