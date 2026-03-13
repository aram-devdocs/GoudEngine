use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::snapshot::{RouteSummaryV1, RuntimeManifestV1};
use super::super::types::RuntimeRouteId;
use super::super::types::RuntimeSurfaceKind;
use super::capture::CaptureArtifactV1;
use super::replay::ReplayExportEnvelopeV1;
use super::state::{lock_runtime, DebuggerRuntimeState, RuntimeArtifactsState};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const MAX_ARTIFACTS_PER_BUCKET: usize = 8;
const CAPTURE_KIND: &str = "capture";
const METRICS_KIND: &str = "metrics";
const RECORDING_KIND: &str = "recording";
static FALLBACK_ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

fn runtime_root_dir() -> PathBuf {
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

fn short_socket_root(base: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let mut hasher = DefaultHasher::new();
        base.hash(&mut hasher);
        PathBuf::from(format!("/tmp/ge-{:x}", hasher.finish() & 0xffff))
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let mut hasher = DefaultHasher::new();
        base.hash(&mut hasher);
        std::env::temp_dir().join(format!("ge-{:x}", hasher.finish() & 0xffff))
    }

    #[cfg(windows)]
    {
        base.to_path_buf()
    }
}

fn socket_path(root: &Path, process_nonce: u64) -> PathBuf {
    let pid = std::process::id();
    let candidate = root.join(format!("goudengine-{pid}-{process_nonce}.sock"));
    #[cfg(windows)]
    {
        candidate
    }

    #[cfg(not(windows))]
    {
        if candidate.as_os_str().len() <= 96 {
            return candidate;
        }
        let mut hasher = DefaultHasher::new();
        candidate.hash(&mut hasher);
        short_socket_root(root).join(format!(
            "g-{:x}.sock",
            hasher.finish()
        ))
    }
}

fn ensure_runtime_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn atomic_write(path: &Path, contents: &str) -> std::io::Result<()> {
    atomic_write_bytes(path, contents.as_bytes())
}

fn atomic_write_bytes(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path must have a parent directory",
        )
    })?;
    let temp_path = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("manifest"),
        std::process::id()
    ));

    fs::write(&temp_path, contents)?;
    #[cfg(unix)]
    fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))?;

    #[cfg(windows)]
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    fs::rename(&temp_path, path)?;
    Ok(())
}

fn route_bucket_dir(artifacts_root_dir: &Path, context_key: u64) -> PathBuf {
    artifacts_root_dir.join(format!("route-{context_key}"))
}

fn allocate_fallback_artifact_id(context_key: u64, kind: &str) -> String {
    let sequence = FALLBACK_ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{kind}-{context_key}-{sequence:016x}")
}

fn allocate_artifact_id(
    artifacts: &mut RuntimeArtifactsState,
    context_key: u64,
    kind: &str,
) -> String {
    let sequence = artifacts.next_artifact_sequence;
    artifacts.next_artifact_sequence = artifacts.next_artifact_sequence.saturating_add(1);
    format!("{kind}-{context_key}-{sequence:016x}")
}

fn remove_artifact_path(path: PathBuf) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn track_artifact_with_retention(
    artifacts: &mut RuntimeArtifactsState,
    context_key: u64,
    kind: &str,
    artifact_id: &str,
    artifact_path: PathBuf,
) {
    let bucket_key = (context_key, kind.to_string());
    let bucket = artifacts.route_buckets.entry(bucket_key).or_default();
    bucket.push_back(artifact_id.to_string());
    artifacts
        .artifact_paths
        .insert(artifact_id.to_string(), artifact_path);

    while bucket.len() > MAX_ARTIFACTS_PER_BUCKET {
        let Some(oldest_id) = bucket.pop_front() else {
            break;
        };
        if let Some(oldest_path) = artifacts.artifact_paths.remove(&oldest_id) {
            remove_artifact_path(oldest_path);
        }
    }
}

