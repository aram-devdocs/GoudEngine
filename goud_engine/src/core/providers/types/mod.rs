//! Shared types for provider traits.
//!
//! Uses opaque handles and simple descriptor structs to keep provider traits
//! self-contained. Concrete implementations convert to and from their internal
//! types.

mod audio;
mod capabilities;
mod handles;
mod physics;
mod render;

pub use audio::{AudioChannel, PlayConfig};
pub use capabilities::{AudioCapabilities, PhysicsCapabilities, RenderCapabilities};
pub use handles::{
    BodyHandle, BufferHandle, ColliderHandle, FrameContext, JointHandle, PipelineHandle,
    PlaybackId, RenderTargetHandle, ShaderHandle, SoundHandle, TextureHandle,
};
pub use physics::{
    BodyDesc, ColliderDesc, CollisionEvent, CollisionEventKind, ContactPair, DebugShape, JointDesc,
    JointKind, JointLimits, JointMotor, PhysicsBackend2D, RaycastHit,
};
pub use render::{
    BufferDesc, CameraData, DrawCommand, MeshDrawCommand, ParticleDrawCommand, PipelineDesc,
    RenderTargetDesc, ShaderDesc, TextDrawCommand, TextureDesc,
};

// =============================================================================
// 3D Physics Types (re-exported from types3d module)
// =============================================================================

pub use super::types3d::{
    BodyDesc3D, ColliderDesc3D, ContactPair3D, DebugShape3D, JointDesc3D, PhysicsCapabilities3D,
    RaycastHit3D,
};

// =============================================================================
// Input Types (re-exported from input_types module)
// =============================================================================

pub use super::input_types::{
    GamepadAxis, GamepadButton, GamepadId, InputCapabilities, KeyCode, MouseButton,
};

// =============================================================================
// Network Types (re-exported from network_types module)
// =============================================================================

pub use super::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats,
};
