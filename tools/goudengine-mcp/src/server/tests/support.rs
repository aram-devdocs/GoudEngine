use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use goud_engine::core::debugger::{
    AttachAcceptedV1, CapabilityStateV1, LocalEndpointV1, RouteSummaryV1, RuntimeManifestV1,
    RuntimeRouteId, RuntimeSurfaceKind,
};
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions, Stream};
use rmcp::handler::server::wrapper::{Json, Parameters};
use serde_json::{json, Value};
use tempfile::TempDir;

use crate::server::types::{
    AttachContextParams, GetDiagnosticsRecordingParams, GetLogsParams,
    GetSubsystemDiagnosticsParams, InjectInputParams, InputEventParams, RecordDiagnosticsParams,
    SetPausedParams, SetTimeScaleParams, StartReplayParams, StepParams,
};
use crate::server::{GoudEngineMcpServer, McpDebuggerStepKind};

static TEST_SOCKET_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct TestHarness {
    pub runtime_root: TempDir,
    pub route: RuntimeRouteId,
    endpoint_location: String,
    requests: Arc<Mutex<Vec<Value>>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    server_thread: Option<thread::JoinHandle<()>>,
}

impl TestHarness {
    pub fn new() -> Self {
        Self::with_server(true)
    }

    pub fn manifest_only() -> Self {
        Self::with_server(false)
    }

    fn with_server(start_server: bool) -> Self {
        assert!(
            GenericFilePath::is_supported(),
            "local socket transport is unsupported in this test environment"
        );

        let runtime_root = TempDir::new().expect("temp runtime root should be created");
        let route = RuntimeRouteId {
            process_nonce: 77,
            context_id: 54,
            surface_kind: RuntimeSurfaceKind::HeadlessContext,
        };
        let endpoint = test_endpoint(runtime_root.path());
        write_manifest(runtime_root.path(), &route, &endpoint);
        write_artifacts(runtime_root.path(), route.context_id);

        let requests = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let server_thread = start_server.then(|| {
            spawn_fake_attach_server(
                endpoint.location.clone(),
                route.clone(),
                requests.clone(),
                shutdown.clone(),
            )
        });

        Self {
            runtime_root,
            route,
            endpoint_location: endpoint.location,
            requests,
            shutdown,
            server_thread,
        }
    }

    pub fn server(&self) -> GoudEngineMcpServer {
        GoudEngineMcpServer::with_runtime_root(self.runtime_root.path())
    }

    pub fn requests(&self) -> Vec<Value> {
        self.requests.lock().expect("request log lock").clone()
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = connect_test_stream(&self.endpoint_location);
        if let Some(handle) = self.server_thread.take() {
            if std::thread::panicking() {
                let _ = handle.join();
            } else {
                handle
                    .join()
                    .expect("fake attach server should shut down cleanly");
            }
        }
    }
}

pub async fn list_contexts(server: &GoudEngineMcpServer) -> Value {
    let Json(listed) = server.list_contexts().await.expect("contexts should list");
    listed.into_value()
}

pub async fn attach_context(
    server: &GoudEngineMcpServer,
    context_id: u64,
    process_nonce: u64,
) -> Value {
    let Json(attached) = server
        .attach_context(Parameters(AttachContextParams {
            context_id,
            process_nonce: Some(process_nonce),
            ws_route_id: None,
        }))
        .await
        .expect("context should attach");
    attached.into_value()
}

pub async fn snapshot(server: &GoudEngineMcpServer) -> Value {
    let Json(snapshot) = server.get_snapshot().await.expect("snapshot should load");
    snapshot.into_value()
}

pub async fn set_paused(server: &GoudEngineMcpServer, paused: bool) -> Value {
    let Json(response) = server
        .set_paused(Parameters(SetPausedParams { paused }))
        .await
        .expect("pause request should succeed");
    response.into_value()
}

