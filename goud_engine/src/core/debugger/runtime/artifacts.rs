use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use super::super::snapshot::{RouteSummaryV1, RuntimeManifestV1};
use super::super::types::RuntimeSurfaceKind;
use super::state::{DebuggerRuntimeState, RuntimeArtifactsState};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    #[cfg(not(windows))]
    {
        base.join("s")
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("goudengine").join("s"))
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
            "g-{pid}-{process_nonce}-{:x}.sock",
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
    let parent = path.parent().expect("manifest path should have a parent");
    let temp_path = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("manifest"),
        std::process::id()
    ));

    fs::write(&temp_path, contents.as_bytes())?;
    #[cfg(unix)]
    fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))?;

    #[cfg(windows)]
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    fs::rename(&temp_path, path)?;
    Ok(())
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
    runtime.artifacts = Some(RuntimeArtifactsState {
        runtime_dir: runtime_dir.clone(),
        manifest_path: manifest_path.clone(),
        endpoint,
    });

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
