//! Render Benchmark Suite
//!
//! Establishes performance baselines for the rendering pipeline before
//! Phase 0 optimizations. Benchmarks the full CPU pipeline: gather, sort,
//! vertex generation, and batch metric extraction.
//!
//! Run with: `cargo bench --bench render_benchmarks`
//!
//! Generate baseline JSON: `RENDER_BASELINE=1 cargo bench --bench render_benchmarks`
//!
//! ## Scenarios
//!
//! - empty: 0 entities
//! - light: 100 entities
//! - moderate: 1,000 entities
//! - target: 10,000 entities
//!
//! ## Throne Context
//!
//! 373 entities at 13.51ms render, 155-345 draw calls.
//! Target: <3ms, <100 draw calls.

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};
use goud_engine::assets::loaders::TextureAsset;
use goud_engine::assets::AssetHandle;
use goud_engine::assets::AssetServer;
use goud_engine::core::error::GoudResult;
use goud_engine::core::math::Vec2;
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::ecs::World;
use goud_engine::libs::graphics::backend::capabilities::{BackendCapabilities, BackendInfo};
use goud_engine::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, RenderTargetDesc, RenderTargetHandle,
    ShaderHandle, TextureFilter, TextureFormat, TextureHandle, TextureWrap, VertexLayout,
};
use goud_engine::libs::graphics::backend::{
    BlendFactor, BufferOps, ClearOps, CullFace, DrawOps, FrameOps, RenderBackend, RenderTargetOps,
    ShaderOps, StateOps, TextureOps,
};
use goud_engine::rendering::sprite_batch::{SpriteBatch, SpriteBatchConfig, SpriteInstance};
use serde::Serialize;
use std::time::Instant;

// ================================================================================================
// NullBackend — minimal RenderBackend for CPU-only benchmarks
// ================================================================================================

struct NullBackend {
    info: BackendInfo,
}

impl NullBackend {
    fn new() -> Self {
        Self {
            info: BackendInfo {
                name: "NullBackend",
                version: String::new(),
                vendor: String::new(),
                renderer: String::new(),
                capabilities: BackendCapabilities::default(),
            },
        }
    }
}

impl RenderBackend for NullBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }
}

impl FrameOps for NullBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }
    fn end_frame(&mut self) -> GoudResult<()> {
        Ok(())
    }
}

impl ClearOps for NullBackend {
    fn set_clear_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) {}
    fn clear_color(&mut self) {}
    fn clear_depth(&mut self) {}
}

