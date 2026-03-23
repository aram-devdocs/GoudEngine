//! Shader compilation, linking, and shader section of the RenderBackend impl.

use super::{
    init::UNIFORM_BUFFER_SIZE, resources::UniformSlot, ShaderHandle, VertexLayout, WgpuBackend,
    WgpuShaderMeta,
};
use crate::libs::error::{GoudError, GoudResult};
use std::collections::HashMap;

impl WgpuBackend {
    /// Attempts to transpile GLSL source to WGSL via naga.
    /// Falls back to treating the source as WGSL if the source does not start with `#`.
    fn transpile_glsl(
        source: &str,
        stage: naga::ShaderStage,
    ) -> GoudResult<(naga::Module, String)> {
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

        let wgsl =
            naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
                .map_err(|e| GoudError::ShaderCompilationFailed(format!("WGSL write: {e}")))?;

        Ok((module, wgsl))
    }

    /// Extract uniform names and byte offsets from a naga module.
    ///
    /// Walks the module's global variables looking for uniform-address-space
    /// bindings in group 0, binding 0 (the standard uniform block).  For each
    /// struct member it records the name and byte offset so that
    /// `get_uniform_location_impl` can return the correct location.
    fn extract_uniform_slots(module: &naga::Module, slots: &mut HashMap<String, UniformSlot>) {
        for (_, gv) in module.global_variables.iter() {
            if gv.space != naga::AddressSpace::Uniform {
                continue;
            }
            // Resolve the inner type
            let ty = &module.types[gv.ty];
            match &ty.inner {
                naga::TypeInner::Struct { members, .. } => {
                    for m in members {
                        if let Some(ref name) = m.name {
                            let member_ty = &module.types[m.ty];
                            let size = member_ty.inner.size(module.to_ctx());
                            slots.entry(name.clone()).or_insert(UniformSlot {
                                offset: m.offset as usize,
                                _size: size as usize,
                            });

                            // Recurse into array-of-struct members to support
                            // names like "lights[0].position" used by the 3D
                            // renderer's per-light uniform lookups.
                            Self::extract_array_of_struct_slots(
                                module,
                                name,
                                m.offset as usize,
                                &member_ty.inner,
                                slots,
                            );
                        }
                    }
                }
                // Single scalar/vector/matrix uniform (not in a struct)
                _ => {
                    if let Some(ref name) = gv.name {
                        let size = ty.inner.size(module.to_ctx());
                        let offset = slots
                            .values()
                            .map(|s| s.offset + s._size)
                            .max()
                            .unwrap_or(0);
                        // Align to 16 bytes (std140)
                        let aligned = (offset + 15) & !15;
                        slots.entry(name.clone()).or_insert(UniformSlot {
                            offset: aligned,
                            _size: size as usize,
                        });
                    }
                }
            }
        }
    }

    /// For a member whose type is `Array { base: Struct, .. }`, enumerate each
    /// array element's struct members and insert slots named
    /// `"{array_name}[{index}].{field_name}"`.
    fn extract_array_of_struct_slots(
        module: &naga::Module,
        array_name: &str,
        array_base_offset: usize,
        inner: &naga::TypeInner,
        slots: &mut HashMap<String, UniformSlot>,
    ) {
        let (base_ty_handle, count, stride) = match inner {
            naga::TypeInner::Array { base, size, stride } => {
                let count = match size {
                    naga::ArraySize::Constant(n) => n.get() as usize,
                    _ => return,
                };
                (*base, count, *stride as usize)
            }
            _ => return,
        };

        let base_ty = &module.types[base_ty_handle];
        let struct_members = match &base_ty.inner {
            naga::TypeInner::Struct { members, .. } => members,
            _ => return,
        };

        for idx in 0..count {
            let element_offset = array_base_offset + idx * stride;
            for m in struct_members {
                if let Some(ref field_name) = m.name {
                    let slot_name = format!("{array_name}[{idx}].{field_name}");
                    let size = module.types[m.ty].inner.size(module.to_ctx());
                    slots.entry(slot_name).or_insert(UniformSlot {
                        offset: element_offset + m.offset as usize,
                        _size: size as usize,
                    });
                }
            }
        }
    }

    /// Parse raw WGSL source via naga for reflection (uniform struct extraction).
    fn parse_wgsl(source: &str) -> Option<naga::Module> {
        naga::front::wgsl::parse_str(source).ok()
    }

    /// Create a wgpu shader module from GLSL (auto-transpiled) or raw WGSL.
    /// Returns `(ShaderModule, Option<naga::Module>)` — the naga module is
    /// used for uniform reflection.
    fn create_shader_module_with_reflection(
        device: &wgpu::Device,
        source: &str,
        _stage: naga::ShaderStage,
    ) -> GoudResult<(wgpu::ShaderModule, Option<naga::Module>)> {
        let (naga_module, wgsl) = if source.trim_start().starts_with('#') {
            // GLSL → transpile to WGSL
            let (m, w) = Self::transpile_glsl(source, _stage)?;
            (Some(m), w)
        } else {
            // Raw WGSL → parse for reflection
            let m = Self::parse_wgsl(source);
            (m, source.to_string())
        };

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
        Ok((module, naga_module))
    }

    pub(super) fn create_shader_impl(
        &mut self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> GoudResult<ShaderHandle> {
        let (vertex_module, vertex_naga) = Self::create_shader_module_with_reflection(
            &self.device,
            vertex_src,
            naga::ShaderStage::Vertex,
        )?;
        let (fragment_module, fragment_naga) = Self::create_shader_module_with_reflection(
            &self.device,
            fragment_src,
            naga::ShaderStage::Fragment,
        )?;

        // Reflect uniform names/offsets from both shader stages.
        let mut uniform_slots = HashMap::new();
        if let Some(ref m) = vertex_naga {
            Self::extract_uniform_slots(m, &mut uniform_slots);
        }
        if let Some(ref m) = fragment_naga {
            Self::extract_uniform_slots(m, &mut uniform_slots);
        }

        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: UNIFORM_BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // For dynamic-offset bind groups, the binding size must be the slot
        // region size, not the entire buffer.  Compute from reflected slots or
        // fall back to a safe minimum.
        let slot_end = uniform_slots
            .values()
            .map(|s| s.offset + s._size)
            .max()
            .unwrap_or(256);
        let align = self.device.limits().min_uniform_buffer_offset_alignment as usize;
        let binding_size = ((slot_end + align - 1) & !(align - 1)).max(align);

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(binding_size as u64),
                }),
            }],
        });

        let handle = self.shader_allocator.allocate();
        self.shaders.insert(
            handle,
            WgpuShaderMeta {
                vertex_module,
                fragment_module,
                uniform_slots,
                uniform_staging: vec![0u8; UNIFORM_BUFFER_SIZE],
                uniform_buffer,
                uniform_bind_group,
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
