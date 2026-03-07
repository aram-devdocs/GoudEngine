---
rfc: "0002"
title: "NetworkProvider Trait Design"
status: accepted
created: 2026-03-06
authors: ["aram-devdocs"]
tracking-issue: "#356"
---

# RFC-0002: NetworkProvider Trait Design

## 1. Summary

This RFC defines `NetworkProvider`, a new subsystem trait following the provider pattern established in RFC-0001. It abstracts transport backends (UDP for desktop, WebSocket for web) behind a unified interface covering connection lifecycle, message passing over typed channels, and event polling. `NetworkProvider` extends both `Provider` and `ProviderLifecycle` supertraits per RFC-0001 §3.3, and integrates with `ProviderRegistry` as an optional field, leaving games that do not need networking unaffected.

---

## 2. Motivation

GoudEngine has no networking subsystem. Parent issue #140 requires a full networking system; this RFC specifies the trait boundary before implementation begins.

RFC-0001 established the provider pattern for rendering, physics, audio, windowing, and input. `NetworkProvider` must follow that same pattern: a trait in `libs/providers/`, concrete implementations in `libs/providers/impls/network/`, registration in `ProviderRegistry` at Layer 2, and FFI exposure at Layer 3. Diverging from this pattern would fragment the architecture.

Multiplayer games target different transports by platform. A desktop build uses UDP (low latency, no browser restrictions); a web/WASM build uses WebSockets (only transport available in browsers). The game layer must not know which transport is active. `NetworkProvider` is the swap point.

Integration attaches to `GoudGame` in `goud_engine/src/sdk/game/instance.rs`, the same file RFC-0001 identified as the central integration point for all providers.

---

## 3. Design

### 3.1 NetworkProvider Trait

```rust
pub trait NetworkProvider: Provider + ProviderLifecycle {
    /// Begin accepting inbound connections on the given config.
    ///
    /// Calling `host` on an already-hosting provider returns an error.
    fn host(&mut self, config: &HostConfig) -> GoudResult<()>;

    /// Open a connection to the given address.
    ///
    /// Returns a `ConnectionId` that is valid until the connection closes.
    /// The connection may not be fully established when this returns; poll
    /// `drain_events` for `NetworkEvent::Connected`.
    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId>;

    /// Close a specific connection.
    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()>;

    /// Close all active connections.
    fn disconnect_all(&mut self) -> GoudResult<()>;

    /// Send raw bytes to one connection on the given channel.
    ///
    /// The provider does not inspect or frame the bytes. Serialization
    /// is the caller's responsibility.
    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()>;

    /// Send raw bytes to all active connections on the given channel.
    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()>;

    /// Return all buffered network events and clear the internal buffer.
    ///
    /// Must be called once per frame. Returns owned `Vec` to avoid holding
    /// a borrow on the provider while the caller processes events.
    fn drain_events(&mut self) -> Vec<NetworkEvent>;

    /// Return the current list of active connection IDs.
    fn connections(&self) -> &[ConnectionId];

    /// Return the state of a specific connection.
    fn connection_state(&self, conn: ConnectionId) -> ConnectionState;

    /// Return this peer's own `ConnectionId`, if the provider has been assigned one.
    fn local_id(&self) -> Option<ConnectionId>;

    /// Return static capability flags for this provider.
    fn network_capabilities(&self) -> &NetworkCapabilities;

    /// Return aggregate network statistics.
    fn stats(&self) -> NetworkStats;

    /// Return per-connection statistics, or `None` if the ID is unknown.
    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats>;
}
```

Design decisions:

- **Raw bytes, not typed messages.** Generic methods break object safety. Serialization (serde, bitcode, etc.) is a game-level concern; the provider passes bytes through.
- **Poll, not callbacks.** Callbacks are not object-safe and require `'static` closures that complicate borrow lifetimes. `drain_events` matches `PhysicsProvider::drain_collision_events` from RFC-0001.
- **`ConnectionId` not `PeerId`.** `ConnectionId` is a transport-level concept. Game-level peer identity (player IDs, lobby slots) belongs above the provider boundary.
- **No async.** The engine has no async runtime. Background I/O threads communicate with the main thread via channels; `drain_events` collects that work synchronously (see §3.5).
- **Owned `Vec` return accepted.** `drain_events` returns `Vec<NetworkEvent>` where `Received` variants contain `data: Vec<u8>`, causing per-message heap allocations. This mirrors `PhysicsProvider::drain_collision_events` from RFC-0001, which justified the pattern to avoid lifetime coupling. Network payloads are larger than collision events, but `drain_events` runs once per frame (not per-packet), and the `Vec` is short-lived. If profiling shows allocation pressure, a future optimization can reuse a scratch buffer inside the provider and return `&[NetworkEvent]` with a borrow — the trait can evolve without breaking the FFI boundary, which already uses caller-provided buffers (§3.8).