pub async fn step_ticks(server: &GoudEngineMcpServer, count: u32) -> Value {
    let Json(response) = server
        .step(Parameters(StepParams {
            kind: McpDebuggerStepKind::Tick,
            count,
        }))
        .await
        .expect("step request should succeed");
    response.into_value()
}

pub async fn set_time_scale(server: &GoudEngineMcpServer, scale: f32) -> Value {
    let Json(response) = server
        .set_time_scale(Parameters(SetTimeScaleParams { scale }))
        .await
        .expect("time-scale request should succeed");
    response.into_value()
}

pub async fn inject_mouse_move(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .inject_input(Parameters(InjectInputParams {
            events: vec![InputEventParams {
                device: "mouse".to_string(),
                action: "move".to_string(),
                key: None,
                button: None,
                position: Some([10.0, 20.0]),
                delta: Some([1.0, -1.0]),
            }],
        }))
        .await
        .expect("input injection should succeed");
    response.into_value()
}

pub async fn metrics_trace(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .get_metrics_trace()
        .await
        .expect("metrics export should succeed");
    response.into_value()
}

pub async fn capture_frame(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .capture_frame()
        .await
        .expect("capture should succeed");
    response.into_value()
}

pub async fn stop_recording(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .stop_recording()
        .await
        .expect("recording export should succeed");
    response.into_value()
}

pub async fn get_diagnostics(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .get_diagnostics()
        .await
        .expect("diagnostics should load");
    response.into_value()
}

pub async fn get_subsystem_diagnostics(server: &GoudEngineMcpServer, key: &str) -> Value {
    let Json(response) = server
        .get_subsystem_diagnostics(Parameters(GetSubsystemDiagnosticsParams {
            key: key.to_string(),
        }))
        .await
        .expect("subsystem diagnostics should load");
    response.into_value()
}

pub async fn get_logs(server: &GoudEngineMcpServer, since_frame: Option<u64>) -> Value {
    let Json(response) = server
        .get_logs(Parameters(GetLogsParams { since_frame }))
        .await
        .expect("logs should load");
    response.into_value()
}

pub async fn get_scene_hierarchy(server: &GoudEngineMcpServer) -> Value {
    let Json(response) = server
        .get_scene_hierarchy()
        .await
        .expect("scene hierarchy should load");
    response.into_value()
}

pub async fn get_diagnostics_recording(server: &GoudEngineMcpServer, slice_count: u32) -> Value {
    let Json(response) = server
        .get_diagnostics_recording(Parameters(GetDiagnosticsRecordingParams { slice_count }))
        .await
        .expect("diagnostics recording should load");
    response.into_value()
}

pub async fn start_replay(server: &GoudEngineMcpServer, artifact_id: String) -> Value {
    let Json(response) = server
        .start_replay(Parameters(StartReplayParams {
            artifact_id: Some(artifact_id),
            resource_uri: None,
            data_base64: None,
        }))
        .await
        .expect("replay should start");
    response.into_value()
}

