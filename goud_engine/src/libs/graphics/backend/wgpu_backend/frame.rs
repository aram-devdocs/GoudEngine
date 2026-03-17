//! Sub-trait and RenderBackend implementations for WgpuBackend.
//!
//! Covers frame lifecycle, render state management, draw call dispatch,
//! and uniform setters. Pipeline building lives in `pipeline.rs`.

use super::{
    super::types::{BufferUsage, TextureFilter, TextureFormat, TextureWrap},
    BlendFactor, BufferHandle, BufferOps, BufferType, CullFace, DepthFunc, DrawOps, DrawType,
    FrameOps, FrameState, FrontFace, PipelineKey, PrimitiveTopology, ShaderHandle, ShaderOps,
    StateOps, TextureHandle, TextureOps, VertexLayout, WgpuBackend,
};
use crate::libs::error::{GoudError, GoudResult};

// ========================================================================
// FrameOps
// ========================================================================

impl FrameOps for WgpuBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|e| GoudError::InternalError(format!("Surface texture: {e}")))?;
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(FrameState {
            surface_texture,
            surface_view,
        });
        self.draw_commands.clear();
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        let frame = self
            .current_frame
            .take()
            .ok_or(GoudError::InvalidState("No active frame".into()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Upload uniform data for each shader used this frame
        let shader_handles: Vec<ShaderHandle> = self
            .draw_commands
            .iter()
            .map(|c| c.shader)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for sh in &shader_handles {
            if let Some(cmd) = self.draw_commands.iter().rev().find(|c| c.shader == *sh) {
                if let Some(meta) = self.shaders.get(sh) {
                    self.queue
                        .write_buffer(&meta.uniform_buffer, 0, &cmd.uniform_snapshot);
                }
            }
        }

        let load_op = if self.needs_clear {
            self.needs_clear = false;
            wgpu::LoadOp::Clear(self.clear_color)
        } else {
            wgpu::LoadOp::Load
        };

        // Collect pipeline keys and ensure pipelines exist before the render pass borrow
        let cmd_keys: Vec<PipelineKey> = self
            .draw_commands
            .iter()
            .map(|cmd| self.make_pipeline_key(cmd))
            .collect();

        self.build_missing_pipelines(&cmd_keys);

        let readback = self.prepare_frame_readback();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            for (i, cmd) in self.draw_commands.iter().enumerate() {
                let key = &cmd_keys[i];
                let Some(pipeline) = self.pipeline_cache.get(key) else {
                    continue;
                };
                let Some(vb_meta) = self.buffers.get(&cmd.vertex_buffer) else {
                    continue;
                };

                pass.set_pipeline(pipeline);
                pass.set_vertex_buffer(0, vb_meta.buffer.slice(..));

                if let Some(shader_meta) = self.shaders.get(&cmd.shader) {
                    pass.set_bind_group(0, &shader_meta.uniform_bind_group, &[]);
                }

                if let Some((_unit, tex_handle)) = cmd.bound_textures.first() {
                    if let Some(tex_meta) = self.textures.get(tex_handle) {
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&tex_meta.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&tex_meta.sampler),
                                },
                            ],
                        });
                        pass.set_bind_group(1, &bg, &[]);
                    }
                } else {
                    let fallback_bg = self.fallback_texture_bind_group();
                    pass.set_bind_group(1, &fallback_bg, &[]);
                }

                if let Some(ib_handle) = cmd.index_buffer {
                    if let Some(ib_meta) = self.buffers.get(&ib_handle) {
                        let format = match cmd.draw_type {
                            DrawType::IndexedU16 { .. } => wgpu::IndexFormat::Uint16,
                            _ => wgpu::IndexFormat::Uint32,
                        };
                        pass.set_index_buffer(ib_meta.buffer.slice(..), format);
                    }
                }

                match cmd.draw_type {
                    DrawType::Arrays { first, count } => {
                        pass.draw(first..first + count, 0..1);
                    }
                    DrawType::Indexed { count, .. } | DrawType::IndexedU16 { count, .. } => {
                        pass.draw_indexed(0..count, 0, 0..1);
                    }
                    DrawType::ArraysInstanced {
                        first,
                        count,
                        instances,
                    } => {
                        pass.draw(first..first + count, 0..instances);
                    }
                    DrawType::IndexedInstanced {
                        count, instances, ..
                    } => {
                        pass.draw_indexed(0..count, 0, 0..instances);
                    }
                }
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &frame.surface_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback.padded_bytes_per_row),
                    rows_per_image: Some(readback.height),
                },
            },
            wgpu::Extent3d {
                width: readback.width,
                height: readback.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        self.finish_frame_readback(readback)?;
        frame.surface_texture.present();
        self.draw_commands.clear();
        Ok(())
    }
}

// ========================================================================
// StateOps
// ========================================================================

impl StateOps for WgpuBackend {
    fn set_viewport(&mut self, _x: i32, _y: i32, _width: u32, _height: u32) {
        // wgpu viewport is set per render pass; tracked state is applied in end_frame
    }

