use rmcp::model::{GetPromptResult, Prompt, PromptMessage, PromptMessageRole};

use crate::resources::{
    CSHARP_SDK_KNOWLEDGE_URI, MCP_WORKFLOW_URI, PYTHON_SDK_KNOWLEDGE_URI, RUST_SDK_KNOWLEDGE_URI,
    SDK_KNOWLEDGE_URI, TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI,
};

pub const SAFE_ATTACH_PROMPT: &str = "goudengine.safe_attach";
pub const INSPECT_RUNTIME_PROMPT: &str = "goudengine.inspect_runtime";
pub const TROUBLESHOOT_ATTACH_PROMPT: &str = "goudengine.troubleshoot_attach";

pub fn static_prompts() -> Vec<Prompt> {
    vec![
        Prompt::new(
            SAFE_ATTACH_PROMPT,
            Some("Attach to a local GoudEngine debugger route without guessing at runtime state."),
            None,
        )
        .with_title("Safe Attach"),
        Prompt::new(
            INSPECT_RUNTIME_PROMPT,
            Some("Inspect snapshot, metrics, entity state, and artifacts through the shared debugger contract."),
            None,
        )
        .with_title("Inspect Runtime"),
        Prompt::new(
            TROUBLESHOOT_ATTACH_PROMPT,
            Some("Diagnose why a local debugger route is missing or attach is failing."),
            None,
        )
        .with_title("Troubleshoot Attach"),
    ]
}

pub fn get_prompt_result(name: &str) -> Option<GetPromptResult> {
    match name {
        SAFE_ATTACH_PROMPT => Some(GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Safely attach to a local GoudEngine runtime and establish the initial inspection context.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                format!(
                    "Use this order and stop if any earlier step fails:\n\
                     1. Read `{SDK_KNOWLEDGE_URI}` and `{MCP_WORKFLOW_URI}`.\n\
                     2. Call `goudengine.list_contexts` and pick one route-scoped context.\n\
                     3. Call `goudengine.attach_context` with the selected `contextId` and `processNonce`.\n\
                     4. Immediately call `goudengine.get_snapshot` before using control, capture, or replay.\n\
                     5. Prefer observation first; only call pause, step, replay, or input injection when the task explicitly needs mutation.\n\
                     6. For language-specific guidance, read one of: `{RUST_SDK_KNOWLEDGE_URI}`, `{CSHARP_SDK_KNOWLEDGE_URI}`, `{PYTHON_SDK_KNOWLEDGE_URI}`, `{TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI}`.\n\
                     Do not invent browser attach flows. TypeScript web is out of scope for this batch."
                ),
            ),
        ])
        .with_description("Safe debugger attach workflow for local agent clients.")),
        INSPECT_RUNTIME_PROMPT => Some(GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Inspect the current runtime through the shared debugger contract and summarize what is safe to do next.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                format!(
                    "After attaching:\n\
                     - Call `goudengine.get_snapshot` to establish frame, route, and selection state.\n\
                     - Use `goudengine.inspect_entity` only when one entity needs expanded detail.\n\
                     - Use `goudengine.get_metrics_trace` for profiler, render, memory, and service-health export.\n\
                     - Use `goudengine.capture_frame` for framebuffer plus JSON attachments.\n\
                     - Use replay only through `goudengine.start_recording`, `goudengine.stop_recording`, `goudengine.start_replay`, and `goudengine.stop_replay`.\n\
                     - Keep interpretation aligned with `{SDK_KNOWLEDGE_URI}` and `{MCP_WORKFLOW_URI}`.\n\
                     Report desktop-only limitations clearly, especially that browser/WASM debugger support is not part of this batch."
                ),
            ),
        ])
        .with_description("Runtime inspection workflow using snapshots, metrics, capture, and replay.")),
        TROUBLESHOOT_ATTACH_PROMPT => Some(GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Troubleshoot why local debugger discovery or attach is failing for a GoudEngine app.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                format!(
                    "Check these conditions in order:\n\
                     1. The app must enable debugger mode before startup through `DebuggerConfig` or `ContextConfig`.\n\
                     2. `publish_local_attach` / `publishLocalAttach` must be true when attachable routes should be discoverable.\n\
                     3. The target must be a supported desktop or headless native flow, not `goudengine/web`.\n\
                     4. `goudengine.list_contexts` should show at least one route with the expected label or context id.\n\
                     5. If the route exists but attach fails, compare the process nonce and route id from the manifest before retrying.\n\
                     6. Use `{RUST_SDK_KNOWLEDGE_URI}`, `{CSHARP_SDK_KNOWLEDGE_URI}`, `{PYTHON_SDK_KNOWLEDGE_URI}`, or `{TYPESCRIPT_DESKTOP_SDK_KNOWLEDGE_URI}` to match the SDK-specific config path.\n\
                     Keep the diagnosis local-only. Do not suggest remote bind or browser debugger transports."
                ),
            ),
        ])
        .with_description("Attach failure checklist aligned with the Rust-owned debugger contract.")),
        _ => None,
    }
}
