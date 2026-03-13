use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use serde_json::{json, Value};

use super::types::{
    structured_response, AttachContextParams, BridgeResponse, GetDiagnosticsRecordingParams,
    GetLogsParams, GetSubsystemDiagnosticsParams, InjectInputParams, InspectEntityParams,
    McpDebuggerStepKind, RecordDiagnosticsParams, SetPausedParams, SetTimeScaleParams,
    StartReplayParams, StepParams,
};
use super::GoudEngineMcpServer;
use crate::discovery::{self, ArtifactKind};
use crate::resources;

#[tool_router(router = tool_router)]
impl GoudEngineMcpServer {
    pub(super) fn create_tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::tool_router()
    }

    #[tool(
        name = "goudengine.list_contexts",
        description = "Discover debugger contexts published by local GoudEngine processes."
    )]
    pub async fn list_contexts(&self) -> Result<Json<BridgeResponse>, McpError> {
        let (contexts, attached_summary, ws_route_id) = {
            let state = self.state()?;
            let contexts = discovery::discover_contexts(&state.runtime_root);
            let attached_summary = Self::current_attached_summary(&state);
            let ws_route_id = state.ws_route_id.clone();
            (contexts, attached_summary, ws_route_id)
        };

        let ws_routes = self.ws_relay.list_routes().await;
        structured_response(json!({
            "contexts": contexts,
            "attached_context": attached_summary,
            "ws_route_id": ws_route_id,
            "browser_routes": ws_routes,
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
        // WebSocket browser route attach path.
        if let Some(ws_id) = params.ws_route_id {
            let ws_routes = self.ws_relay.list_routes().await;
            let route = ws_routes
                .iter()
                .find(|r| r.route_id == ws_id)
                .ok_or_else(|| {
                    McpError::invalid_request(
                        format!("browser route '{ws_id}' is not connected"),
                        None,
                    )
                })?;
            let mut state = self.state()?;
            state.attached = None;
            state.ws_route_id = Some(ws_id.clone());
            return structured_response(json!({
                "ws_route": route,
                "session": { "transport": "websocket" },
            }));
        }

        // Local Unix socket attach path.
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
        let attached = crate::attach_client::AttachClient::connect(manifest, context.route.clone())
            .map_err(super::map_attach_error)?;
        let response = json!({
            "context": context,
            "session": attached.accepted,
        });
        state.attached = Some(attached);
        state.ws_route_id = None;
        structured_response(response)
    }

    #[tool(
        name = "goudengine.get_snapshot",
        description = "Fetch the current debugger snapshot for the attached route."
    )]
    pub async fn get_snapshot(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({ "verb": "get_snapshot" }))
                .await?,
        )
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
        let mut result = self
            .request_attached_or_ws(json!({ "verb": "get_metrics_trace" }))
            .await?;
        resources::add_resource_uri(ArtifactKind::Metrics, &mut result);
        structured_response(result)
    }

    #[tool(
        name = "goudengine.capture_frame",
        description = "Capture the current framebuffer plus debugger metadata attachments for the attached route."
    )]
    pub async fn capture_frame(&self) -> Result<Json<BridgeResponse>, McpError> {
        let mut result = self
            .request_attached_or_ws(json!({ "verb": "capture_frame" }))
            .await?;
        resources::add_resource_uri(ArtifactKind::Capture, &mut result);
        structured_response(result)
    }

    #[tool(
        name = "goudengine.start_recording",
        description = "Start normalized input recording for the attached route."
    )]
    pub async fn start_recording(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({ "verb": "start_recording" }))
                .await?,
        )
    }

    #[tool(
        name = "goudengine.stop_recording",
        description = "Stop recording and export the replay artifact for the attached route."
    )]
    pub async fn stop_recording(&self) -> Result<Json<BridgeResponse>, McpError> {
        let mut result = self
            .request_attached_or_ws(json!({ "verb": "stop_recording" }))
            .await?;
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
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "start_replay",
                "data": bytes,
            }))
            .await?,
        )
    }

    #[tool(
        name = "goudengine.stop_replay",
        description = "Stop replay for the attached route."
    )]
    pub async fn stop_replay(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({ "verb": "stop_replay" }))
                .await?,
        )
    }

    #[tool(
        name = "goudengine.set_paused",
        description = "Pause or resume the attached route."
    )]
    pub async fn set_paused(
        &self,
        Parameters(params): Parameters<SetPausedParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "set_paused",
                "paused": params.paused,
            }))
            .await?,
        )
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
        structured_response(self.request_attached_or_ws(request).await?)
    }

    #[tool(
        name = "goudengine.set_time_scale",
        description = "Set the debugger-owned time scale on the attached route."
    )]
    pub async fn set_time_scale(
        &self,
        Parameters(params): Parameters<SetTimeScaleParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "set_time_scale",
                "time_scale": params.scale,
            }))
            .await?,
        )
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
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "inject_input",
                "events": events,
            }))
            .await?,
        )
    }

    #[tool(
        name = "goudengine.get_diagnostics",
        description = "Return the full provider diagnostics map from the current snapshot for the attached route."
    )]
    pub async fn get_diagnostics(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({ "verb": "get_diagnostics" }))
                .await?,
        )
    }

    #[tool(
        name = "goudengine.get_subsystem_diagnostics",
        description = "Return diagnostics for a single subsystem key (e.g. render, physics_2d, audio, input, sprite_batch, assets, window)."
    )]
    pub async fn get_subsystem_diagnostics(
        &self,
        Parameters(params): Parameters<GetSubsystemDiagnosticsParams>,
    ) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "get_diagnostics_for",
                "key": params.key,
            }))
            .await?,
        )
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
        structured_response(self.request_attached_or_ws(request).await?)
    }

    #[tool(
        name = "goudengine.get_scene_hierarchy",
        description = "Return entities with parent/child relationships for the attached route."
    )]
    pub async fn get_scene_hierarchy(&self) -> Result<Json<BridgeResponse>, McpError> {
        structured_response(
            self.request_attached_or_ws(json!({ "verb": "get_scene_hierarchy" }))
                .await?,
        )
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
        let start_result = self
            .request_attached_or_ws(json!({
                "verb": "start_diagnostics_recording",
                "duration_seconds": params.duration_seconds,
            }))
            .await?;

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
        let _ = self
            .request_attached_or_ws(json!({ "verb": "stop_diagnostics_recording" }))
            .await?;

        // Get the sliced export
        let export = self
            .request_attached_or_ws(json!({
                "verb": "get_diagnostics_recording",
                "slice_count": params.slice_count,
            }))
            .await?;

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
        structured_response(
            self.request_attached_or_ws(json!({
                "verb": "get_diagnostics_recording",
                "slice_count": params.slice_count,
            }))
            .await?,
        )
    }
}
