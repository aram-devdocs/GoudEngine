mod support;

use rmcp::ServerHandler;

use super::GoudEngineMcpServer;
use crate::discovery;
use crate::prompts;
use crate::resources;

use self::support::TestHarness;

#[tokio::test]
async fn list_attach_control_capture_replay_and_resources_work() {
    let harness = TestHarness::new();
    let server = harness.server();

    let listed = support::list_contexts(&server).await;
    let contexts = listed["contexts"]
        .as_array()
        .expect("contexts should be an array");
    assert_eq!(contexts.len(), 1);
    assert_eq!(
        contexts[0]["route"]["route_id"]["context_id"],
        harness.route.context_id
    );
    assert!(listed["attached_context"].is_null());

    let attached = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;
    assert_eq!(
        attached["context"]["route"]["route_id"]["context_id"],
        harness.route.context_id
    );
    assert_eq!(
        attached["session"]["snapshot_schema"],
        "debugger_snapshot_v1"
    );

    let snapshot = support::snapshot(&server).await;
    assert_eq!(snapshot["frame"]["index"], 42);

    let paused = support::set_paused(&server, true).await;
    assert_eq!(paused["paused"], true);

    let step = support::step_ticks(&server, 3).await;
    assert_eq!(step["accepted"], true);

    let time_scale = support::set_time_scale(&server, 0.5).await;
    assert_eq!(time_scale["time_scale"], 0.5);

    let input = support::inject_mouse_move(&server).await;
    assert_eq!(input["accepted"], true);

    let metrics = support::metrics_trace(&server).await;
    assert_eq!(metrics["artifact_id"], "metrics-54-0000000000000003");
    assert_eq!(
        metrics["resource_uri"],
        "goudengine://metrics/metrics-54-0000000000000003"
    );

    let capture = support::capture_frame(&server).await;
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

    let recording = support::stop_recording(&server).await;
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

    let replay = support::start_replay(&server, "recording-54-0000000000000002".to_string()).await;
    assert_eq!(replay["status"], "replaying");

    let diagnostics = support::get_diagnostics(&server).await;
    assert!(diagnostics["diagnostics"]["render"]["draw_calls"] == 42);
    assert!(diagnostics["diagnostics"]["audio"]["active_sources"] == 3);

    let subsystem = support::get_subsystem_diagnostics(&server, "render").await;
    assert_eq!(subsystem["key"], "render");
    assert_eq!(subsystem["diagnostics"]["draw_calls"], 42);

    let logs = support::get_logs(&server, None).await;
    let log_entries = logs["logs"].as_array().expect("logs should be an array");
    assert_eq!(log_entries.len(), 1);
    assert_eq!(log_entries[0]["level"], "INFO");
    assert_eq!(log_entries[0]["message"], "tick");

    let hierarchy = support::get_scene_hierarchy(&server).await;
    let entities = hierarchy["entities"]
        .as_array()
        .expect("entities should be an array");
    assert_eq!(entities.len(), 3);
    assert_eq!(entities[0]["name"], "Root");
    assert!(entities[0]["child_entity_ids"].as_array().unwrap().len() == 2);
    assert_eq!(entities[1]["parent_entity_id"], 1);

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
    assert_eq!(requests[9]["verb"], "get_diagnostics");
    assert_eq!(requests[10]["verb"], "get_diagnostics_for");
    assert_eq!(requests[10]["key"], "render");
    assert_eq!(requests[11]["verb"], "get_logs");
    assert_eq!(requests[12]["verb"], "get_scene_hierarchy");
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

    for uri in [
        resources::SDK_KNOWLEDGE_URI,
        resources::MCP_WORKFLOW_URI,
        resources::RUST_SDK_KNOWLEDGE_URI,
        resources::CSHARP_SDK_KNOWLEDGE_URI,
        resources::PYTHON_SDK_KNOWLEDGE_URI,
        resources::TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI,
    ] {
        let resource = resources::read_resource(uri, harness.runtime_root.path())
            .expect("knowledge resource should load");
        assert_eq!(resource.len(), 1);
    }
}

