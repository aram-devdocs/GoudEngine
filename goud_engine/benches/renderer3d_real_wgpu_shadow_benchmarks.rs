//! Optional real-wgpu shadow benchmark.
//!
//! This stays separate from `renderer3d_frame_benchmarks.rs` so the default
//! NullBackend suite remains CPU-only and CI-safe. The opt-in run creates a
//! small native window because hidden/occluded wgpu surfaces do not produce
//! presentable frames. Opt in with:
//!
//! `GOUD_BENCH_REAL_WGPU_SHADOW=1 cargo bench --bench renderer3d_real_wgpu_shadow_benchmarks`
//!
//! In software-rasterized environments, combine it with
//! `GOUD_WGPU_FORCE_FALLBACK=1` and the usual Xvfb/lavapipe setup.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

#[path = "helpers/scene3d.rs"]
mod scene3d;

#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
fn bench_real_wgpu_shadow_record(c: &mut Criterion) {
    if let Some(reason) = scene3d::real_wgpu_shadow_bench_skip_reason() {
        eprintln!("Skipping shadow_record_real_gpu bench: {reason}");
        return;
    }

    let mut scene = scene3d::RealWgpuScene::shadow_scene(1_400)
        .unwrap_or_else(|e| panic!("failed to create real-wgpu shadow scene: {e}"));

    let mut group = c.benchmark_group("shadow_record_real_gpu");
    for (label, n) in [("casters_1400", 1_400usize), ("casters_5k", 5_000usize)] {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(label, |b| {
            scene
                .reset_shadow_scene(n)
                .unwrap_or_else(|e| panic!("failed to reset real-wgpu shadow scene: {e}"));
            scene
                .render_frame()
                .unwrap_or_else(|e| panic!("failed to warm real-wgpu shadow scene: {e}"));
            b.iter(|| {
                scene
                    .render_frame()
                    .unwrap_or_else(|e| panic!("failed to render real-wgpu shadow frame: {e}"))
            });
        });
    }

    group.finish();
}

#[cfg(not(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
)))]
fn bench_real_wgpu_shadow_record(_c: &mut Criterion) {}

criterion_group!(
    renderer3d_real_wgpu_shadow_benches,
    bench_real_wgpu_shadow_record,
);
criterion_main!(renderer3d_real_wgpu_shadow_benches);