fn spawn_fake_attach_server(
    endpoint_location: String,
    route: RuntimeRouteId,
    requests: Arc<Mutex<Vec<Value>>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let listener = ListenerOptions::new()
            .name(local_socket_name(&endpoint_location).expect("listener name"))
            .create_sync()
            .expect("listener should bind");
        let mut stream = listener.accept().expect("client should connect");

        let hello = match read_frame(&mut stream) {
            Ok(hello) => hello,
            Err(err)
                if err.kind() == ErrorKind::UnexpectedEof && shutdown.load(Ordering::Relaxed) =>
            {
                return;
            }
            Err(err) => panic!("hello frame should read: {err}"),
        };
        assert_eq!(hello["protocol_version"], 1);
        assert_eq!(hello["route_id"]["context_id"], route.context_id);

        write_frame(
            &mut stream,
            &json!(AttachAcceptedV1 {
                protocol_version: 1,
                session_id: 88,
                route_id: route.clone(),
                snapshot_schema: "debugger_snapshot_v1".to_string(),
                heartbeat_interval_ms: 1_000,
            }),
        )
        .expect("attach acceptance should write");

        while requests.lock().expect("request log").len() < 20 {
            let request = match read_frame(&mut stream) {
                Ok(request) => request,
                Err(err) if err.kind() == ErrorKind::UnexpectedEof => break,
                Err(err) => panic!("request frame should read: {err}"),
            };
            requests.lock().expect("request log").push(request.clone());
            let verb = request["verb"].as_str().expect("verb");
            let result = match verb {
                "get_snapshot" => json!({
                    "frame": { "index": 42 },
                    "selection": { "entity_id": null },
                    "entities": [
                        { "entity_id": 7, "name": "Player" }
                    ],
                }),
                "set_paused" => json!({ "paused": request["paused"] }),
                "step" => json!({ "accepted": true }),
                "set_time_scale" => json!({ "time_scale": request["time_scale"] }),
                "inject_input" => json!({ "accepted": true }),
                "get_metrics_trace" => json!({ "artifact_id": "metrics-54-0000000000000003" }),
                "capture_frame" => json!({ "artifact_id": "capture-54-0000000000000001" }),
                "stop_recording" => json!({ "artifact_id": "recording-54-0000000000000002" }),
                "start_replay" => json!({ "status": "replaying" }),
                "get_diagnostics" => json!({
                    "diagnostics": {
                        "render": { "draw_calls": 42 },
                        "audio": { "active_sources": 3 },
                    }
                }),
                "get_diagnostics_for" => {
                    let key = request["key"].as_str().unwrap_or("unknown");
                    json!({ "key": key, "diagnostics": { "draw_calls": 42 } })
                }
                "get_logs" => json!({
                    "logs": [
                        { "timestamp_ms": 1000, "level": "INFO", "message": "tick", "target": "engine" }
                    ]
                }),
                "get_scene_hierarchy" => json!({
                    "entities": [
                        { "entity_id": 1, "name": "Root", "parent_entity_id": null, "child_entity_ids": [2, 3] },
                        { "entity_id": 2, "name": "Child1", "parent_entity_id": 1, "child_entity_ids": [] },
                        { "entity_id": 3, "name": "Child2", "parent_entity_id": 1, "child_entity_ids": [] },
                    ]
                }),
                "start_diagnostics_recording" => json!({
                    "recording_id": "diag-rec-54-100",
                    "status": { "active": true, "frame_count": 0, "recording_id": "diag-rec-54-100" }
                }),
                "stop_diagnostics_recording" => json!({
                    "recording_id": "diag-rec-54-100",
                    "frame_count": 60
                }),
                "get_diagnostics_recording" => {
                    let slice_count = request["slice_count"].as_u64().unwrap_or(10);
                    json!({
                        "version": 1,
                        "recording_id": "diag-rec-54-100",
                        "total_frames": 60,
                        "total_duration_seconds": 1.0,
                        "requested_slices": slice_count,
                        "slices": (0..slice_count).map(|i| json!({
                            "slice_index": i,
                            "frame_range": [i * 6, (i + 1) * 6 - 1],
                            "time_range": [i as f64 * 0.1, (i + 1) as f64 * 0.1],
                            "frame_count": 6,
                            "avg_delta_seconds": 0.0166,
                            "avg_fps": 60.0,
                        })).collect::<Vec<_>>()
                    })
                }
                other => panic!("unexpected debugger verb: {other}"),
            };
            write_frame(&mut stream, &json!({ "ok": true, "result": result }))
                .expect("response frame should write");
        }
    })
}

