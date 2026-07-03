# Performance Roadmap

This is the ordered plan for the renderer and ECS optimization work that follows
the cleanup and measurement groundwork. Each item is measurable against the
benches added under `goud_engine/benches/` and the checked-in baseline
(`benches/baselines/criterion_baseline.json`); run `scripts/bench-gate.py` to
compare.

## Measurement note

Field frame numbers from the consumer game (issues #677/#678) were captured
against a NuGet package that omitted the MSBuild targets file, which inflated
every frame ~60x on its own. That packaging defect is fixed, so re-measure the
absolute millisecond figures on a corrected package or a local project reference
before treating them as ground truth. The mechanisms below are code-verified; the
magnitudes are not.

## Ordered work

The consumer's own profile shows simulation is cheap and GPU-submission dominates,
so the ordering is by consumer impact.

1. **Auto-instance identical primitives + primitive batch FFI** (#679). `CreatePlane`
   and friends never instance; the `instancing_enabled` config flag is dead. Wire it
   (or add `InstantiatePlane` / `create_planes_batch`) to collapse identical
   geometry+material primitives onto the existing `create_instanced_primitive` path.
   Target: 40k identical planes collapse to a few dozen draws.
2. **Coalesce and right-size per-draw uniform uploads** (#677/#678). Every draw
   command uploads a full 4 KB uniform block; the shadow pass doubles it. Right-size
   the slot to the reflected block extent and batch the writes into one per shader per
   pass. Highest single frame-time win. Verify with the count-pinning test and the
   `uniform_layout` / `shadow_record` benches.
3. **Engine-owned frustum culling via a spatial index** (#678). Frustum culling is a
   linear scan of every object each frame. Add a grid/BVH so culling is O(visible),
   and feed a visible set back to consumers.
4. **Cache per-object model matrices behind a dirty flag**. `create_model_matrix` (a
   trig-heavy TRS build) runs per object per frame; skinned meshes already cache. Add
   the same dirty-flagged cache to `Object3D`. Measured by `frame_scan/dynamic_*`.
5. **Reuse render-loop scratch buffers and drop the sort clone**. The visibility loop
   allocates fresh Vecs each frame and the material sort clones every draw record.
   Reuse persistent scratch and iterate sorted indices. Measured by `material_sort`.
6. **Shadow pass: single static-batch draw + caster caching** (#677). The shadow
   pre-pass draws every static object individually and re-records every frame. Draw
   the world-space static batch once and cache the shadow map for static scenes.
7. **Change-tick-gated transform propagation**. The propagation system recomputes
   every frame; the change-tick machinery exists but is unused. Gate the whole system
   on whether any relevant storage changed. Measured by `engine_tick`.
8. **Terrain / tilemap chunk streaming**. Consumers hand-roll a tile window that
   destroys and recreates primitives on recenter. Provide engine-owned chunked static
   meshes with camera-bound residency.

## Infrastructure follow-ups

- Populate real bench baselines on the CI runner and enable `bench-gate.py` as a
  blocking check (#256).
- FFI fuzzing (#267) and a valgrind/miri leak lane (#273) on top of the storage fix
  already landed.
- A golden-image / scene-graph parity harness in the headless-GPU container.
