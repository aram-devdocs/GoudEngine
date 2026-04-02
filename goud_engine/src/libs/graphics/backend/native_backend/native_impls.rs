use crate::libs::error::GoudResult;

use super::NativeRenderBackend;
use crate::libs::graphics::backend::render_backend::{ClearOps, FrameOps, RenderBackend, StateOps};

impl RenderBackend for NativeRenderBackend {
    fn info(&self) -> &crate::libs::graphics::backend::capabilities::BackendInfo {
        self.info()
    }

    fn bind_default_vertex_array(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_default_vertex_array(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.bind_default_vertex_array(),
        }
    }

    fn validate_text_draw_state(&self) -> Result<(), String> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.validate_text_draw_state(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.validate_text_draw_state(),
        }
    }

    fn read_default_framebuffer_rgba8(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, String> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.read_default_framebuffer_rgba8(width, height),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.read_default_framebuffer_rgba8(width, height),
        }
    }
}

impl FrameOps for NativeRenderBackend {
    fn begin_frame(&mut self) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.begin_frame(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.begin_frame(),
        }
    }

    fn end_frame(&mut self) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.end_frame(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.end_frame(),
        }
    }
}

impl ClearOps for NativeRenderBackend {
    fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_clear_color(r, g, b, a),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_clear_color(r, g, b, a),
        }
    }

    fn clear_color(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.clear_color(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.clear_color(),
        }
    }

    fn clear_depth(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.clear_depth(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.clear_depth(),
        }
    }
}

impl StateOps for NativeRenderBackend {
    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_viewport(x, y, width, height),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_viewport(x, y, width, height),
        }
    }

    fn enable_depth_test(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_depth_test(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.enable_depth_test(),
        }
    }

    fn disable_depth_test(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_depth_test(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.disable_depth_test(),
        }
    }

    fn enable_blending(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_blending(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.enable_blending(),
        }
    }

    fn disable_blending(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_blending(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.disable_blending(),
        }
    }

    fn set_blend_func(
        &mut self,
        src: crate::libs::graphics::backend::BlendFactor,
        dst: crate::libs::graphics::backend::BlendFactor,
    ) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_blend_func(src, dst),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_blend_func(src, dst),
        }
    }

    fn enable_culling(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.enable_culling(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.enable_culling(),
        }
    }

    fn disable_culling(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.disable_culling(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.disable_culling(),
        }
    }

    fn set_cull_face(&mut self, face: crate::libs::graphics::backend::CullFace) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_cull_face(face),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_cull_face(face),
        }
    }

    fn set_depth_func(&mut self, func: crate::libs::graphics::backend::types::DepthFunc) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_depth_func(func),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_depth_func(func),
        }
    }

    fn set_front_face(&mut self, face: crate::libs::graphics::backend::types::FrontFace) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_front_face(face),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_front_face(face),
        }
    }

    fn set_depth_mask(&mut self, enabled: bool) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_depth_mask(enabled),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_depth_mask(enabled),
        }
    }

    fn set_multisampling_enabled(&mut self, enabled: bool) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_multisampling_enabled(enabled),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_multisampling_enabled(enabled),
        }
    }

    fn set_line_width(&mut self, width: f32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_line_width(width),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.set_line_width(width),
        }
    }
}
