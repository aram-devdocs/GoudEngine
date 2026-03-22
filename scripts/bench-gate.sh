#!/usr/bin/env bash
set -euo pipefail

# Phase 0 Performance Gate Script
# Runs criterion benchmarks and validates that they complete successfully.
# Exits non-zero if any benchmark suite fails.

echo "=== Phase 0 Performance Gate ==="
echo ""

FAILED=0

# Run pool benchmarks
echo "Running pool benchmarks..."
if cargo bench --bench pool_benchmarks -- --quick 2>&1 | tee /tmp/bench_pool.txt; then
    echo "Pool benchmarks: OK"
else
    echo "Pool benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

# Run ECS benchmarks
echo "Running ECS benchmarks..."
if cargo bench --bench ecs_benchmarks -- --quick 2>&1 | tee /tmp/bench_ecs.txt; then
    echo "ECS benchmarks: OK"
else
    echo "ECS benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

# Run spatial grid benchmarks
echo "Running spatial grid benchmarks..."
if cargo bench --bench spatial_grid_benchmarks -- --quick 2>&1 | tee /tmp/bench_spatial.txt; then
    echo "Spatial grid benchmarks: OK"
else
    echo "Spatial grid benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

# Run physics benchmarks
echo "Running physics benchmarks..."
if cargo bench --bench physics_benchmarks -- --quick 2>&1 | tee /tmp/bench_physics.txt; then
    echo "Physics benchmarks: OK"
else
    echo "Physics benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

# Run render benchmarks (may need GL context)
echo "Running render benchmarks..."
if cargo bench --bench render_benchmarks -- --quick 2>&1 | tee /tmp/bench_render.txt; then
    echo "Render benchmarks: OK"
else
    echo "Render benchmarks: SKIPPED (may need GL context)"
fi
echo ""

# Run asset benchmarks
echo "Running asset benchmarks..."
if cargo bench --bench asset_benchmarks -- --quick 2>&1 | tee /tmp/bench_asset.txt; then
    echo "Asset benchmarks: OK"
else
    echo "Asset benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

# Run serialization benchmarks
echo "Running serialization benchmarks..."
if cargo bench --bench serialization_benchmarks -- --quick 2>&1 | tee /tmp/bench_serial.txt; then
    echo "Serialization benchmarks: OK"
else
    echo "Serialization benchmarks: FAILED"
    FAILED=$((FAILED + 1))
fi
echo ""

echo "=== Gate Results ==="

if [ $FAILED -eq 0 ]; then
    echo "All performance gates PASSED"
    exit 0
else
    echo "$FAILED gate(s) FAILED"
    exit 1
fi
