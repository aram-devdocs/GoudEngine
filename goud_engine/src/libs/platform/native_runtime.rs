//! Native runtime factory for valid window/render backend pairs.

use crate::core::error::{GoudError, GoudResult};
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use crate::libs::graphics::backend::native_backend::NativeRenderBackend;
use crate::libs::graphics::backend::native_backend::SharedNativeRenderBackend;

use super::{PlatformBackend, RenderBackendKind, WindowBackendKind, WindowConfig};

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
        "invalid native backend pair: window={window_backend:?} render={render_backend:?}; supported pairs are Winit+Wgpu, GlfwLegacy+OpenGlLegacy, and XboxGdk+Wgpu"
    ))
}

/// Detects the best available backend pair at runtime based on compiled features.
///
/// Tries wgpu (Winit + Wgpu) first, then falls back to OpenGL (GLFW + OpenGL).
#[allow(unreachable_code)]
pub fn detect_best_backend() -> (WindowBackendKind, RenderBackendKind) {
    #[cfg(all(feature = "xbox-gdk", target_env = "msvc"))]
    {
        return (WindowBackendKind::XboxGdk, RenderBackendKind::Wgpu);
    }
    #[cfg(all(
        feature = "native",
        feature = "wgpu-backend",
        any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows",
            target_os = "android",
            target_os = "ios"
        )
    ))]
    {
        return (WindowBackendKind::Winit, RenderBackendKind::Wgpu);
    }
    #[cfg(feature = "legacy-glfw-opengl")]
    {
        return (
            WindowBackendKind::GlfwLegacy,
            RenderBackendKind::OpenGlLegacy,
        );
    }
    (WindowBackendKind::Winit, RenderBackendKind::Wgpu)
}

/// Creates a native runtime using auto-detection for the best available backend.
pub fn create_native_runtime_auto(window_config: &WindowConfig) -> GoudResult<NativeRuntime> {
    let (window_backend, render_backend) = detect_best_backend();
    create_native_runtime(window_config, window_backend, render_backend)
}

/// Rejects unsupported mixed native backend pairs.
pub fn validate_native_backend_pair(
    window_backend: WindowBackendKind,
    render_backend: RenderBackendKind,
) -> GoudResult<()> {
    match (window_backend, render_backend) {
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => Ok(()),
        (WindowBackendKind::GlfwLegacy, RenderBackendKind::OpenGlLegacy) => Ok(()),
        #[cfg(feature = "xbox-gdk")]
        (WindowBackendKind::XboxGdk, RenderBackendKind::Wgpu) => Ok(()),
        (_, RenderBackendKind::Auto) => Ok(()), // Auto is resolved before validation
        _ => Err(invalid_pair_error(window_backend, render_backend)),
    }
}

/// Creates the configured native platform/backend pair.
pub fn create_native_runtime(
    window_config: &WindowConfig,
    window_backend: WindowBackendKind,
    render_backend: RenderBackendKind,
) -> GoudResult<NativeRuntime> {
    // Auto-detect resolves to the best available pair.
    if render_backend == RenderBackendKind::Auto {
        return create_native_runtime_auto(window_config);
    }

    validate_native_backend_pair(window_backend, render_backend)?;

    match (window_backend, render_backend) {
        #[cfg(all(
            feature = "native",
            feature = "wgpu-backend",
            any(
                target_os = "linux",
                target_os = "macos",
                target_os = "windows",
                target_os = "android",
                target_os = "ios"
            )
        ))]
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => {
            let platform = super::winit_platform::WinitPlatform::new(window_config)?;
            let framebuffer_size = platform.get_framebuffer_size();
            let mut backend = crate::libs::graphics::backend::wgpu_backend::WgpuBackend::new(
                platform.window().clone(),
                window_config.vsync,
            )?;
            backend.resize(framebuffer_size.0, framebuffer_size.1);
            Ok(NativeRuntime {
                platform: Box::new(platform),
                render_backend: SharedNativeRenderBackend::new(NativeRenderBackend::Wgpu(
                    Box::new(backend),
                )),
            })
        }
        #[cfg(not(all(
            feature = "native",
            feature = "wgpu-backend",
            any(
                target_os = "linux",
                target_os = "macos",
                target_os = "windows",
                target_os = "android",
                target_os = "ios"
            )
        )))]
        (WindowBackendKind::Winit, RenderBackendKind::Wgpu) => {
            Err(GoudError::InitializationFailed(
                "wgpu native runtime is only available on desktop targets in this build"
                    .to_string(),
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
                    Box::new(backend),
                )),
            })
        }
        #[cfg(not(feature = "legacy-glfw-opengl"))]
        (WindowBackendKind::GlfwLegacy, RenderBackendKind::OpenGlLegacy) => {
            Err(GoudError::InitializationFailed(
                "legacy GLFW/OpenGL support is not enabled in this build".to_string(),
            ))
        }
        #[cfg(feature = "xbox-gdk")]
        (WindowBackendKind::XboxGdk, RenderBackendKind::Wgpu) => {
            let platform = super::xbox_gdk_platform::XboxGdkPlatform::new(window_config)?;
            let handle = platform.window_handle();
            let (w, h) = platform.get_framebuffer_size();
            let mut backend =
                crate::libs::graphics::backend::wgpu_backend::WgpuBackend::new_from_raw_handle(
                    handle, w, h, true,
                )?;
            backend.resize(w, h);
            Ok(NativeRuntime {
                platform: Box::new(platform),
                render_backend: SharedNativeRenderBackend::new(NativeRenderBackend::Wgpu(
                    Box::new(backend),
                )),
            })
        }
        _ => Err(invalid_pair_error(window_backend, render_backend)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_default_native_pair() {
        assert_eq!(WindowBackendKind::default(), WindowBackendKind::Winit);
        assert_eq!(RenderBackendKind::default(), RenderBackendKind::Wgpu);
        assert!(validate_native_backend_pair(
            WindowBackendKind::default(),
            RenderBackendKind::default()
        )
        .is_ok());
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

    #[test]
    fn detect_best_backend_returns_valid_pair() {
        let (window, render) = detect_best_backend();
        assert!(validate_native_backend_pair(window, render).is_ok());
    }

    #[test]
    fn auto_backend_passes_validation() {
        assert!(
            validate_native_backend_pair(WindowBackendKind::Winit, RenderBackendKind::Auto).is_ok()
        );
    }

    #[cfg(feature = "xbox-gdk")]
    #[test]
    fn accepts_xbox_gdk_pair() {
        assert!(
            validate_native_backend_pair(WindowBackendKind::XboxGdk, RenderBackendKind::Wgpu,)
                .is_ok()
        );
    }
}
