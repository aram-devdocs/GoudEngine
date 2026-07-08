use std::path::Path;
use std::process::Command;

use serde_json::Value;

const FRAME_SPANS: &[&str] = &[
    "wgpu.begin_frame",
    "wgpu.end_frame",
    "wgpu.uniform_upload",
    "wgpu.shadow_pass",
    "wgpu.render_pass",
    "wgpu.gpu_submit",
];

const ECS_SPANS: &[&str] = &[
    "ecs.run_system",
    "ecs.system_stage",
    "ecs.system",
    "ecs.parallel_stage",
    "ecs.parallel_system",
];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("engine crate should live under the workspace root")
}

fn workspace_metadata() -> Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root())
        .output()
        .expect("cargo metadata should run for the workspace");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("cargo metadata should emit JSON")
}

fn metadata_package<'a>(metadata: &'a Value, package_name: &str) -> &'a Value {
    metadata["packages"]
        .as_array()
        .expect("metadata should include packages")
        .iter()
        .find(|package| package["name"].as_str() == Some(package_name))
        .unwrap_or_else(|| panic!("metadata should include package {package_name}"))
}

fn metadata_dependency<'a>(package: &'a Value, dependency_name: &str) -> &'a Value {
    package["dependencies"]
        .as_array()
        .expect("metadata package should include dependencies")
        .iter()
        .find(|dependency| dependency["name"].as_str() == Some(dependency_name))
        .unwrap_or_else(|| panic!("metadata should include dependency {dependency_name}"))
}

fn feature_entries<'a>(package: &'a Value, feature_name: &str) -> Vec<&'a str> {
    package["features"][feature_name]
        .as_array()
        .unwrap_or_else(|| panic!("metadata should include feature {feature_name}"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("feature {feature_name} entries should be strings"))
        })
        .collect()
}

fn markdown_shell_commands(docs: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut in_shell_block = false;
    let mut continued = String::new();

    for line in docs.lines() {
        let trimmed = line.trim();
        if let Some(info) = trimmed.strip_prefix("```") {
            if in_shell_block {
                if !continued.is_empty() {
                    commands.push(shell_tokens(&continued));
                    continued.clear();
                }
                in_shell_block = false;
            } else {
                in_shell_block = matches!(info.trim(), "bash" | "sh" | "shell" | "");
            }
            continue;
        }

        if !in_shell_block || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_part = trimmed.strip_suffix('\\').unwrap_or(trimmed).trim();
        if !continued.is_empty() {
            continued.push(' ');
        }
        continued.push_str(line_part);

        if !trimmed.ends_with('\\') {
            commands.push(shell_tokens(&continued));
            continued.clear();
        }
    }

    commands
}

fn shell_tokens(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(|token| token.trim_matches(['"', '\'']).to_owned())
        .collect()
}

fn assert_docs_have_command(commands: &[Vec<String>], required_tokens: &[&str]) {
    assert!(
        commands
            .iter()
            .any(|command| command_has_tokens(command, required_tokens)),
        "profiling guide should document a command containing {required_tokens:?}"
    );
}

fn command_has_tokens(command: &[String], required_tokens: &[&str]) -> bool {
    required_tokens
        .iter()
        .all(|required| command.iter().any(|token| token == required))
}

fn docs_have_sentence_with_terms(docs: &str, terms: &[&str]) -> bool {
    docs.split_terminator(['.', '\n'])
        .map(str::to_ascii_lowercase)
        .any(|sentence| {
            terms
                .iter()
                .all(|term| sentence.contains(&term.to_ascii_lowercase()))
        })
}

fn dependency_tree_has_exact_package(tree: &str, package_name: &str) -> bool {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .any(|name| name == package_name)
}

fn assert_unique_spans(spans: &[&str]) {
    for (index, span) in spans.iter().enumerate() {
        assert!(
            !spans[..index].contains(span),
            "profiling span {span} should appear once in its contract list"
        );
    }
}

#[test]
fn eng2_p0_04_manifest_declares_backend_features_outside_defaults() {
    let metadata = workspace_metadata();
    let engine = metadata_package(&metadata, "goud-engine-core");
    let default_features = feature_entries(engine, "default");
    let tracy_features = feature_entries(engine, "profiling-tracy");
    let puffin_features = feature_entries(engine, "profiling-puffin");

    assert!(
        tracy_features.contains(&"dep:profiling")
            && tracy_features.contains(&"profiling/profile-with-tracy"),
        "ENG2-P0-04 must expose a Tracy-backed profiling feature"
    );
    assert!(
        puffin_features.contains(&"dep:profiling") && puffin_features.contains(&"dep:puffin"),
        "ENG2-P0-04 must expose a Puffin-backed profiling feature"
    );
    assert!(
        !default_features.contains(&"profiling-tracy"),
        "profiling-tracy must stay out of default features"
    );
    assert!(
        !default_features.contains(&"profiling-puffin"),
        "profiling-puffin must stay out of default features"
    );

    let profiling_dep = metadata_dependency(engine, "profiling");
    assert_eq!(
        profiling_dep["optional"].as_bool(),
        Some(true),
        "profiling crate should be optional"
    );
    assert_eq!(
        profiling_dep["uses_default_features"].as_bool(),
        Some(false),
        "profiling crate should not activate default backend features"
    );
    assert_eq!(
        metadata_dependency(engine, "puffin")["optional"].as_bool(),
        Some(true),
        "Puffin crate should be optional"
    );
}

