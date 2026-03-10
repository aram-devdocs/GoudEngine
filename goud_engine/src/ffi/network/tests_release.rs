use std::path::{Path, PathBuf};
use std::{env, process::Command};

use super::*;
use crate::core::error::last_error_message;

fn release_probe_binary_path(target_dir: &Path) -> PathBuf {
    target_dir.join("release").join(format!(
        "network_sim_release_probe{}",
        std::env::consts::EXE_SUFFIX
    ))
}

fn build_release_probe_binary() -> PathBuf {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let target_dir = env::temp_dir().join("goud-network-release-probe");
    let probe_binary = release_probe_binary_path(&target_dir);

    let status = Command::new(env!("CARGO"))
        .current_dir(workspace_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "build",
            "-p",
            "goud-engine-core",
            "--release",
            "--bin",
            "network_sim_release_probe",
            "--quiet",
        ])
        .status()
        .expect("failed to build release probe binary");

    assert!(status.success(), "release probe build exited with {status}");
    probe_binary
}

#[test]
fn test_release_stub_behavior_helper_returns_expected_error() {
    let _registry = RegistryResetGuard::new();

    let code = simulation_controls_unavailable();

    assert_eq!(code, ERR_INVALID_STATE);
    let message = last_error_message().expect("expected error message");
    assert!(
        message.contains("only available in debug/test builds"),
        "unexpected message: {message}"
    );
}

#[test]
fn test_release_binary_exercises_exported_simulation_stubs() {
    let probe_binary = build_release_probe_binary();
    let status = Command::new(&probe_binary)
        .status()
        .expect("failed to execute release probe binary");

    assert!(status.success(), "release probe exited with {status}");
}
