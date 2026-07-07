# Render Core v2 Rules (ENG2 Phases 2 & 6)

Applies to `**/graphics/**` and the renderer paths. These invariants supersede the performance/submission guidance in `graphics-patterns.md` (the backend-isolation and Renderer-trait rules there still hold). Full design: `docs/src/runbook/phases/phase-2.md` and RFC-0005.

## Submission model

- **O(visible), never O(total).** No per-frame iteration over the entire scene-object set. Cull against a spatial index (grid/BVH) *before* any transform/uniform/encode work. A culled object must pay zero per-frame cost.
- **Dense storage, not hash-map iteration.** Scene objects live in slot arrays with generational handles, iterated contiguously — not a per-frame walk of a `FxHashMap`.
- **Instancing by identity.** Draws are grouped by `(mesh id, material id)` and submitted instanced by default — including `CreatePlane`/`CreateCube`/`CreateSphere`/`CreateCylinder` primitives, not only `InstantiateModel`. A config flag that claims to enable instancing must actually drive the instanced path or not exist.

## Uniforms & buffers

- **Per-frame constants uploaded once** (camera, lights, fog) into one bind group. **Per-object data goes in dynamic-offset slots** — never copy a full multi-KB uniform block per draw, never issue one `write_buffer` per draw. Coalesce writes into one upload per frame.
- **Persistent, dirty-tracked instance/transform buffers.** Upload only changed ranges. Never rebuild the whole buffer, and never CPU-pre-transform vertices into a static VBO that must be re-baked on any change.
- Do not clone per-instance CPU vertex data for instances that share a mesh.

## Data seam

- Transforms are a **structure-of-arrays column shared between ECS and the renderer** (ENG2-P2-13). The renderer consumes dirty ranges from that column; it does not receive per-entity transforms across FFI one at a time.

## Every render/UI/particle change

- Add or update a **story** in the scene gallery (ENG2-P0-14) with a golden image and metric budget — see `[testing-v2]`.
- Meet the perf definition of done — see `[perf-work]` and `docs/src/runbook/perf-dod.md`.
- Raw GPU calls stay in `libs/graphics/backend/` (unchanged boundary).