fn write_manifest(runtime_root: &Path, route: &RuntimeRouteId, endpoint: &LocalEndpointV1) {
    let manifest = RuntimeManifestV1 {
        manifest_version: 1,
        pid: std::process::id(),
        process_nonce: route.process_nonce,
        executable: "mcp-test".to_string(),
        endpoint: endpoint.clone(),
        routes: vec![RouteSummaryV1 {
            route_id: route.clone(),
            label: Some("test".to_string()),
            attachable: true,
            capabilities: BTreeMap::from([
                ("snapshots".to_string(), CapabilityStateV1::Ready),
                ("control_plane".to_string(), CapabilityStateV1::Ready),
                ("capture".to_string(), CapabilityStateV1::Ready),
                ("replay".to_string(), CapabilityStateV1::Ready),
            ]),
        }],
        published_at_unix_ms: 12_345,
    };
    fs::write(
        runtime_root.join(format!("runtime-{}.json", route.process_nonce)),
        serde_json::to_string(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should write");
}

fn write_artifacts(runtime_root: &Path, context_id: u64) {
    let route_dir = runtime_root
        .join("artifacts")
        .join(format!("route-{context_id}"));
    let capture_dir = route_dir.join("capture/capture-54-0000000000000001");
    let recording_dir = route_dir.join("recording/recording-54-0000000000000002");
    let metrics_dir = route_dir.join("metrics");

    fs::create_dir_all(&capture_dir).expect("capture dir should exist");
    fs::create_dir_all(&recording_dir).expect("recording dir should exist");
    fs::create_dir_all(&metrics_dir).expect("metrics dir should exist");

    fs::write(capture_dir.join("image.png"), [0_u8, 1, 2, 3]).expect("image should write");
    fs::write(capture_dir.join("metadata.json"), "{\"capture\":true}")
        .expect("metadata should write");
    fs::write(capture_dir.join("snapshot.json"), "{\"snapshot\":true}")
        .expect("snapshot should write");
    fs::write(capture_dir.join("metrics_trace.json"), "{\"metrics\":true}")
        .expect("capture metrics should write");
    fs::write(recording_dir.join("manifest.json"), "{\"recording\":true}")
        .expect("recording manifest should write");
    fs::write(recording_dir.join("data.bin"), [1_u8, 2, 3, 4]).expect("recording should write");
    fs::write(
        metrics_dir.join("metrics-54-0000000000000003.json"),
        "{\"frames\":[]}",
    )
    .expect("metrics artifact should write");
}

#[cfg(not(windows))]
fn test_endpoint(runtime_root: &Path) -> LocalEndpointV1 {
    let sequence = TEST_SOCKET_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = runtime_root.join(format!("mcp-attach-{sequence}.sock"));
    LocalEndpointV1 {
        transport: "unix".to_string(),
        location: path.to_string_lossy().into_owned(),
    }
}

#[cfg(windows)]
fn test_endpoint(_runtime_root: &Path) -> LocalEndpointV1 {
    let sequence = TEST_SOCKET_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let pipe_name = format!(r"\\.\pipe\goudengine-mcp-test-{sequence}");
    LocalEndpointV1 {
        transport: "named_pipe".to_string(),
        location: pipe_name,
    }
}

#[cfg(not(windows))]
fn local_socket_name(endpoint_location: &str) -> io::Result<interprocess::local_socket::Name<'_>> {
    Path::new(endpoint_location).to_fs_name::<GenericFilePath>()
}

#[cfg(windows)]
fn local_socket_name(endpoint_location: &str) -> io::Result<interprocess::local_socket::Name<'_>> {
    let pipe_name = endpoint_location
        .strip_prefix(r"\\.\pipe\")
        .unwrap_or(endpoint_location);
    pipe_name.to_ns_name::<GenericNamespaced>()
}

fn connect_test_stream(endpoint_location: &str) -> io::Result<Stream> {
    Stream::connect(local_socket_name(endpoint_location)?)
}

fn write_frame(stream: &mut Stream, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_vec(value).expect("frame should serialize");
    let len = u32::try_from(payload.len()).expect("frame length should fit");
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    Ok(())
}

fn read_frame(stream: &mut Stream) -> io::Result<Value> {
    let mut len_buf = [0_u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0_u8; len];
    stream.read_exact(&mut payload)?;
    serde_json::from_slice(&payload).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
}
