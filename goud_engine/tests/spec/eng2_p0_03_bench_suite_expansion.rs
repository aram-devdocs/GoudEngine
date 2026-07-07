use std::path::PathBuf;

use goud_engine::libs::graphics::renderer3d::PrimitiveType;

#[path = "../../benches/helpers/scene3d.rs"]
mod scene3d;

#[test]
fn eng2_p0_03_registers_cull_primitive_and_real_gpu_shadow_benches() {
    let frame_bench_source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/renderer3d_frame_benchmarks.rs"),
    )
    .expect("renderer3d frame bench source should exist");
    let real_gpu_bench_source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("benches/renderer3d_real_wgpu_shadow_benchmarks.rs"),
    )
    .expect("real-wgpu shadow bench source should exist");
    let cargo_toml =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("Cargo.toml should exist");
    let bench_gate_source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("goud_engine should live under the repository root")
            .join("scripts/bench-gate.py"),
    )
    .expect("bench-gate.py should exist");

    assert!(frame_bench_source.contains("benchmark_group(\"cull_scaling\")"));
    assert!(frame_bench_source.contains("benchmark_group(\"primitive_draw_calls\")"));
    assert!(
        bench_gate_source.contains("\"cull_scaling\""),
        "expected bench-gate.py to track the cull_scaling group by default"
    );
    assert!(
        bench_gate_source.contains("\"primitive_draw_calls\""),
        "expected bench-gate.py to track the primitive_draw_calls group by default"
    );
    assert!(
        cargo_toml.contains("name = \"renderer3d_real_wgpu_shadow_benchmarks\""),
        "expected Cargo.toml to register the opt-in real-GPU shadow bench target"
    );
    assert!(
        real_gpu_bench_source.contains("benchmark_group(\"shadow_record_real_gpu\")"),
        "expected an opt-in real-GPU shadow bench group in renderer3d_real_wgpu_shadow_benchmarks.rs"
    );
    assert!(
        real_gpu_bench_source.contains("GOUD_BENCH_REAL_WGPU_SHADOW"),
        "expected the real-GPU shadow bench to be guarded by an explicit opt-in env var"
    );
}

#[test]
fn eng2_p0_03_cull_scaling_scene_keeps_visible_count_fixed() {
    let mut renderer = scene3d::cull_scaling_scene(10_000, 5_000);
    renderer.render(None);
    let stats = renderer.stats();

    assert_eq!(stats.total_objects, 10_000);
    assert_eq!(stats.visible_objects, 5_000);
    assert_eq!(stats.culled_objects, 5_000);
    assert_eq!(stats.draw_calls, 5_000);
}

#[test]
fn eng2_p0_03_plane_and_cube_scenes_pin_draw_calls() {
    for primitive in [PrimitiveType::Plane, PrimitiveType::Cube] {
        let mut renderer = scene3d::dynamic_primitive_scene(512, primitive);
        renderer.render(None);
        let stats = renderer.stats();

        assert_eq!(stats.total_objects, 512);
        assert_eq!(stats.visible_objects, 512);
        assert_eq!(stats.culled_objects, 0);
        assert_eq!(stats.draw_calls, 512);
    }
}
