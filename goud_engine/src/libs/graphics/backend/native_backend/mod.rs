//! Shared native render backend wrappers.

mod native_draw;
mod native_impls;
mod native_resources;
mod shared_core;
mod shared_draw;
mod shared_resources;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::libs::error::GoudResult;

use super::capabilities::BackendInfo;
#[cfg(feature = "legacy-glfw-opengl")]
use super::opengl::OpenGLBackend;
use super::render_backend::RenderBackend;
#[cfg(feature = "legacy-glfw-opengl")]
use super::render_backend::StateOps;
#[cfg(any(
    all(feature = "native", feature = "wgpu-backend"),
    feature = "xbox-gdk",
    feature = "sdl-window",
    feature = "switch-vulkan"
))]
use super::wgpu_backend::WgpuBackend;

/// Concrete native render backend choice.
pub enum NativeRenderBackend {
    #[cfg(feature = "legacy-glfw-opengl")]
    /// Legacy OpenGL backend selected through the explicit legacy feature gate.
    OpenGlLegacy(Box<OpenGLBackend>),
    #[cfg(any(
        all(feature = "native", feature = "wgpu-backend"),
        feature = "xbox-gdk"
    ))]
    /// Default wgpu backend used by the native runtime.
    Wgpu(Box<WgpuBackend>),
}

impl NativeRenderBackend {
    fn info(&self) -> &BackendInfo {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.info(),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.info(),
        }
    }

    pub(crate) fn info_clone(&self) -> BackendInfo {
        self.info().clone()
    }

    pub(crate) fn bind_texture_by_index(&mut self, index: u32, unit: u32) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.bind_texture_by_index(index, unit),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.bind_texture_by_index(index, unit),
        }
    }

    pub(crate) fn resize_surface(&mut self, width: u32, height: u32) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(backend) => backend.set_viewport(0, 0, width, height),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.resize(width, height),
        }
    }

    /// Drops the GPU surface for mobile suspend. No-op on OpenGL.
    pub(crate) fn drop_surface(&mut self) {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(_) => {}
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.drop_surface(),
        }
    }

    /// Recreates the GPU surface after mobile resume. No-op on OpenGL.
    pub(crate) fn recreate_surface(&mut self) -> GoudResult<()> {
        match self {
            #[cfg(feature = "legacy-glfw-opengl")]
            Self::OpenGlLegacy(_) => Ok(()),
            #[cfg(any(
                all(feature = "native", feature = "wgpu-backend"),
                feature = "xbox-gdk"
            ))]
            Self::Wgpu(backend) => backend.recreate_surface(),
        }
    }
}

/// Cloneable native render backend handle backed by shared state.
#[derive(Clone)]
pub struct SharedNativeRenderBackend {
    inner: Arc<Mutex<NativeRenderBackend>>,
    info: BackendInfo,
}

impl SharedNativeRenderBackend {
    /// Wraps a concrete native backend in shared ownership for SDK and FFI runtime state.
    pub fn new(backend: NativeRenderBackend) -> Self {
        let info = backend.info_clone();
        Self {
            inner: Arc::new(Mutex::new(backend)),
            info,
        }
    }

    fn lock(&self) -> MutexGuard<'_, NativeRenderBackend> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn bind_texture_by_index(&self, index: u32, unit: u32) -> GoudResult<()> {
        self.lock().bind_texture_by_index(index, unit)
    }

    pub(crate) fn resize_surface(&self, width: u32, height: u32) {
        self.lock().resize_surface(width, height);
    }

    /// Drops the GPU surface for mobile suspend.
    pub(crate) fn drop_surface(&self) {
        self.lock().drop_surface();
    }

    /// Recreates the GPU surface after mobile resume.
    pub(crate) fn recreate_surface(&self) -> GoudResult<()> {
        self.lock().recreate_surface()
    }
}
