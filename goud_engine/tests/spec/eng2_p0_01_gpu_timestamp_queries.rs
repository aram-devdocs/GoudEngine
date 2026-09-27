use std::process::Command;

use goud_engine::libs::graphics::backend::wgpu_backend::{
    probe_gpu_timestamp_queries, GpuTimestampProbeReport,
};

#[test]
fn eng2_p0_01_query_set_populates_and_resolves_on_wgpu_backend() {
    let report = match probe_gpu_timestamp_queries() {
        Ok(report) => report,
        Err(err) => {
            eprintln!("Skipping ENG2-P0-01 probe: {err}");
            return;
        }
    };

    if !report.supported {
        eprintln!("Skipping ENG2-P0-01 probe: adapter does not expose timestamp-query features");
        return;
    }
    if has_macos_26_metal_zero_timestamp_bug(&report) {
        eprintln!(
            "Skipping ENG2-P0-01 probe: {} on {} returned zero timestamp queries; \
             wgpu #9414 tracks the macOS 26 Metal 4 counter-sample regression",
            report.adapter_name, report.backend
        );
        return;
    }
    assert!(
        report.raw_queries.iter().all(|query| *query > 0),
        "expected every timestamp slot to be written, got {:?}",
        report
    );
    assert!(
        report.raw_queries[1] > report.raw_queries[0],
        "expected shadow timestamps to advance, got {:?}",
        report.raw_queries
    );
    assert!(
        report.raw_queries[3] > report.raw_queries[2],
        "expected render timestamps to advance, got {:?}",
        report.raw_queries
    );
    assert!(
        report.raw_queries[5] > report.raw_queries[4],
        "expected submit-tail timestamps to advance, got {:?}",
        report.raw_queries
    );
    assert!(
        report.shadow_us > 0,
        "expected non-zero gpu_shadow duration, got {:?}",
        report
    );
    assert!(
        report.render_us > 0,
        "expected non-zero gpu_render duration, got {:?}",
        report
    );
    assert!(
        report.submit_us > 0,
        "expected non-zero gpu_submit duration, got {:?}",
        report
    );
}

fn has_macos_26_metal_zero_timestamp_bug(report: &GpuTimestampProbeReport) -> bool {
    cfg!(target_os = "macos")
        && report.backend == "metal"
        && report.raw_queries.iter().all(|query| *query == 0)
        && macos_major_version().is_some_and(|major| major >= 26)
}

fn macos_major_version() -> Option<u32> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8(output.stdout).ok()?;
    version.trim().split('.').next()?.parse().ok()
}