---

### 3.2 Supporting Types

```rust
/// Opaque transport-level connection identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

/// Lifecycle state of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

/// Named channel for message routing.
///
/// Channels map to transport QoS settings (reliable/unreliable, ordered/unordered).
/// Channel 0 is always reliable-ordered. Higher channels are provider-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel(pub u8);

/// An event produced by the network provider during `drain_events`.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Connected { conn: ConnectionId },
    Disconnected { conn: ConnectionId, reason: DisconnectReason },
    Received { conn: ConnectionId, channel: Channel, data: Vec<u8> },
    Error { conn: ConnectionId, message: String },
}

/// Why a connection closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    LocalClose,
    RemoteClose,
    Timeout,
    Error(String),
}

/// Configuration for hosting (accepting inbound connections).
#[derive(Debug, Clone)]
pub struct HostConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: u32,
}

/// Static capability flags for a network provider.
#[derive(Debug, Clone)]
pub struct NetworkCapabilities {
    pub supports_hosting: bool,
    pub max_connections: u32,
    pub max_channels: u8,
    pub max_message_size: usize,
}

/// Aggregate statistics for the provider.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
}

/// Per-connection statistics.
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub round_trip_ms: f32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_lost: u64,
}
```

---

### 3.3 Connection Lifecycle

```
Disconnected
     |
     | connect() / host() receives client
     v
 Connecting
     |              \
     | handshake ok  | handshake fail
     v               v
 Connected         Error
     |
     | disconnect() / remote close / timeout
     v
Disconnecting
     |
     v
Disconnected
```

`drain_events` emits `NetworkEvent::Connected` on the `Connecting -> Connected` transition and `NetworkEvent::Disconnected` on the `Disconnecting -> Disconnected` transition. `NetworkEvent::Error` does not automatically close the connection; call `disconnect` after receiving it.

---

### 3.4 Built-in Implementations

| Implementation | Feature Flag | Notes |
|---|---|---|
| `UdpNetProvider` | `net-udp` | UDP transport; desktop targets |
| `WebSocketNetProvider` | `net-ws` | WebSocket transport; web/WASM targets |
| `NullNetProvider` | always | No-op; for games that do not use networking |

`NullNetProvider` satisfies the optional field in `ProviderRegistry` without adding a dependency or panicking. `drain_events` returns an empty `Vec`; all other methods return `Ok(())` or type-appropriate defaults.

---

### 3.5 Thread Safety

`NetworkProvider` extends `Provider`, which requires `Send + Sync + 'static`. Concrete implementations use background I/O threads that communicate with the main thread via `std::sync::mpsc` channels. `drain_events` drains the receiver end of that channel each frame. No locking is required on the call sites.

`ProviderLifecycle::update(delta)` is called once per frame by the engine loop. For network providers, `update` flushes the outbound send queue and transfers inbound data from the I/O thread channel into the event buffer that `drain_events` returns. Providers that do not need per-frame work (e.g., `NullNetProvider`) implement `update` as a no-op returning `Ok(())`.

---

### 3.6 Layer Placement

| Component | Layer | Path |
|---|---|---|
| `NetworkProvider` trait + supporting types | Layer 1 | `goud_engine/src/libs/providers/network.rs` |
| Concrete implementations | Layer 1 | `goud_engine/src/libs/providers/impls/network/` |
| `ProviderRegistry` (gains `network` field) | Layer 2 | `goud_engine/src/core/providers/registry.rs` |
| FFI functions | Layer 3 | `goud_engine/src/ffi/network.rs` |
| SDK wrappers | Layer 4 | generated via codegen from `goud_sdk.schema.json` |

`ProviderRegistry` gains one optional field:

