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

use super::{
    types::{
        AttachContextParams, InjectInputParams, InputEventParams, SetPausedParams,
        SetTimeScaleParams, StartReplayParams, StepParams,
    },
    GoudEngineMcpServer,
};
use crate::discovery;
use crate::resources;

static TEST_SOCKET_SEQUENCE: AtomicU64 = AtomicU64::new(1);

struct TestHarness {
    runtime_root: TempDir,
    route: RuntimeRouteId,
    endpoint_location: String,
    requests: Arc<Mutex<Vec<Value>>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    server_thread: Option<thread::JoinHandle<()>>,
}

impl TestHarness {
    fn new() -> Self {
        Self::with_server(true)
    }

    fn manifest_only() -> Self {
        Self::with_server(false)
    }

    fn with_server(start_server: bool) -> Self {
        if !GenericFilePath::is_supported() {
            panic!("local socket transport is unsupported in this test environment");
        }

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

    fn server(&self) -> GoudEngineMcpServer {
        GoudEngineMcpServer::with_runtime_root(self.runtime_root.path())
    }

    fn requests(&self) -> Vec<Value> {
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

#[tokio::test]
async fn list_attach_control_capture_replay_and_resources_work() {
    let harness = TestHarness::new();
    let server = harness.server();

    let Json(listed) = server.list_contexts().await.expect("contexts should list");
    let listed = listed.into_value();
    let contexts = listed["contexts"]
        .as_array()
        .expect("contexts should be an array");
    assert_eq!(contexts.len(), 1);
    assert_eq!(
        contexts[0]["route"]["route_id"]["context_id"],
        harness.route.context_id
    );
    assert!(listed["attached_context"].is_null());

    let Json(attached) = server
        .attach_context(Parameters(AttachContextParams {
            context_id: harness.route.context_id,
            process_nonce: Some(harness.route.process_nonce),
        }))
        .await
        .expect("context should attach");
    let attached = attached.into_value();
    assert_eq!(
        attached["context"]["route"]["route_id"]["context_id"],
        harness.route.context_id
    );
    assert_eq!(
        attached["session"]["snapshot_schema"],
        "debugger_snapshot_v1"
    );

    let Json(snapshot) = server.get_snapshot().await.expect("snapshot should load");
    let snapshot = snapshot.into_value();
    assert_eq!(snapshot["frame"]["index"], 42);

    let Json(paused) = server
        .set_paused(Parameters(SetPausedParams { paused: true }))
        .await
        .expect("pause request should succeed");
    let paused = paused.into_value();
    assert_eq!(paused["paused"], true);

    let Json(step) = server
        .step(Parameters(StepParams {
            kind: super::McpDebuggerStepKind::Tick,
            count: 3,
        }))
        .await
        .expect("step request should succeed");
    let step = step.into_value();
    assert_eq!(step["accepted"], true);

    let Json(time_scale) = server
        .set_time_scale(Parameters(SetTimeScaleParams { scale: 0.5 }))
        .await
        .expect("time-scale request should succeed");
    let time_scale = time_scale.into_value();
    assert_eq!(time_scale["time_scale"], 0.5);

    let Json(input) = server
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
    let input = input.into_value();
    assert_eq!(input["accepted"], true);

    let Json(metrics) = server
        .get_metrics_trace()
        .await
        .expect("metrics export should succeed");
    let metrics = metrics.into_value();
    assert_eq!(metrics["artifact_id"], "metrics-54-0000000000000003");
    assert_eq!(
        metrics["resource_uri"],
        "goudengine://metrics/metrics-54-0000000000000003"
    );

    let Json(capture) = server
        .capture_frame()
        .await
        .expect("capture should succeed");
    let capture = capture.into_value();
    assert_eq!(capture["artifact_id"], "capture-54-0000000000000001");
    assert_eq!(
        capture["resource_uri"],
        "goudengine://capture/capture-54-0000000000000001"
    );
    let capture_contents = resources::read_resource(
        capture["resource_uri"].as_str().expect("capture uri"),
        harness.runtime_root.path(),
    )
    .expect("capture resource should load");
    assert_eq!(capture_contents.len(), 4);

    let Json(recording) = server
        .stop_recording()
        .await
        .expect("recording export should succeed");
    let recording = recording.into_value();
    assert_eq!(recording["artifact_id"], "recording-54-0000000000000002");
    assert_eq!(
        recording["resource_uri"],
        "goudengine://recording/recording-54-0000000000000002"
    );
    let recording_contents = resources::read_resource(
        recording["resource_uri"].as_str().expect("recording uri"),
        harness.runtime_root.path(),
    )
    .expect("recording resource should load");
    assert_eq!(recording_contents.len(), 2);

    let Json(replay) = server
        .start_replay(Parameters(StartReplayParams {
            artifact_id: Some("recording-54-0000000000000002".to_string()),
            resource_uri: None,
            data_base64: None,
        }))
        .await
        .expect("replay should start");
    let replay = replay.into_value();
    assert_eq!(replay["status"], "replaying");

    let requests = harness.requests();
    assert_eq!(requests[0]["verb"], "get_snapshot");
    assert_eq!(requests[1]["verb"], "set_paused");
    assert_eq!(requests[2]["verb"], "step");
    assert_eq!(requests[2]["ticks"], 3);
    assert_eq!(requests[3]["verb"], "set_time_scale");
    assert_eq!(requests[4]["verb"], "inject_input");
    assert_eq!(requests[5]["verb"], "get_metrics_trace");
    assert_eq!(requests[6]["verb"], "capture_frame");
    assert_eq!(requests[7]["verb"], "stop_recording");
    assert_eq!(requests[8]["verb"], "start_replay");
    let replay_data = requests[8]["data"]
        .as_array()
        .expect("replay bytes should serialize");
    assert_eq!(replay_data.len(), 4);
}

#[test]
fn discovery_and_knowledge_resources_work() {
    let harness = TestHarness::manifest_only();
    let contexts = discovery::discover_contexts(harness.runtime_root.path());
    assert_eq!(contexts.len(), 1);

    let found = discovery::find_context(
        harness.runtime_root.path(),
        harness.route.context_id,
        Some(harness.route.process_nonce),
    )
    .expect("context lookup should succeed");
    assert_eq!(found.route.route_id, harness.route);

    let sdk_resource =
        resources::read_resource(resources::SDK_KNOWLEDGE_URI, harness.runtime_root.path())
            .expect("sdk knowledge should load");
    assert_eq!(sdk_resource.len(), 1);

    let workflow_resource =
        resources::read_resource(resources::MCP_WORKFLOW_URI, harness.runtime_root.path())
            .expect("workflow knowledge should load");
    assert_eq!(workflow_resource.len(), 1);
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

        while requests.lock().expect("request log").len() < 9 {
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
