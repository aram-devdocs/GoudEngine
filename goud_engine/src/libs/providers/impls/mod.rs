//! Concrete provider implementations.
//!
//! Null (no-op) implementations are canonical in `crate::core::providers::impls`
//! and re-exported here for backward compatibility. Native (GLFW/OpenGL/rodio)
//! implementations live here because they depend on Libs-layer crates.

// Re-export null providers from core (Foundation layer)
pub use crate::core::providers::impls::null_audio;
pub use crate::core::providers::impls::null_input;
pub use crate::core::providers::impls::null_network;
pub use crate::core::providers::impls::null_physics;
pub use crate::core::providers::impls::null_physics3d;
pub use crate::core::providers::impls::null_render;
pub use crate::core::providers::impls::null_window;

pub use crate::core::providers::impls::NullAudioProvider;
pub use crate::core::providers::impls::NullInputProvider;
pub use crate::core::providers::impls::NullNetworkProvider;
pub use crate::core::providers::impls::NullPhysicsProvider;
pub use crate::core::providers::impls::NullPhysicsProvider3D;
pub use crate::core::providers::impls::NullRenderProvider;
pub use crate::core::providers::impls::NullWindowProvider;

#[cfg(feature = "legacy-glfw-opengl")]
pub mod glfw_input;
#[cfg(feature = "legacy-glfw-opengl")]
pub mod glfw_window;
#[cfg(feature = "legacy-glfw-opengl")]
pub mod opengl_render;
#[cfg(feature = "desktop-native")]
pub mod rodio_audio;

#[cfg(feature = "legacy-glfw-opengl")]
pub use glfw_input::GlfwInputProvider;
#[cfg(feature = "legacy-glfw-opengl")]
pub use glfw_window::GlfwWindowProvider;
#[cfg(feature = "legacy-glfw-opengl")]
pub use opengl_render::OpenGLRenderProvider;
#[cfg(feature = "desktop-native")]
pub use rodio_audio::RodioAudioProvider;

#[cfg(feature = "rapier2d")]
pub mod rapier2d_physics;
#[cfg(feature = "rapier2d")]
pub use rapier2d_physics::Rapier2DPhysicsProvider;

#[cfg(feature = "rapier3d")]
pub mod rapier3d_physics;
#[cfg(feature = "rapier3d")]
pub use rapier3d_physics::Rapier3DPhysicsProvider;

pub mod simple_physics;
pub use simple_physics::SimplePhysicsProvider;

/// Debug-only network simulation wrapper shared by native transports.
#[cfg(any(debug_assertions, test))]
pub mod network_sim;
/// TCP transport provider implementing `NetworkProvider`.
#[cfg(feature = "net-tcp")]
pub mod tcp_network;
/// UDP transport provider implementing `NetworkProvider`.
#[cfg(feature = "net-udp")]
pub mod udp_network;
/// UDP reliability sub-module for packet sequencing and retransmission.
#[cfg(feature = "net-udp")]
pub mod udp_reliability;
#[cfg(any(debug_assertions, test))]
pub use network_sim::NetworkSimProvider;
#[cfg(feature = "net-tcp")]
pub use tcp_network::TcpNetProvider;
#[cfg(feature = "net-udp")]
pub use udp_network::UdpNetProvider;

/// WebSocket transport provider implementing `NetworkProvider`.
#[cfg(feature = "net-ws")]
pub mod ws_network;
#[cfg(feature = "net-ws")]
pub use ws_network::WsNetProvider;

/// WebRTC data channel transport provider implementing `NetworkProvider`.
#[cfg(feature = "net-webrtc")]
pub mod webrtc_network;
#[cfg(feature = "net-webrtc")]
pub use webrtc_network::WebRtcNetProvider;

/// Peer-to-peer full-mesh networking layer.
pub mod p2p_mesh;
pub use p2p_mesh::P2pMesh;

#[cfg(test)]
mod network_contract_tests;
