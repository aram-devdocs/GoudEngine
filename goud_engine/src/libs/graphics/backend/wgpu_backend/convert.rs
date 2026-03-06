//! Type conversion helpers: engine types -> wgpu types.
//!
//! Free functions that map engine-level enums to their wgpu equivalents.
//! Kept separate to avoid cluttering the main implementation files.

use super::{
    super::types::VertexAttributeType, BlendFactor, CullFace, DepthFunc, FrontFace,
    PrimitiveTopology,
};

pub(super) fn map_topology(t: PrimitiveTopology) -> wgpu::PrimitiveTopology {
    match t {
        PrimitiveTopology::Points => wgpu::PrimitiveTopology::PointList,
        PrimitiveTopology::Lines => wgpu::PrimitiveTopology::LineList,
        PrimitiveTopology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        PrimitiveTopology::Triangles => wgpu::PrimitiveTopology::TriangleList,
        PrimitiveTopology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        PrimitiveTopology::TriangleFan => wgpu::PrimitiveTopology::TriangleList,
    }
}

pub(super) fn map_vertex_format(ty: VertexAttributeType) -> wgpu::VertexFormat {
    match ty {
        VertexAttributeType::Float => wgpu::VertexFormat::Float32,
        VertexAttributeType::Float2 => wgpu::VertexFormat::Float32x2,
        VertexAttributeType::Float3 => wgpu::VertexFormat::Float32x3,
        VertexAttributeType::Float4 => wgpu::VertexFormat::Float32x4,
        VertexAttributeType::Int => wgpu::VertexFormat::Sint32,
        VertexAttributeType::Int2 => wgpu::VertexFormat::Sint32x2,
        VertexAttributeType::Int3 => wgpu::VertexFormat::Sint32x3,
        VertexAttributeType::Int4 => wgpu::VertexFormat::Sint32x4,
        VertexAttributeType::UInt => wgpu::VertexFormat::Uint32,
        VertexAttributeType::UInt2 => wgpu::VertexFormat::Uint32x2,
        VertexAttributeType::UInt3 => wgpu::VertexFormat::Uint32x3,
        VertexAttributeType::UInt4 => wgpu::VertexFormat::Uint32x4,
    }
}

pub(super) fn map_blend_factor(f: BlendFactor) -> wgpu::BlendFactor {
    match f {
        BlendFactor::Zero => wgpu::BlendFactor::Zero,
        BlendFactor::One => wgpu::BlendFactor::One,
        BlendFactor::SrcColor => wgpu::BlendFactor::Src,
        BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
        BlendFactor::DstColor => wgpu::BlendFactor::Dst,
        BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
        BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
        BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
        BlendFactor::ConstantColor => wgpu::BlendFactor::Constant,
        BlendFactor::OneMinusConstantColor => wgpu::BlendFactor::OneMinusConstant,
        BlendFactor::ConstantAlpha => wgpu::BlendFactor::Constant,
        BlendFactor::OneMinusConstantAlpha => wgpu::BlendFactor::OneMinusConstant,
    }
}

pub(super) fn map_depth_func(f: DepthFunc) -> wgpu::CompareFunction {
    match f {
        DepthFunc::Always => wgpu::CompareFunction::Always,
        DepthFunc::Never => wgpu::CompareFunction::Never,
        DepthFunc::Less => wgpu::CompareFunction::Less,
        DepthFunc::LessEqual => wgpu::CompareFunction::LessEqual,
        DepthFunc::Greater => wgpu::CompareFunction::Greater,
        DepthFunc::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
        DepthFunc::Equal => wgpu::CompareFunction::Equal,
        DepthFunc::NotEqual => wgpu::CompareFunction::NotEqual,
    }
}

pub(super) fn map_front_face(f: FrontFace) -> wgpu::FrontFace {
    match f {
        FrontFace::Ccw => wgpu::FrontFace::Ccw,
        FrontFace::Cw => wgpu::FrontFace::Cw,
    }
}

pub(super) fn map_cull_face(f: CullFace) -> Option<wgpu::Face> {
    match f {
        CullFace::Front => Some(wgpu::Face::Front),
        CullFace::Back => Some(wgpu::Face::Back),
        CullFace::FrontAndBack => None,
    }
}