impl StateOps for NullBackend {
    fn set_viewport(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {}
    fn enable_depth_test(&mut self) {}
    fn disable_depth_test(&mut self) {}
    fn enable_blending(&mut self) {}
    fn disable_blending(&mut self) {}
    fn set_blend_func(&mut self, _src: BlendFactor, _dst: BlendFactor) {}
    fn enable_culling(&mut self) {}
    fn disable_culling(&mut self) {}
    fn set_cull_face(&mut self, _face: CullFace) {}
    fn set_depth_func(&mut self, _func: goud_engine::libs::graphics::backend::types::DepthFunc) {}
    fn set_front_face(&mut self, _face: goud_engine::libs::graphics::backend::types::FrontFace) {}
    fn set_depth_mask(&mut self, _enabled: bool) {}
    fn set_line_width(&mut self, _width: f32) {}
}

impl BufferOps for NullBackend {
    fn create_buffer(
        &mut self,
        _buffer_type: BufferType,
        _usage: BufferUsage,
        _data: &[u8],
    ) -> GoudResult<BufferHandle> {
        Ok(BufferHandle::new(0, 1))
    }
    fn update_buffer(
        &mut self,
        _handle: BufferHandle,
        _offset: usize,
        _data: &[u8],
    ) -> GoudResult<()> {
        Ok(())
    }
    fn destroy_buffer(&mut self, _handle: BufferHandle) -> bool {
        true
    }
    fn is_buffer_valid(&self, _handle: BufferHandle) -> bool {
        true
    }
    fn buffer_size(&self, _handle: BufferHandle) -> Option<usize> {
        Some(0)
    }
    fn bind_buffer(&mut self, _handle: BufferHandle) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_buffer(&mut self, _buffer_type: BufferType) {}
}

impl TextureOps for NullBackend {
    fn create_texture(
        &mut self,
        _width: u32,
        _height: u32,
        _format: TextureFormat,
        _filter: TextureFilter,
        _wrap: TextureWrap,
        _data: &[u8],
    ) -> GoudResult<TextureHandle> {
        Ok(TextureHandle::new(0, 1))
    }
    fn update_texture(
        &mut self,
        _handle: TextureHandle,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
        _data: &[u8],
    ) -> GoudResult<()> {
        Ok(())
    }
    fn destroy_texture(&mut self, _handle: TextureHandle) -> bool {
        true
    }
    fn is_texture_valid(&self, _handle: TextureHandle) -> bool {
        true
    }
    fn texture_size(&self, _handle: TextureHandle) -> Option<(u32, u32)> {
        Some((64, 64))
    }
    fn bind_texture(&mut self, _handle: TextureHandle, _unit: u32) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_texture(&mut self, _unit: u32) {}
}

impl RenderTargetOps for NullBackend {
    fn create_render_target(&mut self, _desc: &RenderTargetDesc) -> GoudResult<RenderTargetHandle> {
        Ok(RenderTargetHandle::new(0, 1))
    }
    fn destroy_render_target(&mut self, _handle: RenderTargetHandle) -> bool {
        true
    }
    fn is_render_target_valid(&self, _handle: RenderTargetHandle) -> bool {
        true
    }
    fn bind_render_target(&mut self, _handle: Option<RenderTargetHandle>) -> GoudResult<()> {
        Ok(())
    }
    fn render_target_texture(&self, _handle: RenderTargetHandle) -> Option<TextureHandle> {
        Some(TextureHandle::new(0, 1))
    }
}

impl ShaderOps for NullBackend {
    fn create_shader(
        &mut self,
        _vertex_src: &str,
        _fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        Ok(ShaderHandle::new(0, 1))
    }
    fn destroy_shader(&mut self, _handle: ShaderHandle) -> bool {
        true
    }
    fn is_shader_valid(&self, _handle: ShaderHandle) -> bool {
        true
    }
    fn bind_shader(&mut self, _handle: ShaderHandle) -> GoudResult<()> {
        Ok(())
    }
    fn unbind_shader(&mut self) {}
    fn get_uniform_location(&self, _handle: ShaderHandle, _name: &str) -> Option<i32> {
        Some(0)
    }
    fn set_uniform_int(&mut self, _location: i32, _value: i32) {}
    fn set_uniform_float(&mut self, _location: i32, _value: f32) {}
    fn set_uniform_vec2(&mut self, _location: i32, _x: f32, _y: f32) {}
    fn set_uniform_vec3(&mut self, _location: i32, _x: f32, _y: f32, _z: f32) {}
    fn set_uniform_vec4(&mut self, _location: i32, _x: f32, _y: f32, _z: f32, _w: f32) {}
    fn set_uniform_mat4(&mut self, _location: i32, _matrix: &[f32; 16]) {}
}

impl DrawOps for NullBackend {
    fn set_vertex_attributes(&mut self, _layout: &VertexLayout) {}
    fn draw_arrays(
        &mut self,
        _topology: PrimitiveTopology,
        _first: u32,
        _count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }
    fn draw_indexed(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
    ) -> GoudResult<()> {
        Ok(())
    }
    fn draw_indexed_u16(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
    ) -> GoudResult<()> {
        Ok(())
    }
    fn draw_arrays_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        _first: u32,
        _count: u32,
        _instance_count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }
    fn draw_indexed_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        _count: u32,
        _offset: usize,
        _instance_count: u32,
    ) -> GoudResult<()> {
        Ok(())
    }
}

// ================================================================================================
// Render Metrics
// ================================================================================================

/// Captured metrics from a single render frame for baseline comparison.
#[derive(Debug, Clone, Serialize)]
struct RenderFrameMetrics {
    entity_count: usize,
    sprite_count: usize,
    draw_calls: usize,
    batch_count: usize,
    batch_ratio: f32,
    culled_count: usize,
    vertex_count: usize,
    vertex_buffer_bytes: usize,
}

// ================================================================================================
// Helpers
// ================================================================================================

const TEXTURE_COUNT: usize = 8;
const SCENARIO_SIZES: [usize; 4] = [0, 100, 1_000, 10_000];
const SCALING_SIZES: [usize; 3] = [500, 2_000, 5_000];

fn create_render_world(n: usize) -> World {
    let mut world = World::new();
    for i in 0..n {
        let entity = world.spawn_empty();
        let texture: AssetHandle<TextureAsset> = AssetHandle::new((i % TEXTURE_COUNT) as u32, 1);
        world.insert(entity, Sprite::new(texture));
        world.insert(
            entity,
            Transform2D::from_position(Vec2::new(i as f32 * 10.0, (i % 100) as f32)),
        );
    }
    world
}

fn create_render_batch() -> SpriteBatch<NullBackend> {
    SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap()
}

/// Count draw calls by counting texture transitions in a sorted sprite list.
fn count_draw_calls(sprites: &[SpriteInstance]) -> usize {
    if sprites.is_empty() {
        return 0;
    }
    let mut calls = 1;
    for window in sprites.windows(2) {
        if window[0].texture != window[1].texture {
            calls += 1;
        }
    }
    calls
}

