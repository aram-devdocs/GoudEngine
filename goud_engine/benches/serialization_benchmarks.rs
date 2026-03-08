//! Serialization Performance Benchmarks
//!
//! Benchmarks for binary encoding/decoding, delta encoding, and size comparisons.
//!
//! Run with: `cargo bench --bench serialization_benchmarks`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use goud_engine::core::math::{Color, Rect, Vec2, Vec3, Vec4};
use goud_engine::core::serialization::{binary, DeltaEncode};
use goud_engine::ecs::components::Transform2D;

// =============================================================================
// Test data helpers
// =============================================================================

fn sample_vec2() -> Vec2 {
    Vec2::new(123.456, -789.012)
}

fn sample_vec3() -> Vec3 {
    Vec3::new(1.0, 2.0, 3.0)
}

fn sample_vec4() -> Vec4 {
    Vec4::new(1.0, 2.0, 3.0, 4.0)
}

fn sample_color() -> Color {
    Color {
        r: 0.8,
        g: 0.2,
        b: 0.5,
        a: 1.0,
    }
}

fn sample_rect() -> Rect {
    Rect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
    }
}

fn sample_transform2d() -> Transform2D {
    Transform2D {
        position: Vec2::new(100.0, 200.0),
        rotation: 1.57,
        scale: Vec2::new(2.0, 2.0),
    }
}

/// Returns (baseline, target) with a small change in one field.
fn delta_pair_vec2() -> (Vec2, Vec2) {
    (Vec2::new(100.0, 200.0), Vec2::new(101.0, 200.0))
}

fn delta_pair_vec3() -> (Vec3, Vec3) {
    (Vec3::new(1.0, 2.0, 3.0), Vec3::new(1.0, 2.5, 3.0))
}

fn delta_pair_vec4() -> (Vec4, Vec4) {
    (Vec4::new(1.0, 2.0, 3.0, 4.0), Vec4::new(1.0, 2.0, 3.0, 4.5))
}

fn delta_pair_color() -> (Color, Color) {
    let base = Color {
        r: 1.0,
        g: 0.5,
        b: 0.0,
        a: 1.0,
    };
    let target = Color {
        r: 1.0,
        g: 0.6,
        b: 0.0,
        a: 1.0,
    };
    (base, target)
}

fn delta_pair_rect() -> (Rect, Rect) {
    let base = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };
    let target = Rect {
        x: 5.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };
    (base, target)
}

