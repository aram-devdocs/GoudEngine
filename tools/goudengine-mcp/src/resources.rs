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
pub const RUST_SDK_KNOWLEDGE_URI: &str = "goudengine://knowledge/sdk-rust";
pub const CSHARP_SDK_KNOWLEDGE_URI: &str = "goudengine://knowledge/sdk-csharp";
pub const PYTHON_SDK_KNOWLEDGE_URI: &str = "goudengine://knowledge/sdk-python";
pub const TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI: &str =
    "goudengine://knowledge/sdk-typescript-desktop";

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
4. Use `goudengine.get_diagnostics` or `goudengine.get_subsystem_diagnostics` to inspect provider diagnostics (render, physics_2d, audio, input, sprite_batch, assets, window).
5. Use `goudengine.get_logs` to retrieve recent engine log entries, optionally filtered by frame number.
6. Use `goudengine.get_scene_hierarchy` to inspect entities with parent/child relationships.
7. Use `goudengine.record_diagnostics` to record diagnostics over time and get time-sliced aggregated results.
8. Use `goudengine.get_diagnostics_recording` to retrieve a previously recorded session as sliced data.
9. Read `goudengine://capture/{id}`, `goudengine://metrics/{id}`, or `goudengine://recording/{id}` resources for stored artifacts.

The bridge stays out of the game process. It only translates MCP requests into local debugger attach requests.
"#;

const RUST_SDK_KNOWLEDGE_TEXT: &str = r#"# Rust SDK Debugger Flow

- Use `DebuggerConfig` as the single debugger enablement model.
- Windowed flows enable debugger mode through `GameConfig` or `EngineConfig::with_debugger(...)`.
- Headless flows enable debugger mode through `ContextConfig` and `Context::create_with_config(...)`.
- Set `publish_local_attach = true` when the route should be discoverable by `goudengine-mcp`.
- Keep Rust as the reference implementation of the shared contract. Do not invent Rust-only debugger semantics.
"#;

const CSHARP_SDK_KNOWLEDGE_TEXT: &str = r#"# C# SDK Debugger Flow

- Use `DebuggerConfig` and `ContextConfig` from the generated C# surface.
- Enable debugger mode before startup through `EngineConfig.SetDebugger(...)` or `new GoudContext(new ContextConfig(...))`.
- Keep the managed layer thin: debugger state, snapshot, replay, capture, and metrics remain Rust-owned.
- Use the same local `goudengine-mcp` attach workflow as every other native desktop SDK.
"#;

const PYTHON_SDK_KNOWLEDGE_TEXT: &str = r#"# Python SDK Debugger Flow

- Use `DebuggerConfig` and `ContextConfig` from the Python package.
- Enable debugger mode before startup with `GoudContext(ContextConfig(...))`.
- Set `publish_local_attach=True` when the route should be discoverable.
- The Python helpers parse raw JSON but do not define a Python-only debugger protocol.
- Use the same local `goudengine-mcp` attach workflow as the other desktop SDKs.
"#;

const TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_TEXT: &str = r#"# TypeScript Desktop Debugger Flow

- This pack applies to the desktop Node/N-API target only.
- Enable debugger mode through the shared config object passed to `GoudContext` or `EngineConfig`.
- Use `publishLocalAttach: true` when the route should be discoverable by `goudengine-mcp`.
- The desktop TypeScript surface stays aligned with the Rust-owned contract.
- Browser/WASM debugger attach is explicitly unsupported in this batch.
"#;

struct StaticResourceDef {
    uri: &'static str,
    name: &'static str,
    description: &'static str,
    text: &'static str,
}

const STATIC_RESOURCE_DEFS: &[StaticResourceDef] = &[
    StaticResourceDef {
        uri: SDK_KNOWLEDGE_URI,
        name: "goudengine-sdk-contract",
        description: "Debugger runtime contract and shared SDK scope notes.",
        text: SDK_KNOWLEDGE_TEXT,
    },
    StaticResourceDef {
        uri: MCP_WORKFLOW_URI,
        name: "goudengine-mcp-workflow",
        description: "Bridge-first workflow for discovery, attach, and artifact reads.",
        text: MCP_WORKFLOW_TEXT,
    },
    StaticResourceDef {
        uri: RUST_SDK_KNOWLEDGE_URI,
        name: "goudengine-sdk-rust",
        description: "Rust SDK debugger enablement and attach guidance.",
        text: RUST_SDK_KNOWLEDGE_TEXT,
    },
    StaticResourceDef {
        uri: CSHARP_SDK_KNOWLEDGE_URI,
        name: "goudengine-sdk-csharp",
        description: "C# SDK debugger enablement and attach guidance.",
        text: CSHARP_SDK_KNOWLEDGE_TEXT,
    },
    StaticResourceDef {
        uri: PYTHON_SDK_KNOWLEDGE_URI,
        name: "goudengine-sdk-python",
        description: "Python SDK debugger enablement and attach guidance.",
        text: PYTHON_SDK_KNOWLEDGE_TEXT,
    },
    StaticResourceDef {
        uri: TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI,
        name: "goudengine-sdk-typescript-desktop",
        description: "TypeScript desktop debugger enablement and browser/WASM scope notes.",
        text: TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_TEXT,
    },
];

pub fn static_resources() -> Vec<Resource> {
    STATIC_RESOURCE_DEFS
        .iter()
        .map(|resource| {
            RawResource::new(resource.uri, resource.name)
                .with_description(resource.description)
                .with_mime_type("text/markdown")
                .with_size(resource.text.len() as u32)
                .no_annotation()
        })
        .collect()
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
    if let Some(resource) = STATIC_RESOURCE_DEFS
        .iter()
        .find(|resource| resource.uri == uri)
    {
        return Ok(vec![
            ResourceContents::text(resource.text, uri).with_mime_type("text/markdown")
        ]);
    }

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
