//! Sprite Batch Performance Benchmarks
//!
//! Benchmarks for the sprite batch CPU pipeline including:
//! - Sprite gathering from ECS world
//! - Sprite sorting (Z-layer + texture batching)
//! - Vertex generation per sprite
//! - Full CPU pipeline (gather + sort)
//!
//! Run with: `cargo bench --bench sprite_batch_benchmarks`
//!
//! ## Performance Targets
//!
//! - gather: <5us per entity
//! - sort: <100ns per entity

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goud_engine::assets::loaders::TextureAsset;
use goud_engine::assets::AssetHandle;
use goud_engine::core::error::GoudResult;
use goud_engine::core::math::Vec2;
use goud_engine::ecs::components::{Mat3x3, Sprite, Transform2D};
use goud_engine::ecs::World;
use goud_engine::libs::graphics::backend::capabilities::{BackendCapabilities, BackendInfo};
use goud_engine::libs::graphics::backend::types::{
    BufferHandle, BufferType, BufferUsage, PrimitiveTopology, ShaderHandle, TextureFilter,
    TextureFormat, TextureHandle, TextureWrap, VertexLayout,
};
use goud_engine::libs::graphics::backend::{
    BlendFactor, BufferOps, ClearOps, CullFace, DrawOps, FrameOps, RenderBackend, ShaderOps,
    StateOps, TextureOps,
};
use goud_engine::rendering::sprite_batch::{SpriteBatch, SpriteBatchConfig, SpriteInstance};
use std::hint::black_box;

// ================================================================================================
// NullBackend — minimal RenderBackend for CPU-only benchmarks
// ================================================================================================

/// A no-op render backend for benchmarking CPU-side sprite batch operations.
///
/// All GPU operations return dummy/default values. No actual GPU calls are made.
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
// Helpers
// ================================================================================================

/// Creates a world populated with N entities each having Sprite + Transform2D.
/// Uses a small set of texture handles to simulate realistic batching scenarios.
fn create_sprite_world(n: usize) -> World {
    let mut world = World::new();
    let texture_count = 8;

    for i in 0..n {
        let entity = world.spawn_empty();
        let texture: AssetHandle<TextureAsset> = AssetHandle::new((i % texture_count) as u32, 1);
        world.insert(entity, Sprite::new(texture));
        world.insert(
            entity,
            Transform2D::from_position(Vec2::new(i as f32 * 10.0, (i % 100) as f32)),
        );
    }

    world
}

/// Creates a Vec of SpriteInstance values for sort/vertex benchmarks,
/// bypassing the ECS gather step.
fn create_sprite_instances(n: usize) -> Vec<SpriteInstance> {
    let texture_count = 8;

    (0..n)
        .map(|i| {
            let texture: AssetHandle<TextureAsset> =
                AssetHandle::new((i % texture_count) as u32, 1);
            let sprite = Sprite::new(texture);
            let z_layer = (i % 100) as f32;
            let size = Vec2::new(64.0, 64.0);

            SpriteInstance::from_components(
                goud_engine::ecs::Entity::new(i as u32, 1),
                &sprite,
                Mat3x3::IDENTITY,
                z_layer,
                size,
            )
        })
        .collect()
}

fn create_batch() -> SpriteBatch<NullBackend> {
    SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap()
}

// ================================================================================================
// Gather Benchmarks
// ================================================================================================

fn bench_gather_sprites(c: &mut Criterion) {
    let mut group = c.benchmark_group("sprite_batch_gather");

    for size in [100, 1_000, 5_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_sprite_world(size);
            let mut batch = create_batch();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world).unwrap();
                black_box(batch.sprite_count());
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Sort Benchmarks
// ================================================================================================

fn bench_sort_sprites(c: &mut Criterion) {
    let mut group = c.benchmark_group("sprite_batch_sort");

    for size in [100, 1_000, 5_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("sprites", size), &size, |b, &size| {
            let instances = create_sprite_instances(size);

            b.iter_batched(
                || {
                    let mut batch = create_batch();
                    batch.sprites = instances.clone();
                    batch
                },
                |mut batch| {
                    batch.sort_sprites();
                    black_box(batch.sprite_count());
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ================================================================================================
// Vertex Generation Benchmarks
// ================================================================================================

fn bench_vertex_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sprite_batch_vertex_gen");

    for size in [100, 1_000, 5_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("sprites", size), &size, |b, &size| {
            let instances = create_sprite_instances(size);
            let texture_size = Vec2::new(64.0, 64.0);

            b.iter(|| {
                let mut batch = create_batch();
                for sprite in &instances {
                    batch
                        .generate_sprite_vertices(sprite, texture_size)
                        .unwrap();
                }
                black_box(batch.vertices.len());
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Full CPU Pipeline Benchmarks
// ================================================================================================

fn bench_full_cpu_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("sprite_batch_full_cpu");

    for size in [100, 1_000, 5_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("entities", size), &size, |b, &size| {
            let world = create_sprite_world(size);
            let mut batch = create_batch();

            b.iter(|| {
                batch.begin();
                batch.gather_sprites(&world).unwrap();
                batch.sort_sprites();
                black_box(batch.sprite_count());
            });
        });
    }

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(gather_benches, bench_gather_sprites,);

criterion_group!(sort_benches, bench_sort_sprites,);

criterion_group!(vertex_benches, bench_vertex_generation,);

criterion_group!(pipeline_benches, bench_full_cpu_pipeline,);

criterion_main!(
    gather_benches,
    sort_benches,
    vertex_benches,
    pipeline_benches,
);