fn delta_pair_transform2d() -> (Transform2D, Transform2D) {
    let base = Transform2D {
        position: Vec2::new(100.0, 200.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
    };
    let target = Transform2D {
        position: Vec2::new(102.0, 200.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
    };
    (base, target)
}

// =============================================================================
// Binary encode benchmarks
// =============================================================================

fn bench_binary_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_encode");

    group.bench_function("vec2", |b| {
        let v = sample_vec2();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.bench_function("vec3", |b| {
        let v = sample_vec3();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.bench_function("vec4", |b| {
        let v = sample_vec4();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.bench_function("color", |b| {
        let v = sample_color();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.bench_function("rect", |b| {
        let v = sample_rect();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.bench_function("transform2d", |b| {
        let v = sample_transform2d();
        b.iter(|| binary::encode(black_box(&v)).unwrap());
    });

    group.finish();
}

// =============================================================================
// Binary decode benchmarks
// =============================================================================

fn bench_binary_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_decode");

    let vec2_bytes = binary::encode(&sample_vec2()).unwrap();
    group.bench_function("vec2", |b| {
        b.iter(|| binary::decode::<Vec2>(black_box(&vec2_bytes)).unwrap());
    });

    let vec3_bytes = binary::encode(&sample_vec3()).unwrap();
    group.bench_function("vec3", |b| {
        b.iter(|| binary::decode::<Vec3>(black_box(&vec3_bytes)).unwrap());
    });

    let vec4_bytes = binary::encode(&sample_vec4()).unwrap();
    group.bench_function("vec4", |b| {
        b.iter(|| binary::decode::<Vec4>(black_box(&vec4_bytes)).unwrap());
    });

    let color_bytes = binary::encode(&sample_color()).unwrap();
    group.bench_function("color", |b| {
        b.iter(|| binary::decode::<Color>(black_box(&color_bytes)).unwrap());
    });

    let rect_bytes = binary::encode(&sample_rect()).unwrap();
    group.bench_function("rect", |b| {
        b.iter(|| binary::decode::<Rect>(black_box(&rect_bytes)).unwrap());
    });

    let t2d_bytes = binary::encode(&sample_transform2d()).unwrap();
    group.bench_function("transform2d", |b| {
        b.iter(|| binary::decode::<Transform2D>(black_box(&t2d_bytes)).unwrap());
    });

    group.finish();
}

// =============================================================================
// JSON vs Binary size comparison
// =============================================================================

fn bench_json_vs_binary_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_vs_binary_size");

    let types: Vec<(&str, Vec<u8>, Vec<u8>)> = vec![
        (
            "Vec2",
            serde_json::to_vec(&sample_vec2()).unwrap(),
            binary::encode(&sample_vec2()).unwrap(),
        ),
        (
            "Vec3",
            serde_json::to_vec(&sample_vec3()).unwrap(),
            binary::encode(&sample_vec3()).unwrap(),
        ),
        (
            "Vec4",
            serde_json::to_vec(&sample_vec4()).unwrap(),
            binary::encode(&sample_vec4()).unwrap(),
        ),
        (
            "Color",
            serde_json::to_vec(&sample_color()).unwrap(),
            binary::encode(&sample_color()).unwrap(),
        ),
        (
            "Rect",
            serde_json::to_vec(&sample_rect()).unwrap(),
            binary::encode(&sample_rect()).unwrap(),
        ),
        (
            "Transform2D",
            serde_json::to_vec(&sample_transform2d()).unwrap(),
            binary::encode(&sample_transform2d()).unwrap(),
        ),
    ];

    for (name, json_bytes, bin_bytes) in &types {
        // Print size comparison (visible in benchmark output)
        println!(
            "{}: JSON={} bytes, Binary={} bytes, ratio={:.1}x",
            name,
            json_bytes.len(),
            bin_bytes.len(),
            json_bytes.len() as f64 / bin_bytes.len() as f64
        );

        // Benchmark JSON encode to measure speed difference
        group.bench_with_input(
            BenchmarkId::new("json_encode", name),
            name,
            |b, _name| match *name {
                "Vec2" => {
                    let v = sample_vec2();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                "Vec3" => {
                    let v = sample_vec3();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                "Vec4" => {
                    let v = sample_vec4();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                "Color" => {
                    let v = sample_color();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                "Rect" => {
                    let v = sample_rect();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                "Transform2D" => {
                    let v = sample_transform2d();
                    b.iter(|| serde_json::to_vec(black_box(&v)).unwrap());
                }
                _ => unreachable!(),
            },
        );

        group.bench_with_input(
            BenchmarkId::new("binary_encode", name),
            name,
            |b, _name| match *name {
                "Vec2" => {
                    let v = sample_vec2();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                "Vec3" => {
                    let v = sample_vec3();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                "Vec4" => {
                    let v = sample_vec4();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                "Color" => {
                    let v = sample_color();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                "Rect" => {
                    let v = sample_rect();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                "Transform2D" => {
                    let v = sample_transform2d();
                    b.iter(|| binary::encode(black_box(&v)).unwrap());
                }
                _ => unreachable!(),
            },
        );
    }

    group.finish();
}

// =============================================================================
// Delta encode benchmarks
// =============================================================================

fn bench_delta_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_encode");

    let (base, target) = delta_pair_vec2();
    group.bench_function("vec2", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    let (base, target) = delta_pair_vec3();
    group.bench_function("vec3", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    let (base, target) = delta_pair_vec4();
    group.bench_function("vec4", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    let (base, target) = delta_pair_color();
    group.bench_function("color", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    let (base, target) = delta_pair_rect();
    group.bench_function("rect", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    let (base, target) = delta_pair_transform2d();
    group.bench_function("transform2d", |b| {
        b.iter(|| black_box(&target).delta_from(black_box(&base)));
    });

    group.finish();
}

// =============================================================================
// Delta decode (apply_delta) benchmarks
// =============================================================================

fn bench_delta_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_decode");

    let (base, target) = delta_pair_vec2();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("vec2", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    let (base, target) = delta_pair_vec3();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("vec3", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    let (base, target) = delta_pair_vec4();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("vec4", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    let (base, target) = delta_pair_color();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("color", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    let (base, target) = delta_pair_rect();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("rect", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    let (base, target) = delta_pair_transform2d();
    let delta = target.delta_from(&base).unwrap();
    group.bench_function("transform2d", |b| {
        b.iter(|| black_box(&base).apply_delta(black_box(&delta)));
    });

    group.finish();
}

// =============================================================================
// Delta size comparison
// =============================================================================

fn bench_delta_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_size");

    // Measure and report delta vs full sizes for typical partial updates
    let pairs: Vec<(&str, usize, usize)> = vec![
        {
            let (base, target) = delta_pair_vec2();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Vec2", delta.data.len(), full)
        },
        {
            let (base, target) = delta_pair_vec3();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Vec3", delta.data.len(), full)
        },
        {
            let (base, target) = delta_pair_vec4();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Vec4", delta.data.len(), full)
        },
        {
            let (base, target) = delta_pair_color();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Color", delta.data.len(), full)
        },
        {
            let (base, target) = delta_pair_rect();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Rect", delta.data.len(), full)
        },
        {
            let (base, target) = delta_pair_transform2d();
            let delta = target.delta_from(&base).unwrap();
            let full = binary::encode(&target).unwrap().len();
            ("Transform2D", delta.data.len(), full)
        },
    ];

    for (name, delta_sz, full_sz) in &pairs {
        println!(
            "{}: delta_data={} bytes, full={} bytes, savings={:.0}%",
            name,
            delta_sz,
            full_sz,
            (1.0 - *delta_sz as f64 / *full_sz as f64) * 100.0,
        );
    }

    // Benchmark the full delta+encode pipeline vs plain encode
    let (base, target) = delta_pair_transform2d();
    group.bench_function("transform2d_delta_pipeline", |b| {
        b.iter(|| {
            let delta = black_box(&target).delta_from(black_box(&base)).unwrap();
            black_box(binary::encode(&delta).unwrap());
        });
    });

    group.bench_function("transform2d_full_encode", |b| {
        b.iter(|| {
            black_box(binary::encode(black_box(&target)).unwrap());
        });
    });

    group.finish();
}

// =============================================================================
// Criterion configuration
// =============================================================================

criterion_group!(encode_benches, bench_binary_encode, bench_binary_decode,);

criterion_group!(delta_benches, bench_delta_encode, bench_delta_decode,);

criterion_group!(
    comparison_benches,
    bench_json_vs_binary_size,
    bench_delta_size,
);

criterion_main!(encode_benches, delta_benches, comparison_benches);
