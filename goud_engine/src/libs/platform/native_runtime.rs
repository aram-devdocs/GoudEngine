//! Native runtime factory for valid window/render backend pairs.

use crate::core::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::native_backend::{
    NativeRenderBackend, SharedNativeRenderBackend,
};
use crate::sdk::game_config::{RenderBackendKind, WindowBackendKind};

use super::{PlatformBackend, WindowConfig};

/// Concrete native runtime bundle used by SDK and FFI entry points.
pub struct NativeRuntime {
    /// Platform backend that owns the native window and event pump.
    pub platform: Box<dyn PlatformBackend>,
    /// Shared render backend handle for SDK, FFI, and renderer state.
    pub render_backend: SharedNativeRenderBackend,
}

fn invalid_pair_error(
    window_backend: WindowBackendKind,
    render_backend: RenderBackendKind,
) -> GoudError {
    GoudError::InitializationFailed(format!(
        "invalid native backend pair: window={window_backend:?} render={render_backend:?}; supported pairs are Winit+Wgpu and GlfwLegacy+OpenGlLegacy"
    ))
}

/// Rejects unsupported mixed native backend pairs.
pub fn validate_native_backend_pair(
    window_backend: WindowBackendKind,
    render_backend: RenderBackendKind,
) -> GoudResult<()> {
    match (window_backend, render_backend) {
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => Ok(()),
        (WindowBackendKind::GlfwLegacy, RenderBackendKind::OpenGlLegacy) => Ok(()),
        _ => Err(invalid_pair_error(window_backend, render_backend)),
    }
}

/// Creates the configured native platform/backend pair.
pub fn create_native_runtime(
    window_config: &WindowConfig,
    window_backend: WindowBackendKind,
    render_backend: RenderBackendKind,
) -> GoudResult<NativeRuntime> {
    validate_native_backend_pair(window_backend, render_backend)?;

    match (window_backend, render_backend) {
        #[cfg(all(feature = "native", feature = "wgpu-backend"))]
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => {
            let platform = super::winit_platform::WinitPlatform::new(window_config)?;
            let framebuffer_size = platform.get_framebuffer_size();
            let mut backend = crate::libs::graphics::backend::wgpu_backend::WgpuBackend::new(
                platform.window().clone(),
            )?;
            backend.resize(framebuffer_size.0, framebuffer_size.1);
            Ok(NativeRuntime {
                platform: Box::new(platform),
                render_backend: SharedNativeRenderBackend::new(NativeRenderBackend::Wgpu(backend)),
            })
        }
        #[cfg(not(all(feature = "native", feature = "wgpu-backend")))]
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => {
            Err(GoudError::InitializationFailed(
                "wgpu support is not enabled in this build".to_string(),
            ))
        }
        #[cfg(feature = "legacy-glfw-opengl")]
        (WindowBackendKind::GlfwLegacy, RenderBackendKind::OpenGlLegacy) => {
            use crate::libs::graphics::backend::StateOps;

            let platform = super::glfw_platform::GlfwPlatform::new(window_config)?;
            let mut backend = crate::libs::graphics::backend::opengl::OpenGLBackend::new()?;
            backend.set_viewport(0, 0, window_config.width, window_config.height);
            Ok(NativeRuntime {
                platform: Box::new(platform),
                render_backend: SharedNativeRenderBackend::new(NativeRenderBackend::OpenGlLegacy(
                    backend,
                )),
            })
        }
        #[cfg(not(feature = "legacy-glfw-opengl"))]
        (WindowBackendKind::GlfwLegacy, RenderBackendKind::OpenGlLegacy) => {
            Err(GoudError::InitializationFailed(
                "legacy GLFW/OpenGL support is not enabled in this build".to_string(),
            ))
        }
        _ => Err(invalid_pair_error(window_backend, render_backend)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::game_config::GameConfig;

    #[test]
    fn validates_default_native_pair() {
        let config = GameConfig::default();
        assert_eq!(config.window_backend, WindowBackendKind::Winit);
        assert_eq!(config.render_backend, RenderBackendKind::Wgpu);
        assert!(validate_native_backend_pair(config.window_backend, config.render_backend).is_ok());
    }

    #[test]
    fn rejects_mixed_native_pair() {
        let error =
            validate_native_backend_pair(WindowBackendKind::Winit, RenderBackendKind::OpenGlLegacy)
                .expect_err("mixed native pair should fail");

        assert!(error.to_string().contains("invalid native backend pair"));
    }

    #[test]
    fn accepts_explicit_legacy_pair() {
        assert!(validate_native_backend_pair(
            WindowBackendKind::GlfwLegacy,
            RenderBackendKind::OpenGlLegacy,
        )
        .is_ok());
    }
}
