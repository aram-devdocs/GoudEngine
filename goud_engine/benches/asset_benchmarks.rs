//! Asset System Performance Benchmarks
//!
//! Comprehensive benchmarks for asset loading and management including:
//! - Asset loading performance
//! - Handle allocation/deallocation
//! - Asset storage operations
//! - Hot reload performance
//! - Batch loading performance
//!
//! Run with: `cargo bench --bench asset_benchmarks`
//!
//! ## Performance Targets
//!
//! - Handle allocation: <50ns per handle
//! - Asset insertion: <100ns per asset
//! - Asset lookup: <10ns per lookup
//! - Texture loading: <10ms for 1024x1024 RGBA
//! - Batch load (100 assets): <100ms total

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goud_engine::assets::*;

// ================================================================================================
// Test Asset Types
// ================================================================================================

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct DummyAsset {
    data: Vec<u8>,
}

impl Asset for DummyAsset {
    fn asset_type_name() -> &'static str {
        "DummyAsset"
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct SmallAsset {
    value: u32,
}

impl Asset for SmallAsset {}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct LargeAsset {
    data: Vec<u8>, // 1MB
}

impl Asset for LargeAsset {}

// Note: Handle allocation benchmarks omitted (internal API)
// TODO: Add public API benchmarks for AssetHandle operations

// ================================================================================================
// Asset Storage Benchmarks
// ================================================================================================

fn bench_asset_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_storage");

    // Insert small asset
    group.bench_function("insert_small", |b| {
        let mut storage = AssetStorage::new();

        b.iter(|| {
            let handle = storage.insert(SmallAsset { value: 42 });
            black_box(handle);
        });
    });

    // Insert large asset (1MB)
    group.bench_function("insert_large", |b| {
        let mut storage = AssetStorage::new();

        b.iter(|| {
            let handle = storage.insert(LargeAsset {
                data: vec![0u8; 1024 * 1024],
            });
            black_box(handle);
        });
    });

    // Get asset
    group.bench_function("get", |b| {
        let mut storage = AssetStorage::new();
        let handle = storage.insert(SmallAsset { value: 42 });

        b.iter(|| {
            let asset = storage.get(&handle);
            black_box(asset);
        });
    });

    // Get mutable asset
    group.bench_function("get_mut", |b| {
        let mut storage = AssetStorage::new();
        let handle = storage.insert(SmallAsset { value: 42 });

        b.iter(|| {
            let asset = storage.get_mut(&handle);
            black_box(asset);
        });
    });

    // Remove asset
    group.bench_function("remove", |b| {
        b.iter_batched(
            || {
                let mut storage = AssetStorage::new();
                let handle = storage.insert(SmallAsset { value: 42 });
                (storage, handle)
            },
            |(mut storage, handle)| {
                let asset = storage.remove(&handle);
                black_box(asset);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Batch insert
    for size in [100, 1_000, 10_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("batch_insert", size), &size, |b, &size| {
            b.iter(|| {
                let mut storage = AssetStorage::new();
                for i in 0..size {
                    let handle = storage.insert(SmallAsset { value: i as u32 });
                    black_box(handle);
                }
            });
        });
    }

    group.finish();
}

// Note: Path-based benchmarks omitted for now
// TODO: Add after AssetPath API stabilizes

// ================================================================================================
// Asset Loader Benchmarks
// ================================================================================================

#[derive(Clone, Default)]
struct BenchmarkLoader;

impl AssetLoader for BenchmarkLoader {
    type Asset = DummyAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["bench"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        Ok(DummyAsset {
            data: bytes.to_vec(),
        })
    }
}

fn bench_asset_loader(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_loader");

    // Load small asset (1KB)
    group.bench_function("load_1kb", |b| {
        let loader = BenchmarkLoader;
        let data = vec![0u8; 1024];

        b.iter(|| {
            let mut context = LoadContext::new("test.bench".into());
            let result = loader.load(&data, &(), &mut context);
            let _ = black_box(result);
        });
    });

    // Load medium asset (100KB)
    group.bench_function("load_100kb", |b| {
        let loader = BenchmarkLoader;
        let data = vec![0u8; 100 * 1024];

        b.iter(|| {
            let mut context = LoadContext::new("test.bench".into());
            let result = loader.load(&data, &(), &mut context);
            let _ = black_box(result);
        });
    });

    // Load large asset (1MB)
    group.bench_function("load_1mb", |b| {
        let loader = BenchmarkLoader;
        let data = vec![0u8; 1024 * 1024];

        b.iter(|| {
            let mut context = LoadContext::new("test.bench".into());
            let result = loader.load(&data, &(), &mut context);
            let _ = black_box(result);
        });
    });

    group.finish();
}

// ================================================================================================
// AssetServer Benchmarks
// ================================================================================================

fn bench_asset_server(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_server");

    // Handle allocation overhead
    group.bench_function("reserve_handle", |b| {
        let mut server = AssetServer::new();
        server.register_loader(BenchmarkLoader);

        b.iter(|| {
            // Reserve handle without loading
            let handle = server.load::<DummyAsset>("nonexistent.bench");
            black_box(handle);
        });
    });

    group.finish();
}

// ================================================================================================
// Criterion Configuration
// ================================================================================================

criterion_group!(storage_benches, bench_asset_storage_operations,);

criterion_group!(loader_benches, bench_asset_loader, bench_asset_server,);

criterion_main!(storage_benches, loader_benches,);