fn store_capture_artifact_in_runtime(
    runtime: &mut DebuggerRuntimeState,
    route_id: &RuntimeRouteId,
    artifact: &CaptureArtifactV1,
) -> String {
    let Some(artifacts) = runtime.artifacts.as_mut() else {
        return allocate_fallback_artifact_id(route_id.context_id, CAPTURE_KIND);
    };

    let artifact_id = allocate_artifact_id(artifacts, route_id.context_id, CAPTURE_KIND);
    let route_dir = route_bucket_dir(&artifacts.artifacts_root_dir, route_id.context_id);
    let capture_dir = route_dir.join(CAPTURE_KIND);
    let artifact_dir = capture_dir.join(&artifact_id);
    if ensure_runtime_dir(&route_dir).is_err()
        || ensure_runtime_dir(&capture_dir).is_err()
        || ensure_runtime_dir(&artifact_dir).is_err()
    {
        return allocate_fallback_artifact_id(route_id.context_id, CAPTURE_KIND);
    }

    if atomic_write_bytes(&artifact_dir.join("image.png"), &artifact.image_png).is_err()
        || atomic_write(&artifact_dir.join("metadata.json"), &artifact.metadata_json).is_err()
        || atomic_write(&artifact_dir.join("snapshot.json"), &artifact.snapshot_json).is_err()
        || atomic_write(
            &artifact_dir.join("metrics_trace.json"),
            &artifact.metrics_trace_json,
        )
        .is_err()
    {
        let _ = fs::remove_dir_all(&artifact_dir);
        return allocate_fallback_artifact_id(route_id.context_id, CAPTURE_KIND);
    }

    track_artifact_with_retention(
        artifacts,
        route_id.context_id,
        CAPTURE_KIND,
        &artifact_id,
        artifact_dir,
    );
    artifact_id
}

fn store_metrics_trace_in_runtime(
    runtime: &mut DebuggerRuntimeState,
    route_id: &RuntimeRouteId,
    metrics_trace_json: &str,
) -> String {
    let Some(artifacts) = runtime.artifacts.as_mut() else {
        return allocate_fallback_artifact_id(route_id.context_id, METRICS_KIND);
    };

    let artifact_id = allocate_artifact_id(artifacts, route_id.context_id, METRICS_KIND);
    let route_dir = route_bucket_dir(&artifacts.artifacts_root_dir, route_id.context_id);
    let metrics_dir = route_dir.join(METRICS_KIND);
    if ensure_runtime_dir(&route_dir).is_err() || ensure_runtime_dir(&metrics_dir).is_err() {
        return allocate_fallback_artifact_id(route_id.context_id, METRICS_KIND);
    }

    let artifact_file = metrics_dir.join(format!("{artifact_id}.json"));
    if atomic_write(&artifact_file, metrics_trace_json).is_err() {
        let _ = fs::remove_file(&artifact_file);
        return allocate_fallback_artifact_id(route_id.context_id, METRICS_KIND);
    }

    track_artifact_with_retention(
        artifacts,
        route_id.context_id,
        METRICS_KIND,
        &artifact_id,
        artifact_file,
    );
    artifact_id
}

pub(super) fn store_recording_artifact_in_state(
    artifacts: Option<&mut RuntimeArtifactsState>,
    route_id: &RuntimeRouteId,
    export: &ReplayExportEnvelopeV1,
) -> String {
    let Some(artifacts) = artifacts else {
        return allocate_fallback_artifact_id(route_id.context_id, RECORDING_KIND);
    };

    let artifact_id = allocate_artifact_id(artifacts, route_id.context_id, RECORDING_KIND);
    let route_dir = route_bucket_dir(&artifacts.artifacts_root_dir, route_id.context_id);
    let recording_dir = route_dir.join(RECORDING_KIND);
    let artifact_dir = recording_dir.join(&artifact_id);
    if ensure_runtime_dir(&route_dir).is_err()
        || ensure_runtime_dir(&recording_dir).is_err()
        || ensure_runtime_dir(&artifact_dir).is_err()
    {
        return allocate_fallback_artifact_id(route_id.context_id, RECORDING_KIND);
    }

    if atomic_write(&artifact_dir.join("manifest.json"), &export.manifest_json).is_err()
        || atomic_write_bytes(&artifact_dir.join("data.bin"), &export.data).is_err()
    {
        let _ = fs::remove_dir_all(&artifact_dir);
        return allocate_fallback_artifact_id(route_id.context_id, RECORDING_KIND);
    }

    track_artifact_with_retention(
        artifacts,
        route_id.context_id,
        RECORDING_KIND,
        &artifact_id,
        artifact_dir,
    );
    artifact_id
}

pub(super) fn store_capture_artifact_for_route(
    route_id: &RuntimeRouteId,
    artifact: &CaptureArtifactV1,
) -> String {
    let mut guard = lock_runtime();
    let Some(runtime) = guard.as_mut() else {
        return allocate_fallback_artifact_id(route_id.context_id, CAPTURE_KIND);
    };
    store_capture_artifact_in_runtime(runtime, route_id, artifact)
}

pub(super) fn store_metrics_trace_for_route(
    route_id: &RuntimeRouteId,
    metrics_trace_json: &str,
) -> String {
    let mut guard = lock_runtime();
    let Some(runtime) = guard.as_mut() else {
        return allocate_fallback_artifact_id(route_id.context_id, METRICS_KIND);
    };
    store_metrics_trace_in_runtime(runtime, route_id, metrics_trace_json)
}

