//! WebSocket transport provider implementing `NetworkProvider`.
//!
//! Uses `tungstenite` for synchronous WebSocket I/O on native targets.
//! Each connection runs a single I/O thread that handles both reading
//! (non-blocking with short timeouts) and writing (from an mpsc channel).

#[cfg(not(target_arch = "wasm32"))]
mod io;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
mod tls;

#[cfg(not(target_arch = "wasm32"))]
pub use native::WsNetProvider;

// WASM WebSocket support deferred to future PR
#[cfg(target_arch = "wasm32")]
compile_error!("WebSocket provider is not yet implemented for WASM targets");

#[cfg(test)]
#[path = "../ws_network_tests.rs"]
mod tests;
