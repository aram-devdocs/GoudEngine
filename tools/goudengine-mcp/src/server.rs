use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourceTemplatesResult,
    ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
    ServerCapabilities, ServerInfo,
};
use rmcp::ErrorData as McpError;
use rmcp::{tool_handler, ServerHandler};
use serde_json::{json, Value};

use crate::attach_client::{AttachError, AttachedRoute};
use crate::discovery::{self, ArtifactKind, DiscoveredContext};
use crate::prompts;
use crate::resources;
use crate::ws_relay::WsRelayState;

mod tools;
mod types;

use self::types::StartReplayParams;

struct BridgeState {
    runtime_root: std::path::PathBuf,
    attached: Option<AttachedRoute>,
    /// When attached to a WebSocket browser route, holds the relay route id.
    ws_route_id: Option<String>,
}

#[derive(Clone)]
pub struct GoudEngineMcpServer {
    state: Arc<Mutex<BridgeState>>,
    tool_router: ToolRouter<Self>,
    ws_relay: WsRelayState,
}

impl Default for GoudEngineMcpServer {
    fn default() -> Self {
        Self::new()
    }
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
                ws_route_id: None,
            })),
            tool_router: Self::create_tool_router(),
            ws_relay: WsRelayState::new(),
        }
    }

    /// Returns a reference to the WebSocket relay state for spawning the relay.
    pub fn ws_relay(&self) -> &WsRelayState {
        &self.ws_relay
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

    /// Sends an IPC request through the WebSocket relay if a browser route is
    /// attached, otherwise falls back to the local Unix socket client.
    async fn request_attached_or_ws(&self, request: Value) -> Result<Value, McpError> {
        let ws_route_id = {
            let state = self.state()?;
            state.ws_route_id.clone()
        };
        if let Some(route_id) = ws_route_id {
            return self
                .ws_relay
                .request(&route_id, request)
                .await
                .map_err(|e| McpError::internal_error(e, None));
        }
        self.request_attached(request)
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
