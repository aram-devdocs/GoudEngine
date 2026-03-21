//! FFI Component Query Benchmarks
//!
//! Benchmarks for the FFI-layer component query operations that C# SDK uses.
//! Measures the cost of `goud_component_count` and `goud_component_get_all`.
//!
//! Run with: `cargo bench --bench ffi_component_benchmarks`
//!
//! ## Performance Targets
//!
//! - 10k entity iteration (get_all + read): <1ms

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goud_engine::ffi::{
    component::{
        goud_component_add, goud_component_count, goud_component_get_all,
        goud_component_register_type,
    },
    context::goud_context_create,
    entity::goud_entity_spawn_empty,
    GoudEntityId,
};

/// Benchmarks for FFI-layer component iteration (the path C# SDK uses).
/// Target: 10k entity iteration <1ms.
fn bench_ffi_component_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_component_query");

    const FFI_TYPE_ID: u64 = 0xBE_0000_0001;
    const COMPONENT_SIZE: usize = std::mem::size_of::<[f32; 2]>();
    const COMPONENT_ALIGN: usize = std::mem::align_of::<[f32; 2]>();

    // Register the component type once.
    let name = b"BenchComponent";
    unsafe {
        goud_component_register_type(
            FFI_TYPE_ID,
            name.as_ptr(),
            name.len(),
            COMPONENT_SIZE,
            COMPONENT_ALIGN,
        );
    }

    for count in [1_000u32, 10_000, 100_000] {
        // Setup: create context and populate with entities + components.
        let ctx = goud_context_create();
        let mut entity_ids = Vec::with_capacity(count as usize);
        for i in 0..count {
            let bits = goud_entity_spawn_empty(ctx);
            let data: [f32; 2] = [i as f32, (i as f32) * 2.0];
            unsafe {
                goud_component_add(
                    ctx,
                    GoudEntityId::new(bits),
                    FFI_TYPE_ID,
                    data.as_ptr() as *const u8,
                    COMPONENT_SIZE,
                );
            }
            entity_ids.push(bits);
        }

        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(BenchmarkId::new("count", count), &count, |b, _| {
            b.iter(|| {
                black_box(goud_component_count(ctx, FFI_TYPE_ID));
            });
        });

        group.bench_with_input(BenchmarkId::new("get_all", count), &count, |b, &n| {
            let mut ents = vec![0u64; n as usize];
            let mut ptrs: Vec<*const u8> = vec![std::ptr::null(); n as usize];
            b.iter(|| {
                let returned = unsafe {
                    goud_component_get_all(
                        ctx,
                        FFI_TYPE_ID,
                        ents.as_mut_ptr(),
                        ptrs.as_mut_ptr(),
                        n,
                    )
                };
                // Simulate reading each component (what C# foreach would do).
                let mut sum = 0.0f32;
                for i in 0..returned as usize {
                    let ptr = ptrs[i];
                    // SAFETY: ptr points to valid [f32; 2] data in component storage.
                    let val = unsafe { *(ptr as *const [f32; 2]) };
                    sum += val[0];
                }
                black_box(sum);
            });
        });
    }

    group.finish();
}

criterion_group!(ffi_query_benches, bench_ffi_component_query,);

criterion_main!(ffi_query_benches,);