#[test]
fn eng2_p0_04_default_dependency_graph_excludes_profiler_backends() {
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "goud-engine-core",
            "-e",
            "normal",
            "--prefix",
            "none",
            "--charset",
            "ascii",
        ])
        .current_dir(repo_root())
        .output()
        .expect("cargo tree should run for the default dependency graph");

    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tree = String::from_utf8_lossy(&output.stdout);
    for package in ["tracy-client", "tracy-client-sys", "puffin"] {
        assert!(
            !dependency_tree_has_exact_package(&tree, package),
            "default dependency graph must not activate {package}:\n{tree}"
        );
    }
}

#[test]
fn eng2_p0_04_docs_include_backend_commands_and_runtime_contracts() {
    let docs = include_str!("../../../docs/src/development/profiling.md");
    let metadata = workspace_metadata();
    let sandbox = metadata_package(&metadata, "sandbox");
    let engine_dependency = metadata_dependency(sandbox, "goud-engine-core");
    let engine_dependency_name = engine_dependency["rename"]
        .as_str()
        .unwrap_or("goud-engine-core");
    let tracy_feature = format!("{engine_dependency_name}/profiling-tracy");
    let puffin_feature = format!("{engine_dependency_name}/profiling-puffin");
    let commands = markdown_shell_commands(docs);

    assert!(docs_have_sentence_with_terms(docs, &["tracy", "viewer"]));
    assert!(docs_have_sentence_with_terms(docs, &["puffin_viewer"]));
    assert!(docs_have_sentence_with_terms(
        docs,
        &["all-features", "tracy"]
    ));
    assert!(docs_have_sentence_with_terms(
        docs,
        &["headless", "ecs", "puffin", "advance"]
    ));

    assert_docs_have_command(
        &commands,
        &[
            "cargo",
            "run",
            "-p",
            "sandbox",
            "--features",
            &tracy_feature,
        ],
    );
    assert_docs_have_command(
        &commands,
        &[
            "cargo",
            "run",
            "-p",
            "sandbox",
            "--features",
            &puffin_feature,
        ],
    );
    assert_docs_have_command(
        &commands,
        &[
            "cargo",
            "check",
            "-p",
            "sandbox",
            "--features",
            &tracy_feature,
        ],
    );
    assert_docs_have_command(
        &commands,
        &[
            "cargo",
            "check",
            "-p",
            "sandbox",
            "--features",
            &puffin_feature,
        ],
    );
}

#[test]
fn eng2_p0_04_docs_list_frame_and_ecs_spans() {
    let docs = include_str!("../../../docs/src/development/profiling.md");

    assert_unique_spans(FRAME_SPANS);
    assert_unique_spans(ECS_SPANS);

    for span in FRAME_SPANS.iter().chain(ECS_SPANS.iter()) {
        assert!(
            docs.contains(span),
            "profiling guide should list the {span:?} span"
        );
    }
}

#[cfg(feature = "profiling-tracy")]
#[test]
fn eng2_p0_04_tracy_scope_macros_compile_when_enabled() {
    let _client = profiling::tracy_client::Client::start();

    profiling::scope!("eng2_p0_04_spec_tracy_scope");
    profiling::finish_frame!();
}

#[cfg(feature = "profiling-puffin")]
#[test]
fn eng2_p0_04_puffin_scope_macros_compile_when_enabled() {
    puffin::set_scopes_on(true);

    puffin::profile_scope!("eng2_p0_04_spec_puffin_scope");
    puffin::GlobalProfiler::lock().new_frame();
}

#[cfg(all(feature = "profiling-tracy", feature = "profiling-puffin"))]
#[test]
fn eng2_p0_04_all_features_exercise_tracy_backend() {
    puffin::set_scopes_on(false);
    let _client = profiling::tracy_client::Client::start();

    profiling::scope!("eng2_p0_04_spec_all_features_tracy_scope");
    profiling::finish_frame!();

    assert!(
        !puffin::are_scopes_on(),
        "the all-features spec smoke should exercise Tracy without needing Puffin capture"
    );
}
