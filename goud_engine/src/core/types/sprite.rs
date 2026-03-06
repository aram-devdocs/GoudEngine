//! FFI-safe sprite and collision contact types.

// =============================================================================
// Contact Type
// =============================================================================

/// FFI-compatible contact information from a collision.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoudContact {
    /// Contact point X coordinate
    pub point_x: f32,
    /// Contact point Y coordinate
    pub point_y: f32,
    /// Collision normal X component (unit vector pointing from A to B)
    pub normal_x: f32,
    /// Collision normal Y component
    pub normal_y: f32,
    /// Penetration depth (positive = overlapping)
    pub penetration: f32,
}

impl From<crate::ecs::collision::Contact> for GoudContact {
    fn from(contact: crate::ecs::collision::Contact) -> Self {
        Self {
            point_x: contact.point.x,
            point_y: contact.point.y,
            normal_x: contact.normal.x,
            normal_y: contact.normal.y,
            penetration: contact.penetration,
        }
    }
}

// =============================================================================
// Sprite Types
// =============================================================================

/// FFI-safe Sprite representation.
///
/// This is a simplified version of the Sprite component suitable for FFI.
/// It uses raw u64 texture handles instead of generic AssetHandle types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FfiSprite {
    /// Texture handle (index and generation packed as u64).
    pub texture_handle: u64,

    /// Color tint red component (0.0 - 1.0).
    pub color_r: f32,
    /// Color tint green component (0.0 - 1.0).
    pub color_g: f32,
    /// Color tint blue component (0.0 - 1.0).
    pub color_b: f32,
    /// Color tint alpha component (0.0 - 1.0).
    pub color_a: f32,

    /// Source rectangle X position (if has_source_rect is true).
    pub source_rect_x: f32,
    /// Source rectangle Y position.
    pub source_rect_y: f32,
    /// Source rectangle width.
    pub source_rect_width: f32,
    /// Source rectangle height.
    pub source_rect_height: f32,
    /// Whether source_rect is set.
    pub has_source_rect: bool,

    /// Flip horizontally flag.
    pub flip_x: bool,
    /// Flip vertically flag.
    pub flip_y: bool,

    /// Anchor point X (normalized 0-1).
    pub anchor_x: f32,
    /// Anchor point Y (normalized 0-1).
    pub anchor_y: f32,

    /// Custom size width (if has_custom_size is true).
    pub custom_size_x: f32,
    /// Custom size height.
    pub custom_size_y: f32,
    /// Whether custom_size is set.
    pub has_custom_size: bool,
}

/// Heap-allocated sprite builder for FFI use.
///
/// This builder allows constructing a sprite by setting properties one at
/// a time without copying the entire struct on each modification.
#[repr(C)]
pub struct FfiSpriteBuilder {
    /// The sprite being built.
    pub sprite: FfiSprite,
}