```rust
pub struct ProviderRegistry {
    pub render: Box<dyn RenderProvider>,
    pub physics: Box<dyn PhysicsProvider>,
    pub audio: Box<dyn AudioProvider>,
    pub input: Box<dyn InputProvider>,
    pub network: Option<Box<dyn NetworkProvider>>, // new
}
```

The field is `Option` because networking is not required — unlike render, physics, audio, and input, which every game needs at minimum as a `Null*Provider`. Most single-player games never touch networking. Making it `Option` avoids forcing a `NullNetProvider` allocation on every game and makes "networking not configured" a distinct, type-level state rather than a silent no-op. Call sites use `if let Some(net) = registry.network.as_mut()` (see §3.9). Open Question 1 tracks whether this should change.

The error type prerequisite from RFC-0001 §3.10 applies here too: `GoudError` must move to Layer 1 before `NetworkProvider` trait methods can return `GoudResult<T>` without a Layer 1 → Layer 2 import violation.

---

### 3.7 Error Handling

Network errors use the `ProviderError` variant established in RFC-0001 §3.9:

```rust
ProviderError { subsystem: "network", message: String }
```

Error codes 700–709 are reserved for the network subsystem, following the 10-codes-per-subsystem granularity established in RFC-0001 (600–609 render, 610–619 physics, etc.). The `error_code` match arm routes `ProviderError` by `subsystem` discriminator to the appropriate code range. The existing pattern in `goud_engine/src/core/error/types.rs` applies: the `message` carries a human-readable description; the code carries the machine-readable category.

---

### 3.8 FFI Boundary

SDK users select the transport at engine initialization. Per RFC-0001 §5, `goud_game_create` gains provider-type parameters; networking adds `GoudNetworkType` to that signature. Passing `GoudNetworkType::Null` leaves `ProviderRegistry.network` as `None`. SDK-level overloads may default to `Null` when the parameter is omitted.

Rust SDK users configure networking through the builder pattern established in RFC-0001 §3.5:

```rust
let game = GoudEngine::builder()
    .with_renderer(OpenGLRenderProvider::new(RenderConfig::default()))
    .with_network(UdpNetProvider::new(NetworkConfig::default()))
    .build()?;
```

```rust
#[repr(C)]
pub enum GoudNetworkType {
    Udp = 0,
    WebSocket = 1,
    Null = 99,
}
```

The FFI surface exposes init, connection management, send, and event drain:

```rust
#[no_mangle]
pub unsafe extern "C" fn goud_network_host(
    game: *mut GoudGame,
    port: u16,
    max_connections: u32,
) -> i32 { ... }

#[no_mangle]
pub unsafe extern "C" fn goud_network_connect(
    game: *mut GoudGame,
    addr: *const c_char,
    out_conn: *mut u64,
) -> i32 { ... }

#[no_mangle]
pub unsafe extern "C" fn goud_network_send(
    game: *mut GoudGame,
    conn: u64,
    channel: u8,
    data: *const u8,
    len: usize,
) -> i32 { ... }

#[no_mangle]
pub unsafe extern "C" fn goud_network_drain_events(
    game: *mut GoudGame,
    out_buf: *mut GoudNetworkEvent,
    buf_len: usize,
    out_count: *mut usize,
) -> i32 { ... }
```

`GoudNetworkEvent` is a `#[repr(C)]` flat struct with a discriminant field:

```rust
#[repr(u32)]
pub enum GoudNetworkEventKind {
    Connected    = 0,
    Disconnected = 1,
    Received     = 2,
    Error        = 3,
}

/// C-compatible network event. Fields are variant-dependent:
/// - Connected/Disconnected: `conn` is set; `data`/`data_len` are zero/null.
/// - Received: `conn`, `channel`, `data`, `data_len` are set.
/// - Error: `conn` is set; `message` points to a null-terminated string.
#[repr(C)]
pub struct GoudNetworkEvent {
    pub kind: GoudNetworkEventKind,
    pub conn: u64,
    pub channel: u8,
    pub data: *const u8,
    pub data_len: usize,
    pub message: *const c_char,
}
```

All pointer parameters require null checks before dereferencing; each `unsafe` block carries a `// SAFETY:` comment per the FFI patterns rule. The `data` and `message` pointers are valid only until the next call to `goud_network_drain_events`; the provider owns the backing memory.

