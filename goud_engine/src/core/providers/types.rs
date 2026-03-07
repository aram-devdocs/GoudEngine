//! Shared types for provider traits.
//!
//! Uses opaque `u64` handles and simple descriptor structs to keep provider
//! traits self-contained. Concrete implementations convert to/from their
//! internal types.

// =============================================================================
// Resource Handles
// =============================================================================

/// Opaque handle to a GPU texture resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u64);

/// Opaque handle to a GPU buffer resource (vertex, index, uniform).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u64);

/// Opaque handle to a compiled shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub u64);

/// Opaque handle to a render pipeline (shader + vertex layout + blend mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineHandle(pub u64);

/// Opaque handle to a render target (framebuffer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderTargetHandle(pub u64);

/// Opaque handle to a physics body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub u64);

/// Opaque handle to a physics collider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColliderHandle(pub u64);

/// Opaque handle to a physics joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JointHandle(pub u64);

/// Opaque handle to a loaded sound asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u64);

/// Opaque handle to an active audio playback instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId(pub u64);

// =============================================================================
// Render Descriptors
// =============================================================================

/// Opaque frame token returned by `begin_frame` and consumed by `end_frame`.
///
/// Enforces begin/end pairing -- the caller cannot skip frame finalization.
/// The `id` field will be used by concrete provider implementations to track
/// frame state.
#[derive(Debug)]
pub struct FrameContext {
    pub(crate) _id: u64,
}

/// Describes a texture to be created.
#[derive(Debug, Clone, Default)]
pub struct TextureDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format (opaque integer; format enum can come later).
    pub format: u32,
    /// Optional raw pixel data. `None` creates an empty texture.
    pub data: Option<Vec<u8>>,
}

/// Describes a GPU buffer to be created.
#[derive(Debug, Clone, Default)]
pub struct BufferDesc {
    /// Buffer size in bytes.
    pub size: u64,
    /// Usage flags (opaque integer; usage enum can come later).
    pub usage: u32,
    /// Optional initial data.
    pub data: Option<Vec<u8>>,
}

/// Describes a shader to be compiled.
#[derive(Debug, Clone, Default)]
pub struct ShaderDesc {
    /// Vertex shader source code.
    pub vertex_source: String,
    /// Fragment shader source code.
    pub fragment_source: String,
}

/// Describes a render pipeline to be created.
#[derive(Debug, Clone, Default)]
pub struct PipelineDesc {
    /// Shader to use for this pipeline.
    pub shader: Option<ShaderHandle>,
    /// Blend mode (opaque integer; blend enum can come later).
    pub blend_mode: u32,
    /// Depth testing enabled.
    pub depth_test: bool,
    /// Depth writing enabled.
    pub depth_write: bool,
}

/// Describes a render target (framebuffer) to be created.
#[derive(Debug, Clone, Default)]
pub struct RenderTargetDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Number of color attachments.
    pub color_attachments: u32,
    /// Whether to include a depth attachment.
    pub has_depth: bool,
}

// =============================================================================
// Render Commands
// =============================================================================

/// A low-level draw command submitted to the render provider.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Pipeline state for this draw call.
    pub pipeline: PipelineHandle,
    /// Vertex buffer to draw from.
    pub vertex_buffer: BufferHandle,
    /// Optional index buffer for indexed drawing.
    pub index_buffer: Option<BufferHandle>,
    /// Optional texture binding.
    pub texture: Option<TextureHandle>,
    /// Number of instances to draw (1 for non-instanced).
    pub instance_count: u32,
    /// Number of vertices (or indices if index buffer is present).
    pub vertex_count: u32,
}

/// A 3D mesh draw command.
#[derive(Debug, Clone)]
pub struct MeshDrawCommand {
    /// Pipeline state for this draw call.
    pub pipeline: PipelineHandle,
    /// Vertex buffer containing mesh vertex data.
    pub vertex_buffer: BufferHandle,
    /// Index buffer for indexed drawing.
    pub index_buffer: BufferHandle,
    /// Optional texture binding.
    pub texture: Option<TextureHandle>,
    /// Model transform matrix (column-major 4x4).
    pub transform: [f32; 16],
    /// Number of indices to draw.
    pub index_count: u32,
}

/// A text draw command.
#[derive(Debug, Clone)]
pub struct TextDrawCommand {
    /// The text string to render.
    pub text: String,
    /// Position as [x, y].
    pub position: [f32; 2],
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color as [r, g, b, a].
    pub color: [f32; 4],
}

/// A particle system draw command.
#[derive(Debug, Clone)]
pub struct ParticleDrawCommand {
    /// Pipeline state for particle rendering.
    pub pipeline: PipelineHandle,
    /// Vertex buffer containing particle data.
    pub vertex_buffer: BufferHandle,
    /// Optional texture for particle sprites.
    pub texture: Option<TextureHandle>,
    /// Number of particles to draw.
    pub particle_count: u32,
}

/// Camera data passed to the render provider.
#[derive(Debug, Clone, Default)]
pub struct CameraData {
    /// View matrix (column-major 4x4).
    pub view: [f32; 16],
    /// Projection matrix (column-major 4x4).
    pub projection: [f32; 16],
    /// Camera position in world space.
    pub position: [f32; 3],
}

// =============================================================================
// Physics Descriptors
// =============================================================================

