//! Texture operations: create, update, destroy, bind, unbind.

use super::{
    super::types::{TextureFilter, TextureFormat, TextureWrap},
    TextureHandle, WgpuBackend, WgpuTextureMeta,
};
use crate::libs::error::{GoudError, GoudResult};

impl WgpuBackend {
    pub(super) fn create_texture_impl(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        let wgpu_format = match format {
            TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
            TextureFormat::RG8 => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::RGB8 | TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::RGBA16F => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::RGBA32F => wgpu::TextureFormat::Rgba32Float,
            TextureFormat::Depth => wgpu::TextureFormat::Depth32Float,
            TextureFormat::DepthStencil => wgpu::TextureFormat::Depth24PlusStencil8,
            TextureFormat::BC1 => wgpu::TextureFormat::Bc1RgbaUnorm,
            TextureFormat::BC3 => wgpu::TextureFormat::Bc3RgbaUnorm,
            TextureFormat::BC5 => wgpu::TextureFormat::Bc5RgUnorm,
            TextureFormat::BC7 => wgpu::TextureFormat::Bc7RgbaUnorm,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        if !data.is_empty() {
            let bytes_per_pixel = match format {
                TextureFormat::R8 => 1,
                TextureFormat::RG8 => 2,
                TextureFormat::RGB8 | TextureFormat::RGBA8 => 4,
                TextureFormat::RGBA16F => 8,
                TextureFormat::RGBA32F => 16,
                _ => 4,
            };

            let upload_data = if matches!(format, TextureFormat::RGB8)
                && data.len() == (width * height * 3) as usize
            {
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for pixel in data.chunks(3) {
                    rgba.extend_from_slice(pixel);
                    rgba.push(255);
                }
                rgba
            } else {
                data.to_vec()
            };

            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &upload_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_pixel * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let wgpu_filter = match filter {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        };
        let wgpu_wrap = match wrap {
            TextureWrap::Repeat => wgpu::AddressMode::Repeat,
            TextureWrap::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            TextureWrap::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            TextureWrap::ClampToBorder => wgpu::AddressMode::ClampToEdge,
        };

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu_wrap,
            address_mode_v: wgpu_wrap,
            address_mode_w: wgpu_wrap,
            mag_filter: wgpu_filter,
            min_filter: wgpu_filter,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let handle = self.texture_allocator.allocate();
        self.textures.insert(
            handle,
            WgpuTextureMeta {
                _texture: texture,
                view,
                sampler,
                width,
                height,
            },
        );
        Ok(handle)
    }

    pub(super) fn update_texture_impl(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()> {
        let meta = self.textures.get(&handle).ok_or(GoudError::InvalidHandle)?;
        if x + width > meta.width || y + height > meta.height {
            return Err(GoudError::InvalidState(
                "Texture update out of bounds".into(),
            ));
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &meta._texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    pub(super) fn destroy_texture_impl(&mut self, handle: TextureHandle) -> bool {
        if self.textures.remove(&handle).is_some() {
            self.texture_allocator.deallocate(handle);
            true
        } else {
            false
        }
    }

    pub(super) fn is_texture_valid_impl(&self, handle: TextureHandle) -> bool {
        self.textures.contains_key(&handle)
    }

    pub(super) fn texture_size_impl(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.textures.get(&handle).map(|m| (m.width, m.height))
    }

    pub(super) fn bind_texture_impl(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        if !self.textures.contains_key(&handle) {
            return Err(GoudError::InvalidHandle);
        }
        if (unit as usize) < self.bound_textures.len() {
            self.bound_textures[unit as usize] = Some(handle);
        }
        Ok(())
    }

    pub(super) fn unbind_texture_impl(&mut self, unit: u32) {
        if (unit as usize) < self.bound_textures.len() {
            self.bound_textures[unit as usize] = None;
        }
    }
}
