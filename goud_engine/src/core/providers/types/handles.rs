//! Opaque handle types shared across providers.

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

/// Opaque handle to a character controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterControllerHandle(pub u64);

/// Opaque handle to a loaded sound asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u64);

/// Opaque handle to an active audio playback instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId(pub u64);

/// Opaque frame token returned by `begin_frame` and consumed by `end_frame`.
///
/// Enforces begin/end pairing. The provider uses the hidden id to track
/// frame state without exposing internal bookkeeping.
#[derive(Debug)]
pub struct FrameContext {
    pub(crate) _id: u64,
}
