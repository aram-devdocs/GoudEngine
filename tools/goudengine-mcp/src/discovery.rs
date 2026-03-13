use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use goud_engine::core::debugger::{LocalEndpointV1, RouteSummaryV1, RuntimeManifestV1};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredContext {
    pub pid: u32,
    pub process_nonce: u64,
    pub executable: String,
    pub published_at_unix_ms: u64,
    pub endpoint: LocalEndpointV1,
    pub route: RouteSummaryV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Capture,
    Metrics,
    Recording,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Metrics => "metrics",
            Self::Recording => "recording",
        }
    }
}

pub fn default_runtime_root() -> PathBuf {
    if let Some(override_root) = std::env::var_os("GOUDENGINE_DEBUGGER_RUNTIME_DIR") {
        return PathBuf::from(override_root);
    }

    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("GoudEngine")
            .join("runtime")
    }

    #[cfg(not(windows))]
    {
        if let Some(xdg_runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg_runtime).join("goudengine")
        } else {
            std::env::temp_dir().join("goudengine")
        }
    }
}

pub fn discover_contexts(runtime_root: &Path) -> Vec<DiscoveredContext> {
    let Ok(entries) = fs::read_dir(runtime_root) else {
        return Vec::new();
    };

    let mut contexts = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.starts_with("runtime-") || !file_name.ends_with(".json") {
            continue;
        }

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<RuntimeManifestV1>(&contents) else {
            continue;
        };

        for route in manifest.routes.iter().cloned() {
            contexts.push(DiscoveredContext {
                pid: manifest.pid,
                process_nonce: manifest.process_nonce,
                executable: manifest.executable.clone(),
                published_at_unix_ms: manifest.published_at_unix_ms,
                endpoint: manifest.endpoint.clone(),
                route,
            });
        }
    }

    contexts.sort_by(|left, right| {
        right
            .published_at_unix_ms
            .cmp(&left.published_at_unix_ms)
            .then_with(|| {
                left.route
                    .route_id
                    .context_id
                    .cmp(&right.route.route_id.context_id)
            })
            .then_with(|| left.process_nonce.cmp(&right.process_nonce))
    });
    contexts
}

pub fn find_context(
    runtime_root: &Path,
    context_id: u64,
    process_nonce: Option<u64>,
) -> Result<DiscoveredContext> {
    let matches: Vec<DiscoveredContext> = discover_contexts(runtime_root)
        .into_iter()
        .filter(|context| {
            context.route.route_id.context_id == context_id
                && process_nonce
                    .map(|nonce| context.process_nonce == nonce)
                    .unwrap_or(true)
        })
        .collect();

    match matches.as_slice() {
        [] => bail!("no debugger context found for context_id={context_id}"),
        [single] => Ok(single.clone()),
        _ if process_nonce.is_some() => Ok(matches[0].clone()),
        _ => {
            let candidates: Vec<String> = matches
                .iter()
                .map(|context| {
                    format!(
                        "process_nonce={} pid={} surface_kind={:?}",
                        context.process_nonce, context.pid, context.route.route_id.surface_kind
                    )
                })
                .collect();
            Err(anyhow!(
                "context_id={context_id} is ambiguous; provide processNonce. candidates: {}",
                candidates.join(", ")
            ))
        }
    }
}

pub fn artifacts_root(runtime_root: &Path) -> PathBuf {
    runtime_root.join("artifacts")
}

pub fn artifact_uri(kind: ArtifactKind, artifact_id: &str) -> String {
    format!("goudengine://{}/{artifact_id}", kind.as_str())
}

pub fn artifact_id_from_uri(uri: &str, expected_kind: ArtifactKind) -> Option<String> {
    let prefix = format!("goudengine://{}/", expected_kind.as_str());
    uri.strip_prefix(&prefix).map(ToOwned::to_owned)
}

pub fn capture_artifact_dir(runtime_root: &Path, artifact_id: &str) -> Result<PathBuf> {
    find_artifact_entry(runtime_root, ArtifactKind::Capture, artifact_id)
}

pub fn metrics_artifact_file(runtime_root: &Path, artifact_id: &str) -> Result<PathBuf> {
    find_artifact_entry(runtime_root, ArtifactKind::Metrics, artifact_id)
}

pub fn recording_artifact_dir(runtime_root: &Path, artifact_id: &str) -> Result<PathBuf> {
    find_artifact_entry(runtime_root, ArtifactKind::Recording, artifact_id)
}

pub fn recording_bytes(runtime_root: &Path, artifact_id: &str) -> Result<Vec<u8>> {
    let recording_dir = recording_artifact_dir(runtime_root, artifact_id)?;
    fs::read(recording_dir.join("data.bin"))
        .with_context(|| format!("failed to read recording bytes for artifact {artifact_id}"))
}

fn find_artifact_entry(
    runtime_root: &Path,
    kind: ArtifactKind,
    artifact_id: &str,
) -> Result<PathBuf> {
    let root = artifacts_root(runtime_root);
    let entries = fs::read_dir(&root)
        .with_context(|| format!("failed to read artifacts directory {}", root.display()))?;

    for route_entry in entries.flatten() {
        let candidate = match kind {
            ArtifactKind::Capture | ArtifactKind::Recording => {
                route_entry.path().join(kind.as_str()).join(artifact_id)
            }
            ArtifactKind::Metrics => route_entry
                .path()
                .join(kind.as_str())
                .join(format!("{artifact_id}.json")),
        };
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(anyhow!(
        "artifact {}:{} was not found under {}",
        kind.as_str(),
        artifact_id,
        root.display()
    ))
}