    fn enable_depth_test(&mut self) {
        self.depth_test_enabled = true;
    }
    fn disable_depth_test(&mut self) {
        self.depth_test_enabled = false;
    }
    fn enable_blending(&mut self) {
        self.blend_enabled = true;
        self.blend_src = BlendFactor::SrcAlpha;
        self.blend_dst = BlendFactor::OneMinusSrcAlpha;
    }
    fn disable_blending(&mut self) {
        self.blend_enabled = false;
    }
    fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        self.blend_src = src;
        self.blend_dst = dst;
    }
    fn enable_culling(&mut self) {
        self.cull_enabled = true;
    }
    fn disable_culling(&mut self) {
        self.cull_enabled = false;
    }
    fn set_cull_face(&mut self, face: CullFace) {
        self.cull_face = face;
    }
    fn set_depth_func(&mut self, func: DepthFunc) {
        self.depth_func = func;
    }
    fn set_front_face(&mut self, face: FrontFace) {
        self.front_face_state = face;
    }
    fn set_depth_mask(&mut self, enabled: bool) {
        self.depth_write_enabled = enabled;
    }
    fn set_line_width(&mut self, _width: f32) {
        // wgpu does not support variable line width (WebGPU spec limitation)
    }
}

// ========================================================================
// BufferOps (delegated to buffer.rs)
// ========================================================================

impl BufferOps for WgpuBackend {
    fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        usage: BufferUsage,
        data: &[u8],
    ) -> GoudResult<BufferHandle> {
        self.create_buffer_impl(buffer_type, usage, data)
    }

    fn update_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        self.update_buffer_impl(handle, offset, data)
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) -> bool {
        self.destroy_buffer_impl(handle)
    }

    fn is_buffer_valid(&self, handle: BufferHandle) -> bool {
        self.is_buffer_valid_impl(handle)
    }

    fn buffer_size(&self, handle: BufferHandle) -> Option<usize> {
        self.buffer_size_impl(handle)
    }

    fn bind_buffer(&mut self, handle: BufferHandle) -> GoudResult<()> {
        self.bind_buffer_impl(handle)
    }

    fn unbind_buffer(&mut self, buffer_type: BufferType) {
        self.unbind_buffer_impl(buffer_type);
    }
}

// ========================================================================
// TextureOps (delegated to texture.rs)
// ========================================================================

impl TextureOps for WgpuBackend {
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle> {
        self.create_texture_impl(width, height, format, filter, wrap, data)
    }

    fn update_texture(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()> {
        self.update_texture_impl(handle, x, y, width, height, data)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> bool {
        self.destroy_texture_impl(handle)
    }

    fn is_texture_valid(&self, handle: TextureHandle) -> bool {
        self.is_texture_valid_impl(handle)
    }

    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.texture_size_impl(handle)
    }

    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()> {
        self.bind_texture_impl(handle, unit)
    }

    fn unbind_texture(&mut self, unit: u32) {
        self.unbind_texture_impl(unit);
    }
}

// ========================================================================
// ShaderOps (delegated to shader.rs)
// ========================================================================

impl ShaderOps for WgpuBackend {
    fn create_shader(&mut self, vertex_src: &str, fragment_src: &str) -> GoudResult<ShaderHandle> {
        self.create_shader_impl(vertex_src, fragment_src)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> bool {
        self.destroy_shader_impl(handle)
    }

    fn is_shader_valid(&self, handle: ShaderHandle) -> bool {
        self.is_shader_valid_impl(handle)
    }

    fn bind_shader(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        self.bind_shader_impl(handle)
    }

    fn unbind_shader(&mut self) {
        self.unbind_shader_impl();
    }

    fn get_uniform_location(&self, handle: ShaderHandle, name: &str) -> Option<i32> {
        self.get_uniform_location_impl(handle, name)
    }

    fn set_uniform_int(&mut self, location: i32, value: i32) {
        self.write_uniform(location, &value.to_le_bytes());
    }

    fn set_uniform_float(&mut self, location: i32, value: f32) {
        self.write_uniform(location, &value.to_le_bytes());
    }

    fn set_uniform_vec2(&mut self, location: i32, x: f32, y: f32) {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        let mut buf = [0u8; 12];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        buf[8..12].copy_from_slice(&z.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_vec4(&mut self, location: i32, x: f32, y: f32, z: f32, w: f32) {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&x.to_le_bytes());
        buf[4..8].copy_from_slice(&y.to_le_bytes());
        buf[8..12].copy_from_slice(&z.to_le_bytes());
        buf[12..16].copy_from_slice(&w.to_le_bytes());
        self.write_uniform(location, &buf);
    }

    fn set_uniform_mat4(&mut self, location: i32, matrix: &[f32; 16]) {
        self.write_uniform(location, bytemuck::cast_slice(matrix));
    }
}

// ========================================================================
// DrawOps
// ========================================================================

impl DrawOps for WgpuBackend {
    fn set_vertex_attributes(&mut self, layout: &VertexLayout) {
        self.set_vertex_attributes_impl(layout);
    }

    fn draw_arrays(
        &mut self,
        _topology: PrimitiveTopology,
        first: u32,
        count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::Arrays { first, count })
    }

    fn draw_indexed(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::Indexed {
            count,
            _offset: offset,
        })
    }

    fn draw_indexed_u16(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::IndexedU16 {
            count,
            _offset: offset,
        })
    }

    fn draw_arrays_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        first: u32,
        count: u32,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::ArraysInstanced {
            first,
            count,
            instances: instance_count,
        })
    }

    fn draw_indexed_instanced(
        &mut self,
        _topology: PrimitiveTopology,
        count: u32,
        offset: usize,
        instance_count: u32,
    ) -> GoudResult<()> {
        self.record_draw(DrawType::IndexedInstanced {
            count,
            _offset: offset,
            instances: instance_count,
        })
    }
}
