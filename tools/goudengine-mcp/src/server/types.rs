use std::collections::BTreeMap;

use anyhow::Result;
use rmcp::handler::server::wrapper::Json;
use rmcp::schemars::JsonSchema;
use rmcp::ErrorData as McpError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct AttachContextParams {
    pub(super) context_id: u64,
    pub(super) process_nonce: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum McpDebuggerStepKind {
    Frame,
    Tick,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct StepParams {
    pub(super) kind: McpDebuggerStepKind,
    pub(super) count: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetPausedParams {
    pub(super) paused: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetTimeScaleParams {
    pub(super) scale: f32,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct InspectEntityParams {
    pub(super) entity_id: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct InputEventParams {
    pub(super) device: String,
    pub(super) action: String,
    pub(super) key: Option<String>,
    pub(super) button: Option<String>,
    pub(super) position: Option<[f32; 2]>,
    pub(super) delta: Option<[f32; 2]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct InjectInputParams {
    pub(super) events: Vec<InputEventParams>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetSubsystemDiagnosticsParams {
    pub(super) key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetLogsParams {
    pub(super) since_frame: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct StartReplayParams {
    pub(super) artifact_id: Option<String>,
    pub(super) resource_uri: Option<String>,
    pub(super) data_base64: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(super) struct BridgeResponse {
    #[serde(flatten)]
    data: BTreeMap<String, Value>,
}

impl BridgeResponse {
    pub(super) fn from_value(value: Value) -> Result<Self, McpError> {
        let Value::Object(object) = value else {
            return Err(McpError::internal_error(
                "bridge tools require object-shaped debugger responses",
                None,
            ));
        };
        Ok(Self {
            data: object.into_iter().collect(),
        })
    }

    #[cfg(test)]
    pub(super) fn into_value(self) -> Value {
        Value::Object(self.data.into_iter().collect())
    }
}

pub(super) fn structured_response(value: Value) -> Result<Json<BridgeResponse>, McpError> {
    Ok(Json(BridgeResponse::from_value(value)?))
}
