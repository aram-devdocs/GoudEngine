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
