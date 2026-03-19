//! Shader compilation, linking, and shader section of the RenderBackend impl.

use super::{init::UNIFORM_BUFFER_SIZE, ShaderHandle, VertexLayout, WgpuBackend, WgpuShaderMeta};
use crate::libs::error::{GoudError, GoudResult};
use std::collections::HashMap;

impl WgpuBackend {
    /// Attempts to transpile GLSL source to WGSL via naga.
    /// Falls back to treating the source as WGSL if the source does not start with `#`.
    pub(super) fn transpile_to_wgsl(source: &str, stage: naga::ShaderStage) -> GoudResult<String> {
        let opts = naga::front::glsl::Options {
            stage,
            defines: Default::default(),
        };
        let module = naga::front::glsl::Frontend::default()
            .parse(&opts, source)
            .map_err(|errs| {
                GoudError::ShaderCompilationFailed(format!("GLSL parse: {:?}", errs))
            })?;

        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        let info = validator
            .validate(&module)
            .map_err(|e| GoudError::ShaderCompilationFailed(format!("Validation: {e}")))?;

        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .map_err(|e| GoudError::ShaderCompilationFailed(format!("WGSL write: {e}")))
    }

    pub(super) fn create_shader_module(
        device: &wgpu::Device,
        source: &str,
        stage: naga::ShaderStage,
    ) -> GoudResult<wgpu::ShaderModule> {
        let wgsl = if source.trim_start().starts_with('#') {
            Self::transpile_to_wgsl(source, stage)?
        } else {
            source.to_string()
        };

        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        }))
    }

    pub(super) fn create_shader_impl(
        &mut self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        let vertex_module =
            Self::create_shader_module(&self.device, vertex_src, naga::ShaderStage::Vertex)?;
        let fragment_module =
            Self::create_shader_module(&self.device, fragment_src, naga::ShaderStage::Fragment)?;

        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: UNIFORM_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let handle = self.shader_allocator.allocate();
        self.shaders.insert(
            handle,
            WgpuShaderMeta {
                vertex_module,
                fragment_module,
                uniform_slots: HashMap::new(),
                uniform_staging: vec![0u8; UNIFORM_BUFFER_SIZE],
                uniform_buffer,
                uniform_bind_group,
                _next_uniform_offset: 0,
            },
        );
        Ok(handle)
    }

    pub(super) fn destroy_shader_impl(&mut self, handle: ShaderHandle) -> bool {
        if self.shaders.remove(&handle).is_some() {
            self.shader_allocator.deallocate(handle);
            // Invalidate cached pipelines that referenced this shader
            self.pipeline_cache.retain(|k, _| k.shader != handle);
            true
        } else {
            false
        }
    }

    pub(super) fn is_shader_valid_impl(&self, handle: ShaderHandle) -> bool {
        self.shaders.contains_key(&handle)
    }

    pub(super) fn bind_shader_impl(&mut self, handle: ShaderHandle) -> GoudResult<()> {
        if !self.shaders.contains_key(&handle) {
            return Err(GoudError::InvalidHandle);
        }
        self.bound_shader = Some(handle);
        Ok(())
    }

    pub(super) fn unbind_shader_impl(&mut self) {
        self.bound_shader = None;
    }

    pub(super) fn get_uniform_location_impl(
        &self,
        handle: ShaderHandle,
        name: &str,
    ) -> Option<i32> {
        self.shaders.get(&handle).and_then(|meta| {
            meta.uniform_slots
                .get(name)
                .map(|slot| (slot.offset / 4) as i32)
        })
    }

    pub(super) fn set_vertex_attributes_impl(&mut self, layout: &VertexLayout) {
        self.current_layout = Some(layout.clone());
        self.current_vertex_bindings.clear();
    }
}