fn endpoint_for_runtime(
    root: &Path,
    process_nonce: u64,
) -> super::super::snapshot::LocalEndpointV1 {
    #[cfg(windows)]
    {
        super::super::snapshot::LocalEndpointV1 {
            transport: "named_pipe".to_string(),
            location: format!(
                r"\\.\pipe\goudengine-{}-{}",
                std::process::id(),
                process_nonce
            ),
        }
    }

    #[cfg(not(windows))]
    {
        super::super::snapshot::LocalEndpointV1 {
            transport: "unix".to_string(),
            location: socket_path(root, process_nonce).display().to_string(),
        }
    }
}

pub(super) fn current_manifest(runtime: &DebuggerRuntimeState) -> Option<RuntimeManifestV1> {
    if runtime.attached_route_count() == 0 {
        return None;
    }

    let artifacts = runtime.artifacts.as_ref()?;
    let executable = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut routes: Vec<RouteSummaryV1> = runtime
        .routes
        .values()
        .map(|route| RouteSummaryV1 {
            route_id: route.snapshot.route_id.clone(),
            label: route.label.clone(),
            attachable: route.attachable,
            capabilities: route.capabilities.clone(),
        })
        .collect();
    routes.sort_by_key(|route| route.route_id.context_id);

    Some(RuntimeManifestV1 {
        manifest_version: 1,
        pid: std::process::id(),
        process_nonce: runtime.process_nonce,
        executable,
        endpoint: artifacts.endpoint.clone(),
        routes,
        published_at_unix_ms: runtime.published_at_unix_ms,
    })
}

pub(super) fn sync_manifest(runtime: &mut DebuggerRuntimeState) {
    if runtime.attached_route_count() == 0 {
        cleanup(runtime);
        return;
    }

    let runtime_dir = runtime_root_dir();
    if ensure_runtime_dir(&runtime_dir).is_err() {
        return;
    }

    let endpoint = endpoint_for_runtime(&runtime_dir, runtime.process_nonce);
    #[cfg(not(windows))]
    if endpoint.transport == "unix" {
        let socket_parent = Path::new(&endpoint.location)
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| runtime_dir.clone());
        if ensure_runtime_dir(&socket_parent).is_err() {
            return;
        }
    }

    let manifest_path = runtime_dir.join(format!(
        "runtime-{}-{}.json",
        std::process::id(),
        runtime.process_nonce
    ));
    let artifacts_root_dir = runtime_dir.join("artifacts");
    if ensure_runtime_dir(&artifacts_root_dir).is_err() {
        return;
    }
    if let Some(artifacts) = runtime.artifacts.as_mut() {
        if artifacts.runtime_dir != runtime_dir {
            artifacts.artifact_paths.clear();
            artifacts.route_buckets.clear();
        }
        artifacts.runtime_dir = runtime_dir.clone();
        artifacts.artifacts_root_dir = artifacts_root_dir.clone();
        artifacts.manifest_path = manifest_path.clone();
        artifacts.endpoint = endpoint.clone();
    } else {
        runtime.artifacts = Some(RuntimeArtifactsState {
            runtime_dir: runtime_dir.clone(),
            artifacts_root_dir: artifacts_root_dir.clone(),
            manifest_path: manifest_path.clone(),
            endpoint: endpoint.clone(),
            next_artifact_sequence: 1,
            artifact_paths: Default::default(),
            route_buckets: Default::default(),
        });
    }

    if let Some(manifest) = current_manifest(runtime) {
        if let Ok(json) = manifest.to_json() {
            let _ = atomic_write(&manifest_path, &json);
        }
    }
}

pub(super) fn cleanup(runtime: &mut DebuggerRuntimeState) {
    let Some(artifacts) = runtime.artifacts.take() else {
        return;
    };

    let _ = fs::remove_file(&artifacts.manifest_path);
    if artifacts.endpoint.transport == "unix" {
        let _ = fs::remove_file(&artifacts.endpoint.location);
    }
    let _ = fs::remove_dir_all(&artifacts.artifacts_root_dir);
    let _ = fs::remove_dir(&artifacts.runtime_dir);
}

#[cfg(test)]
pub(super) fn manifest_path(runtime: &DebuggerRuntimeState) -> Option<PathBuf> {
    runtime
        .artifacts
        .as_ref()
        .map(|artifacts| artifacts.manifest_path.clone())
}

#[allow(dead_code)]
fn _surface_kind_marker(kind: RuntimeSurfaceKind) -> RuntimeSurfaceKind {
    kind
}
