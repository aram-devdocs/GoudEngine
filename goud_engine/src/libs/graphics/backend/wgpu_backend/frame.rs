//! Sub-trait and RenderBackend implementations for `WgpuBackend`.
use super::{
    super::types::{BufferUsage, TextureFilter, TextureFormat, TextureWrap},
    BlendFactor, BufferHandle, BufferOps, BufferType, CullFace, DepthFunc, DrawType, FrameOps,
    FrameState, FrontFace, PipelineKey, ShaderHandle, ShaderOps, StateOps, TextureHandle,
    TextureOps, WgpuBackend,
};
use crate::libs::error::{GoudError, GoudResult};

impl FrameOps for WgpuBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        use crate::libs::graphics::frame_timing;

        frame_timing::reset_timings();

        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return Ok(()), // Surface dropped (mobile suspended) -- skip frame
        };
        let acquire_start = std::time::Instant::now();
        let surface_texture = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) => tex,
            wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                surface.configure(&self.device, &self.surface_config);
                tex
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(()); // skip frame
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&self.device, &self.surface_config);
                return Err(GoudError::InternalError("Surface lost or outdated".into()));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(GoudError::InternalError("Surface validation error".into()));
            }
        };
        let acquire_us = acquire_start.elapsed().as_micros() as u64;
        frame_timing::record_timing("surface_acquire", acquire_us);
        crate::core::debugger::record_phase_duration("surface_acquire", acquire_us);

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(FrameState {
            surface_texture,
            surface_view,
        });
        self.draw_commands.clear();
        self.uniform_ring.clear();
        // Always clear each frame to match OpenGL's glClear() behavior and avoid
        // uninitialized surface data showing through as garbage artifacts.
        self.needs_clear = true;
        Ok(())
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        use crate::libs::graphics::frame_timing;

        let frame = self
            .current_frame
            .take()
            .ok_or(GoudError::InvalidState("No active frame".into()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // -- uniform_upload phase -------------------------------------------------
        let uniform_start = std::time::Instant::now();

        // Upload per-draw-command uniform data into aligned dynamic-offset
        // slots.  Returns the byte offset for each command.
        let cmd_offsets = self.upload_per_draw_uniforms();

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

        // Ensure cached storage buffer bind groups exist for each draw command.
        for cmd in &self.draw_commands {
            if let Some(buf_handle) = cmd.storage_buffer {
                if !self.storage_bind_group_cache.contains_key(&buf_handle) {
                    if let Some(meta) = self.buffers.get(&buf_handle) {
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("skinning-storage-bg"),
                            layout: &self.storage_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: meta.buffer.as_entire_binding(),
                            }],
                        });
                        self.storage_bind_group_cache.insert(buf_handle, bg);
                    }
                }
            }
        }

        let uniform_us = uniform_start.elapsed().as_micros() as u64;
        frame_timing::record_timing("uniform_upload", uniform_us);
        crate::core::debugger::record_phase_duration("uniform_upload", uniform_us);

        let readback = self
            .surface_supports_copy_src
            .then(|| self.prepare_frame_readback());

        // -- render_pass phase ----------------------------------------------------
        let render_pass_start = std::time::Instant::now();
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
                pass.set_pipeline(pipeline);
                for (slot, binding) in cmd.vertex_bindings.iter().enumerate() {
                    let Some(vb_meta) = self.buffers.get(&binding.buffer) else {
                        continue;
                    };
                    pass.set_vertex_buffer(slot as u32, vb_meta.buffer.slice(..));
                }

                if let Some(shader_meta) = self.shaders.get(&cmd.shader) {
                    pass.set_bind_group(0, &shader_meta.uniform_bind_group, &[cmd_offsets[i]]);
                }

                if let Some((_unit, tex_handle)) = cmd.bound_textures.first() {
                    if let Some(tex_meta) = self.textures.get(tex_handle) {
                        pass.set_bind_group(1, &tex_meta.bind_group, &[]);
                    }
                } else {
                    pass.set_bind_group(1, &self.fallback_tex_bind_group, &[]);
                }

                // Set storage buffer bind group at group(2) for GPU skinning.
                if let Some(bg) = cmd
                    .storage_buffer
                    .and_then(|h| self.storage_bind_group_cache.get(&h))
                {
                    pass.set_bind_group(2, bg, &[]);
                } else if cmd.storage_buffer.is_some() {
                    pass.set_bind_group(2, &self.fallback_storage_bind_group, &[]);
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
                        let first = cmd.draw_type.first_index();
                        pass.draw_indexed(first..first + count, 0, 0..1);
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
                        let first = cmd.draw_type.first_index();
                        pass.draw_indexed(first..first + count, 0, 0..instances);
                    }
                }
            }
        }
        let render_pass_us = render_pass_start.elapsed().as_micros() as u64;
        frame_timing::record_timing("render_pass", render_pass_us);
        crate::core::debugger::record_phase_duration("render_pass", render_pass_us);

        if let Some(readback) = readback {
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

            // -- gpu_submit phase (readback path) ---------------------------------
            let submit_start = std::time::Instant::now();
            self.queue.submit(std::iter::once(encoder.finish()));
            let submit_us = submit_start.elapsed().as_micros() as u64;
            frame_timing::record_timing("gpu_submit", submit_us);
            crate::core::debugger::record_phase_duration("gpu_submit", submit_us);

            // -- readback_stall phase ---------------------------------------------
            let readback_start = std::time::Instant::now();
            self.finish_frame_readback(readback)?;
            let readback_us = readback_start.elapsed().as_micros() as u64;
            frame_timing::record_timing("readback_stall", readback_us);
            crate::core::debugger::record_phase_duration("readback_stall", readback_us);
        } else {
            // -- gpu_submit phase (no-readback path) ------------------------------
            let submit_start = std::time::Instant::now();
            self.queue.submit(std::iter::once(encoder.finish()));
            let submit_us = submit_start.elapsed().as_micros() as u64;
            frame_timing::record_timing("gpu_submit", submit_us);
            crate::core::debugger::record_phase_duration("gpu_submit", submit_us);

            self.last_frame_readback = None;
        }

        // -- surface_present phase ------------------------------------------------
        let present_start = std::time::Instant::now();
        frame.surface_texture.present();
        let present_us = present_start.elapsed().as_micros() as u64;
        frame_timing::record_timing("surface_present", present_us);
        crate::core::debugger::record_phase_duration("surface_present", present_us);

        self.draw_commands.clear();
        self.flush_pending_buffer_destroys();
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
    fn set_multisampling_enabled(&mut self, _enabled: bool) {
        // Sample count is configured when pipelines and render targets are created.
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

    fn supports_storage_buffers(&self) -> bool {
        true
    }

    fn create_storage_buffer(&mut self, data: &[u8]) -> GoudResult<BufferHandle> {
        self.create_storage_buffer_impl(data)
    }

    fn update_storage_buffer(
        &mut self,
        handle: BufferHandle,
        offset: usize,
        data: &[u8],
    ) -> GoudResult<()> {
        self.update_storage_buffer_impl(handle, offset, data)
    }

    fn bind_storage_buffer(&mut self, handle: BufferHandle, _binding: u32) -> GoudResult<()> {
        // Record the storage buffer handle so subsequent draw commands include
        // it. The actual bind group is created at end_frame time.
        self.bound_storage_buffer = Some(handle);
        Ok(())
    }

    fn unbind_storage_buffer(&mut self) {
        self.bound_storage_buffer = None;
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

// DrawOps is implemented in frame_draw_ops.rs
