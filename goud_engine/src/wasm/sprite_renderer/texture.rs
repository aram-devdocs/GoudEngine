//! Texture helpers: white-fallback creation, sprite texture upload, index generation,
//! and orthographic projection math.

use bytemuck::{Pod, Zeroable};

use super::types::TextureEntry;

// ---------------------------------------------------------------------------
// Sprite capacity constants (re-exported here for use across submodules)
// ---------------------------------------------------------------------------

pub(super) const MAX_SPRITES: usize = 4096;
pub(super) const VERTS_PER_SPRITE: usize = 4;
pub(super) const INDICES_PER_SPRITE: usize = 6;

// ---------------------------------------------------------------------------
// Index buffer generation
// ---------------------------------------------------------------------------

/// Generates a static index buffer for `max_sprites` quads (two triangles each).
pub(super) fn generate_indices(max_sprites: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity(max_sprites * INDICES_PER_SPRITE);
    for i in 0..max_sprites as u32 {
        let base = i * VERTS_PER_SPRITE as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
    indices
}

// ---------------------------------------------------------------------------
// White fallback texture
// ---------------------------------------------------------------------------

/// Creates a 1×1 white RGBA texture and returns its bind group.
///
/// Used as the fallback when no texture is bound to a sprite batch.
pub(super) fn create_white_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("white_1x1"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[255u8, 255, 255, 255],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("white_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

// ---------------------------------------------------------------------------
// Sprite texture upload
// ---------------------------------------------------------------------------

/// Creates a [`TextureEntry`] from raw RGBA pixels.
pub fn create_texture_entry(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> TextureEntry {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("sprite_tex_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    TextureEntry {
        view,
        bind_group,
        width,
        height,
    }
}

// ---------------------------------------------------------------------------
// Orthographic projection
// ---------------------------------------------------------------------------

/// 2D orthographic projection matrix: (0,0) top-left, (w,h) bottom-right.
/// Column-major layout matching WGSL `mat4x4<f32>`.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct OrthoMatrix {
    pub data: [[f32; 4]; 4],
}

/// Builds an [`OrthoMatrix`] for a screen of `w × h` pixels.
pub(super) fn ortho_projection(w: f32, h: f32) -> OrthoMatrix {
    OrthoMatrix {
        data: [
            [2.0 / w, 0.0, 0.0, 0.0],
            [0.0, -2.0 / h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ],
    }
}