/// Run a full CPU frame and extract metrics.
fn capture_frame_metrics(n: usize) -> RenderFrameMetrics {
    let world = create_render_world(n);
    let mut batch = create_render_batch();
    let mut asset_server = AssetServer::new();

    batch.begin();
    batch.gather_sprites(&world, &mut asset_server).unwrap();
    batch.sort_sprites();

    let draw_calls = count_draw_calls(&batch.sprites);
    let texture_size = Vec2::new(64.0, 64.0);
    for sprite in batch.sprites.clone().iter() {
        batch
            .generate_sprite_vertices(sprite, texture_size)
            .unwrap();
    }

    let vertex_count = batch.vertices.len();
    let vertex_buffer_bytes =
        vertex_count * std::mem::size_of::<goud_engine::rendering::sprite_batch::SpriteVertex>();

    RenderFrameMetrics {
        entity_count: n,
        sprite_count: batch.sprite_count(),
        draw_calls,
        batch_count: batch.batch_count().max(draw_calls),
        batch_ratio: if draw_calls > 0 {
            batch.sprite_count() as f32 / draw_calls as f32
        } else {
            0.0
        },
        culled_count: batch.culled_count(),
        vertex_count,
        vertex_buffer_bytes,
    }
}

// ================================================================================================
// Benchmark Group 1: Full CPU Frame Pipeline
// ================================================================================================

fn bench_render_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_full_frame");

    for &size in &SCENARIO_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let texture_size = Vec2::new(64.0, 64.0);
                let sprites_snapshot: Vec<SpriteInstance> = batch.sprites.clone();
                for sprite in &sprites_snapshot {
                    batch
                        .generate_sprite_vertices(sprite, texture_size)
                        .unwrap();
                }
                black_box((batch.sprite_count(), batch.vertices.len()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 2: Metrics Capture
// ================================================================================================

fn bench_render_metrics_capture(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_metrics_capture");

    for &size in &SCENARIO_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let dc = count_draw_calls(&batch.sprites);
                black_box((batch.sprite_count(), dc, batch.culled_count()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 3: Frame Percentiles (iter_custom for P95/P99)
// ================================================================================================

fn bench_render_frame_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_frame_percentiles");

    for &size in &SCENARIO_SIZES {
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter_custom(|iters| {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..iters {
                    let start = Instant::now();
                    batch.begin();
                    batch.gather_sprites(&world, &mut asset_server).unwrap();
                    batch.sort_sprites();
                    let sprites_snapshot: Vec<SpriteInstance> = batch.sprites.clone();
                    let texture_size = Vec2::new(64.0, 64.0);
                    for sprite in &sprites_snapshot {
                        batch
                            .generate_sprite_vertices(sprite, texture_size)
                            .unwrap();
                    }
                    black_box(batch.sprite_count());
                    total += start.elapsed();
                }
                total
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Benchmark Group 4: Scaling Curve
// ================================================================================================

fn bench_render_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_scaling");

    for &size in &SCALING_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_render_world(size);
            let mut batch = create_render_batch();
            let mut asset_server = AssetServer::new();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world, &mut asset_server).unwrap();
                batch.sort_sprites();
                let texture_size = Vec2::new(64.0, 64.0);
                let sprites_snapshot: Vec<SpriteInstance> = batch.sprites.clone();
                for sprite in &sprites_snapshot {
                    batch
                        .generate_sprite_vertices(sprite, texture_size)
                        .unwrap();
                }
                black_box((batch.sprite_count(), batch.vertices.len()));
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(full_frame_benches, bench_render_full_frame,);

criterion_group!(metrics_benches, bench_render_metrics_capture,);

criterion_group!(percentile_benches, bench_render_frame_percentiles,);

criterion_group!(scaling_benches, bench_render_scaling,);

// ================================================================================================
// Custom Main — supports RENDER_BASELINE=1 for JSON baseline generation
// ================================================================================================

fn generate_baseline() {
    println!("Generating render baseline...");

    let all_sizes: Vec<usize> = SCENARIO_SIZES
        .iter()
        .chain(SCALING_SIZES.iter())
        .copied()
        .collect();

    let metrics: Vec<RenderFrameMetrics> = all_sizes
        .iter()
        .map(|&size| capture_frame_metrics(size))
        .collect();

    #[derive(Serialize)]
    struct Baseline {
        generated_at: String,
        commit: String,
        scenarios: Vec<RenderFrameMetrics>,
    }

    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let baseline = Baseline {
        generated_at: format!("unix:{now}"),
        commit,
        scenarios: metrics,
    };

    let json = serde_json::to_string_pretty(&baseline).unwrap();

    let baselines_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/baselines");
    std::fs::create_dir_all(&baselines_dir).unwrap();
    let path = baselines_dir.join("render_baseline.json");
    std::fs::write(&path, &json).unwrap();

    println!("Baseline written to {}", path.display());
    println!("{json}");
}

fn main() {
    if std::env::var("RENDER_BASELINE").is_ok() {
        generate_baseline();
        return;
    }

    // Run criterion benchmarks
    full_frame_benches();
    metrics_benches();
    percentile_benches();
    scaling_benches();

    Criterion::default().configure_from_args().final_summary();
}
