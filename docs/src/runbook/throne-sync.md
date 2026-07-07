# Throne Adoption & Sync Points

Throne (the C# colony-sim at `aram-devdocs/throne_ge`) is the flagship consumer and the forcing function for the v2 perf targets. Because the engine breaks its API where necessary (no legacy code kept), Throne adopts each breaking change at defined sync points. Adoption work lives in `throne_ge` under the `goudengine-v2-adoption` milestone (label `goudengine-adoption`), not in this repo.

Every GoudEngine `breaking-change` issue names its Throne follow-up before it closes. This is the reverse of `throne_ge/docs/runbook/engine-dependencies.md`.

## Sync points

| Sync | After | Throne issues | What Throne does |
|---|---|---|---|
| **Sync 0** | Phase 0 | THR-S0-01 | Bump GoudEngine; delete the `CopyGoudEngineNativeLib` workaround; re-profile GameWorld with the fixed phase counters (this becomes Phase 2's "before" evidence). |
| **Sync A** | Phase 4 gate | THR-A-01 … THR-A-04 | Bump to the v2 FFI surface; adopt batch spawn + bulk transform upload + command-buffer submission (delete per-entity `SetModelPosition`/`SetModelRotation` loops); replace the 80×80 terrain window with the engine chunked tilemap-grid primitive (full 200×200 map); re-enable shadows. |
| **Sync B** | Phase 5 gate | THR-B-01 | Replace the `Parallel.For`/job stubs with the engine job system; wire `goud_world_state_hash` into `DeterminismTests`; re-run Throne Phase 1 scale gates against engine primitives. |
| **Sync C** | Phase 8 gate | THR-C-01 | Evaluate engine A*/flow-field, event bus, and RNG/noise against Throne's C# implementations; adopt where they beat the stubs. |

## Sync A acceptance (the prime-directive validation)

Posted on both repos' tracking issues: Throne GameWorld, full 200×200 map, **no terrain window, shadows ON** → **≥ 120 FPS on Apple M-series**, BeginFrame+EndFrame ≤ 4 ms, and a grep gate showing zero per-entity transform-push loops.

## Provenance

THR-A-01 supersedes GoudEngine #596 and carries the content of the migrated GoudEngine issues #590–#593 (V2-P07…P11). Those were closed on the engine side with migration links. The engine-side prerequisites for Sync A are ENG2 Phase 2 (render core) + Phase 4 (FFI v2).