#[tokio::test]
async fn prompt_listing_and_lookup_work() {
    let server = GoudEngineMcpServer::new();
    let info = server.get_info();
    assert!(info.capabilities.prompts.is_some());

    let listed = prompts::static_prompts();
    assert_eq!(listed.len(), 3);
    assert_eq!(listed[0].name, prompts::SAFE_ATTACH_PROMPT);
    assert_eq!(listed[1].name, prompts::INSPECT_RUNTIME_PROMPT);
    assert_eq!(listed[2].name, prompts::TROUBLESHOOT_ATTACH_PROMPT);

    let prompt = prompts::get_prompt_result(prompts::SAFE_ATTACH_PROMPT)
        .expect("safe attach prompt should load");
    assert_eq!(prompt.messages.len(), 2);

    let instructions = match &prompt.messages[1].content {
        rmcp::model::PromptMessageContent::Text { text } => text.as_str(),
        other => panic!("expected text prompt content, got {other:?}"),
    };
    assert!(instructions.contains("goudengine.list_contexts"));
    assert!(instructions.contains(resources::RUST_SDK_KNOWLEDGE_URI));
    assert!(instructions.contains(resources::TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI));

    let troubleshoot = prompts::get_prompt_result(prompts::TROUBLESHOOT_ATTACH_PROMPT)
        .expect("troubleshoot prompt should exist");
    let troubleshoot_text = match &troubleshoot.messages[1].content {
        rmcp::model::PromptMessageContent::Text { text } => text.as_str(),
        other => panic!("expected text prompt content, got {other:?}"),
    };
    assert!(troubleshoot_text.contains("publish_local_attach"));
    assert!(troubleshoot_text.contains("goudengine/web"));
}

// =========================================================================
// Focused tests for diagnostic, log, and scene hierarchy tools (Phase 6)
// =========================================================================

#[tokio::test]
async fn test_get_diagnostics_returns_map() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let diagnostics = support::get_diagnostics(&server).await;
    // The fake server wraps diagnostics under a "diagnostics" key with "render" and "audio"
    let diag_map = diagnostics
        .get("diagnostics")
        .expect("response should contain diagnostics key");
    assert!(
        diag_map.get("render").is_some(),
        "diagnostics should contain render key"
    );
    assert!(
        diag_map.get("audio").is_some(),
        "diagnostics should contain audio key"
    );
}

#[tokio::test]
async fn test_get_subsystem_diagnostics_valid_key() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let subsystem = support::get_subsystem_diagnostics(&server, "render").await;
    assert_eq!(subsystem["key"], "render");
    assert_eq!(subsystem["diagnostics"]["draw_calls"], 42);
}

#[tokio::test]
async fn test_get_subsystem_diagnostics_invalid_key() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let subsystem = support::get_subsystem_diagnostics(&server, "nonexistent_subsystem").await;
    // The fake server returns the key echoed back even for unknown subsystems;
    // the engine-side IPC would return null, but the MCP layer just forwards.
    assert_eq!(subsystem["key"], "nonexistent_subsystem");
}

#[tokio::test]
async fn test_get_logs_returns_entries() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let logs = support::get_logs(&server, None).await;
    let entries = logs["logs"]
        .as_array()
        .expect("logs response should contain a logs array");
    assert!(!entries.is_empty(), "logs should contain at least one entry");
    assert_eq!(entries[0]["level"], "INFO");
}

#[tokio::test]
async fn test_get_scene_hierarchy_returns_entities() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let hierarchy = support::get_scene_hierarchy(&server).await;
    let entities = hierarchy["entities"]
        .as_array()
        .expect("hierarchy response should contain entities array");
    assert!(
        !entities.is_empty(),
        "entities array should not be empty"
    );
    // Verify parent/child structure
    assert!(entities[0].get("name").is_some(), "entities should have names");
    assert!(
        entities[0].get("parent_entity_id").is_some(),
        "entities should have parent_entity_id field"
    );
    assert!(
        entities[0].get("child_entity_ids").is_some(),
        "entities should have child_entity_ids field"
    );
}

// =========================================================================
// Focused tests for diagnostics recording tools
// =========================================================================

#[tokio::test]
async fn test_get_diagnostics_recording_returns_slices() {
    let harness = TestHarness::new();
    let server = harness.server();
    let _ = support::attach_context(
        &server,
        harness.route.context_id,
        harness.route.process_nonce,
    )
    .await;

    let recording = support::get_diagnostics_recording(&server, 10).await;
    assert_eq!(recording["version"], 1);
    assert_eq!(recording["recording_id"], "diag-rec-54-100");
    assert_eq!(recording["total_frames"], 60);
    let slices = recording["slices"]
        .as_array()
        .expect("slices should be an array");
    assert_eq!(slices.len(), 10);
    assert_eq!(slices[0]["slice_index"], 0);
    assert_eq!(slices[0]["frame_count"], 6);
}