/// Describes a physics body to be created.
#[derive(Debug, Clone, Default)]
pub struct BodyDesc {
    /// Initial position as [x, y].
    pub position: [f32; 2],
    /// Body type (0 = static, 1 = dynamic, 2 = kinematic).
    pub body_type: u32,
    /// Linear damping.
    pub linear_damping: f32,
    /// Angular damping.
    pub angular_damping: f32,
    /// Whether gravity applies to this body.
    pub gravity_scale: f32,
    /// Fixed rotation (no angular velocity).
    pub fixed_rotation: bool,
}

/// Describes a physics collider to be attached to a body.
#[derive(Debug, Clone, Default)]
pub struct ColliderDesc {
    /// Collider shape (0 = circle, 1 = box, 2 = capsule).
    pub shape: u32,
    /// Half-extents for box shapes as [half_w, half_h].
    pub half_extents: [f32; 2],
    /// Radius for circle/capsule shapes.
    pub radius: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Restitution (bounciness).
    pub restitution: f32,
    /// Whether this collider is a sensor (triggers events, no physical response).
    pub is_sensor: bool,
}

/// Describes a physics joint connecting two bodies.
#[derive(Debug, Clone, Default)]
pub struct JointDesc {
    /// First body in the joint.
    pub body_a: Option<BodyHandle>,
    /// Second body in the joint.
    pub body_b: Option<BodyHandle>,
    /// Joint type (0 = revolute, 1 = prismatic, 2 = distance).
    pub joint_type: u32,
    /// Anchor point on body A as [x, y] in local space.
    pub anchor_a: [f32; 2],
    /// Anchor point on body B as [x, y] in local space.
    pub anchor_b: [f32; 2],
}

// =============================================================================
// Physics Events
// =============================================================================

/// Result of a physics raycast query.
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// The body that was hit.
    pub body: BodyHandle,
    /// The hit point in world space as [x, y].
    pub point: [f32; 2],
    /// The surface normal at the hit point as [x, y].
    pub normal: [f32; 2],
    /// Distance from ray origin to hit point.
    pub distance: f32,
}

/// A collision event between two bodies.
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// First body involved in the collision.
    pub body_a: BodyHandle,
    /// Second body involved in the collision.
    pub body_b: BodyHandle,
    /// Whether the collision started (true) or ended (false).
    pub started: bool,
}

/// A contact pair with contact point information.
#[derive(Debug, Clone)]
pub struct ContactPair {
    /// First body in contact.
    pub body_a: BodyHandle,
    /// Second body in contact.
    pub body_b: BodyHandle,
    /// Contact normal as [x, y].
    pub normal: [f32; 2],
    /// Penetration depth.
    pub depth: f32,
}

/// A debug visualization shape from the physics engine.
#[derive(Debug, Clone)]
pub struct DebugShape {
    /// Shape type (0 = circle, 1 = box, 2 = line).
    pub shape_type: u32,
    /// Position as [x, y].
    pub position: [f32; 2],
    /// Size/extent data (interpretation depends on shape_type).
    pub size: [f32; 2],
    /// Rotation in radians.
    pub rotation: f32,
    /// Color as [r, g, b, a].
    pub color: [f32; 4],
}

// =============================================================================
// Audio Types
// =============================================================================

/// Audio channel for volume grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    /// Master channel (affects all audio).
    Master,
    /// Music/soundtrack channel.
    Music,
    /// Sound effects channel.
    Effects,
    /// Voice/dialogue channel.
    Voice,
    /// Ambient/environmental channel.
    Ambient,
}

/// Configuration for playing a sound.
#[derive(Debug, Clone)]
pub struct PlayConfig {
    /// Volume (0.0 to 1.0).
    pub volume: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether to loop the sound.
    pub looping: bool,
    /// Audio channel for volume grouping.
    pub channel: AudioChannel,
    /// Optional spatial position as [x, y, z]. `None` for non-spatial audio.
    pub position: Option<[f32; 3]>,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            looping: false,
            channel: AudioChannel::Effects,
            position: None,
        }
    }
}

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

// =============================================================================
// Capability Structs
// =============================================================================

/// Capabilities reported by a render provider.
#[derive(Debug, Clone, Default)]
pub struct RenderCapabilities {
    /// Maximum number of texture units available.
    pub max_texture_units: u32,
    /// Maximum texture dimension (width or height).
    pub max_texture_size: u32,
    /// Whether hardware instancing is supported.
    pub supports_instancing: bool,
    /// Whether compute shaders are supported.
    pub supports_compute: bool,
    /// Whether multisample anti-aliasing is supported.
    pub supports_msaa: bool,
}

/// Capabilities reported by a physics provider.
#[derive(Debug, Clone, Default)]
pub struct PhysicsCapabilities {
    /// Whether continuous collision detection is supported.
    pub supports_continuous_collision: bool,
    /// Whether joints are supported.
    pub supports_joints: bool,
    /// Maximum number of physics bodies.
    pub max_bodies: u32,
}

/// Capabilities reported by an audio provider.
#[derive(Debug, Clone, Default)]
pub struct AudioCapabilities {
    /// Whether spatial/3D audio is supported.
    pub supports_spatial: bool,
    /// Maximum number of simultaneous audio channels.
    pub max_channels: u32,
}
