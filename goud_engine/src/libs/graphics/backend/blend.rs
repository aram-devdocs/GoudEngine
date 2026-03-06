//! Blend and rasterisation state types.
//!
//! This module defines enumerations used to configure the fixed-function
//! blend and rasterisation stages of the render pipeline.

/// Blend factor for alpha blending operations.
#[allow(dead_code)] // Will be used in Step 5.1.3+
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    /// `(0, 0, 0, 0)`
    Zero = 0,
    /// `(1, 1, 1, 1)`
    One = 1,
    /// `(Rs, Gs, Bs, As)`
    SrcColor = 2,
    /// `(1-Rs, 1-Gs, 1-Bs, 1-As)`
    OneMinusSrcColor = 3,
    /// `(Rd, Gd, Bd, Ad)`
    DstColor = 4,
    /// `(1-Rd, 1-Gd, 1-Bd, 1-Ad)`
    OneMinusDstColor = 5,
    /// `(As, As, As, As)`
    SrcAlpha = 6,
    /// `(1-As, 1-As, 1-As, 1-As)`
    OneMinusSrcAlpha = 7,
    /// `(Ad, Ad, Ad, Ad)`
    DstAlpha = 8,
    /// `(1-Ad, 1-Ad, 1-Ad, 1-Ad)`
    OneMinusDstAlpha = 9,
    /// `(Rc, Gc, Bc, Ac)`
    ConstantColor = 10,
    /// `(1-Rc, 1-Gc, 1-Bc, 1-Ac)`
    OneMinusConstantColor = 11,
    /// `(Ac, Ac, Ac, Ac)`
    ConstantAlpha = 12,
    /// `(1-Ac, 1-Ac, 1-Ac, 1-Ac)`
    OneMinusConstantAlpha = 13,
}

/// Face culling mode.
#[allow(dead_code)] // Will be used in Step 5.1.3+
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CullFace {
    /// Cull front-facing triangles
    Front = 0,
    /// Cull back-facing triangles (most common)
    #[default]
    Back = 1,
    /// Cull both front and back faces
    FrontAndBack = 2,
}
