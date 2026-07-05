---
globs:
  - "**/networking/**"
alwaysApply: false
---

# Networking Patterns

The networking module (`goud_engine/src/libs/networking/`) is Layer 2 (Libs). It holds transport-agnostic netcode built on top of the provider transport boundary. It MAY import from `core/` (and its own layer) only.

## Provider Transport Boundary

- The transport is abstracted behind the `NetworkProvider` trait, defined canonically in `core/providers/network.rs` and re-exported via `crate::libs::providers::network`. Concrete transports (UDP, TCP, WebSocket, WebRTC, P2P mesh, null, simulation) live in `libs/providers/impls/`.
- `NetworkProvider` is object-safe and stored as a boxed trait object. It exposes connection lifecycle (`host`, `connect`, `disconnect`, `disconnect_all`), `send`/`broadcast` over typed `Channel`s, `drain_events`, and stats — nothing above it should name a concrete provider.
- The transport moves **raw bytes only**. It never inspects, frames, or serializes payloads; serialization is the caller's responsibility. Keep it that way.
- `drain_events` returns an owned `Vec<NetworkEvent>` and MUST be called once per frame. It clears the internal buffer so the caller can process events without holding a borrow on the provider.
- `ConnectionId` values are meaningful only within the provider instance that issued them. Never cache a connection ID and use it against a different provider or engine context. The same applies to stats — resolve the active provider handle first.

## Higher-Level Netcode

- `rpc.rs` (`RpcFramework`) sits above the transport. It does **not** own a provider: `call()` and `process_incoming()` produce `OutgoingRpcMessage` values that the caller sends over Channel 0 (reliable-ordered), and inbound bytes are fed back in. Keep this transport-independence.
- RPC wire format is fixed: `[2 bytes rpc_id][8 bytes call_id][1 byte msg_type][payload]`, where `msg_type` is 0 = call, 1 = success response, 2 = error response. RPCs are registered with a direction (`ServerToClient`, `ClientToServer`, `Bidirectional`).
- `rollback.rs` implements GGPO-style rollback: local input prediction, ring-buffer state snapshots, mismatch-triggered resimulation, and hash-based desync detection. Game state participates by implementing the `GameState` trait (`advance` for one deterministic tick, `state_hash` for desync checks).
- Rollback correctness depends on determinism: `advance` must be pure over its inputs, and `state_hash` must be stable across peers. Do not introduce nondeterminism (unordered iteration, wall-clock reads, floating-point drift) into either.
