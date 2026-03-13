use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use rmcp::model::{
    AnnotateAble, RawResource, RawResourceTemplate, Resource, ResourceContents, ResourceTemplate,
};

use crate::discovery::{
    artifact_id_from_uri, artifact_uri, capture_artifact_dir, metrics_artifact_file,
    recording_artifact_dir, ArtifactKind,
};

pub const SDK_KNOWLEDGE_URI: &str = "goudengine://knowledge/sdk-contract";
pub const MCP_WORKFLOW_URI: &str = "goudengine://knowledge/mcp-workflow";

const SDK_KNOWLEDGE_TEXT: &str = r#"# GoudEngine Debugger Contract

The debugger runtime is Rust-owned and local-only in this batch.

- Enable debugger mode before startup through `GoudGame` or `GoudContext` config.
- Desktop/native SDKs expose control, capture, replay, and metrics through thin wrappers.
- TypeScript web is explicitly unsupported for the debugger runtime in this batch.
- MCP discovery reads process manifests and then attaches to one route-scoped local IPC session.
"#;

const MCP_WORKFLOW_TEXT: &str = r#"# GoudEngine MCP Workflow

Typical flow:

1. Call `goudengine.list_contexts`.
2. Call `goudengine.attach_context` for the target route.
3. Use snapshot, control, capture, replay, and metrics tools against the attached route.
4. Read `goudengine://capture/{id}`, `goudengine://metrics/{id}`, or `goudengine://recording/{id}` resources for stored artifacts.

The bridge stays out of the game process. It only translates MCP requests into local debugger attach requests.
"#;

pub fn static_resources() -> Vec<Resource> {
    vec![
        RawResource::new(SDK_KNOWLEDGE_URI, "goudengine-sdk-contract")
            .with_description("Debugger runtime contract and SDK scope notes.")
            .with_mime_type("text/markdown")
            .with_size(SDK_KNOWLEDGE_TEXT.len() as u32)
            .no_annotation(),
        RawResource::new(MCP_WORKFLOW_URI, "goudengine-mcp-workflow")
            .with_description("Bridge-first workflow for discovery, attach, and artifact reads.")
            .with_mime_type("text/markdown")
            .with_size(MCP_WORKFLOW_TEXT.len() as u32)
            .no_annotation(),
    ]
}

pub fn resource_templates() -> Vec<ResourceTemplate> {
    vec![
        RawResourceTemplate::new("goudengine://capture/{id}", "goudengine-capture")
            .with_description("Read a stored debugger frame capture artifact.")
            .with_mime_type("application/json")
            .no_annotation(),
        RawResourceTemplate::new("goudengine://metrics/{id}", "goudengine-metrics")
            .with_description("Read a stored debugger metrics trace export.")
            .with_mime_type("application/json")
            .no_annotation(),
        RawResourceTemplate::new("goudengine://recording/{id}", "goudengine-recording")
            .with_description("Read a stored debugger replay artifact.")
            .with_mime_type("application/octet-stream")
            .no_annotation(),
    ]
}

pub fn add_resource_uri(kind: ArtifactKind, value: &mut serde_json::Value) {
    let Some(artifact_id) = value
        .get("artifact_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "resource_uri".to_string(),
            serde_json::Value::String(artifact_uri(kind, &artifact_id)),
        );
    }
}

pub fn read_resource(uri: &str, runtime_root: &Path) -> Result<Vec<ResourceContents>> {
    match uri {
        SDK_KNOWLEDGE_URI => Ok(vec![
            ResourceContents::text(SDK_KNOWLEDGE_TEXT, uri).with_mime_type("text/markdown")
        ]),
        MCP_WORKFLOW_URI => Ok(vec![
            ResourceContents::text(MCP_WORKFLOW_TEXT, uri).with_mime_type("text/markdown")
        ]),
        _ => {
            if let Some(artifact_id) = artifact_id_from_uri(uri, ArtifactKind::Capture) {
                return read_capture_resource(uri, runtime_root, &artifact_id);
            }
            if let Some(artifact_id) = artifact_id_from_uri(uri, ArtifactKind::Metrics) {
                return read_metrics_resource(uri, runtime_root, &artifact_id);
            }
            if let Some(artifact_id) = artifact_id_from_uri(uri, ArtifactKind::Recording) {
                return read_recording_resource(uri, runtime_root, &artifact_id);
            }
            anyhow::bail!("unknown resource uri: {uri}")
        }
    }
}

fn read_capture_resource(
    uri: &str,
    runtime_root: &Path,
    artifact_id: &str,
) -> Result<Vec<ResourceContents>> {
    let artifact_dir = capture_artifact_dir(runtime_root, artifact_id)?;
    let image_bytes = fs::read(artifact_dir.join("image.png"))
        .with_context(|| format!("failed to read capture image for artifact {artifact_id}"))?;
    let metadata_json = fs::read_to_string(artifact_dir.join("metadata.json"))
        .with_context(|| format!("failed to read capture metadata for artifact {artifact_id}"))?;
    let snapshot_json = fs::read_to_string(artifact_dir.join("snapshot.json"))
        .with_context(|| format!("failed to read capture snapshot for artifact {artifact_id}"))?;
    let metrics_json = fs::read_to_string(artifact_dir.join("metrics_trace.json"))
        .with_context(|| format!("failed to read capture metrics for artifact {artifact_id}"))?;

    Ok(vec![
        ResourceContents::blob(BASE64_STANDARD.encode(image_bytes), format!("{uri}#image"))
            .with_mime_type("image/png"),
        ResourceContents::text(metadata_json, format!("{uri}#metadata"))
            .with_mime_type("application/json"),
        ResourceContents::text(snapshot_json, format!("{uri}#snapshot"))
            .with_mime_type("application/json"),
        ResourceContents::text(metrics_json, format!("{uri}#metrics"))
            .with_mime_type("application/json"),
    ])
}

fn read_metrics_resource(
    uri: &str,
    runtime_root: &Path,
    artifact_id: &str,
) -> Result<Vec<ResourceContents>> {
    let metrics_file = metrics_artifact_file(runtime_root, artifact_id)?;
    let metrics_json = fs::read_to_string(&metrics_file)
        .with_context(|| format!("failed to read metrics artifact {}", metrics_file.display()))?;
    Ok(vec![
        ResourceContents::text(metrics_json, uri).with_mime_type("application/json")
    ])
}

fn read_recording_resource(
    uri: &str,
    runtime_root: &Path,
    artifact_id: &str,
) -> Result<Vec<ResourceContents>> {
    let artifact_dir = recording_artifact_dir(runtime_root, artifact_id)?;
    let manifest_json = fs::read_to_string(artifact_dir.join("manifest.json"))
        .with_context(|| format!("failed to read recording manifest for artifact {artifact_id}"))?;
    let data = fs::read(artifact_dir.join("data.bin"))
        .with_context(|| format!("failed to read recording bytes for artifact {artifact_id}"))?;

    Ok(vec![
        ResourceContents::text(manifest_json, format!("{uri}#manifest"))
            .with_mime_type("application/json"),
        ResourceContents::blob(BASE64_STANDARD.encode(data), format!("{uri}#data"))
            .with_mime_type("application/octet-stream"),
    ])
}