---

### 3.9 ECS Integration

`ProviderRegistry` is stored as a `World` resource. Systems access `NetworkProvider` through `ProviderRegistry`:

```rust
fn network_system(registry: &mut ProviderRegistry) {
    if let Some(net) = registry.network.as_mut() {
        for event in net.drain_events() {
            // handle event
        }
    }
}
```

No separate ECS component type is needed. The provider is a singleton resource, not a per-entity component.

---

### 3.10 Network Simulation (Deferred)

A `NetworkSimWrapper` decorator will wrap any `NetworkProvider` implementation and inject configurable latency, jitter, and packet loss for local development and testing. It implements `NetworkProvider` and delegates to the inner provider after applying simulation parameters.

Implementation is deferred to F25-13. This RFC defines the trait boundary that `NetworkSimWrapper` will target.

---

## 4. Alternatives Considered

### Callback-based events

Passing closures or function pointers into the provider for event dispatch avoids the `drain_events` polling step. Closures with non-`'static` lifetimes are not object-safe, and `'static` closures make it difficult to borrow game state during the callback. `PhysicsProvider::drain_collision_events` in RFC-0001 set the precedent; networking follows the same model.

### Typed messages with generics

Generic methods (`fn send<M: Serialize>`) cannot appear in an object-safe trait. Serialization format is not a transport concern. Games choose their serialization layer independently of the transport.

### Monolithic NetworkManager

A single `NetworkManager` struct hardcoded to one transport would not support transport swapping between desktop and web targets and would violate the provider pattern established in RFC-0001.

### Async trait methods

No async runtime exists in the engine. Async traits via `async-trait` produce non-object-safe signatures. Background threads with channel-based communication achieve the same non-blocking I/O without an executor dependency.

### PeerId instead of ConnectionId

A `PeerId` implies game-level identity (player slot, lobby position). `ConnectionId` is transport-level. Conflating them forces the network layer to understand game concepts. Game code maps `ConnectionId` to player identity; the provider does not.

---

## 5. Impact

This RFC introduces a new subsystem with no breaking changes to existing code.

- **`ProviderRegistry`**: gains `network: Option<Box<dyn NetworkProvider>>`. Existing construction code continues to compile; the field defaults to `None`.
- **Error types**: network errors use `ProviderError { subsystem: "network", message }` from RFC-0001; error codes 700–709 reserved. No new `GoudError` variant needed.
- **FFI**: new `goud_network_*` functions in `goud_engine/src/ffi/network.rs`. No existing FFI function signatures change. C# bindings regenerate automatically on `cargo build`. Python `generated/_ffi.py` requires manual update.
- **SDK wrappers**: generated from `goud_sdk.schema.json` for C#, Python, and TypeScript. No existing wrapper changes.
- **Examples**: unaffected unless they explicitly opt in to networking.

Prerequisite: error type Layer 1 move, shared with RFC-0001 §3.10.

---

## 6. Open Questions

1. **Optional vs mandatory in `ProviderRegistry`.** Using `Option<Box<dyn NetworkProvider>>` requires every network call site to unwrap. An alternative is always storing `NullNetProvider` and removing the `Option`. The choice affects how the engine signals "networking not configured" vs "networking failed."

2. **Async I/O strategy.** Background `std::thread` per provider is simple but wastes a thread for games not using networking. A shared I/O thread pool (or eventual tokio adoption) would be more efficient. The trait boundary does not constrain this; implementations may evolve.

3. **`ConnectionId` reuse policy.** After a connection closes, can its `ConnectionId` value be assigned to a new connection? Reuse risks use-after-close bugs. Generational IDs (epoch + index) would eliminate the ambiguity at the cost of a wider type.

4. **Max message size enforcement.** `NetworkCapabilities::max_message_size` declares the limit but the trait does not specify whether `send` silently truncates, returns an error, or panics on oversized messages. The current leaning is that implementations MUST return `Err(ProviderError)` on oversized data (no silent truncation, no panics), making a caller-side bounds check optional but safe. This must be confirmed before implementation.

5. **Encryption and TLS surface.** `HostConfig` and `connect` have no TLS parameters. If encryption is required, `HostConfig` needs certificate paths or raw key material. Deferring this leaves a gap for any game that needs secure transport.
