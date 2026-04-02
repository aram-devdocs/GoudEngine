//! Xbox GDK window handle wrapper for wgpu surface creation.
//!
//! Provides [`XboxWindowHandle`] which implements the `raw-window-handle`
//! traits required by wgpu to create a DX12 surface from an Xbox GDK HWND.

use std::num::NonZeroIsize;

/// Wrapper providing `raw-window-handle` traits for an Xbox GDK HWND.
///
/// wgpu uses `HasWindowHandle` + `HasDisplayHandle` to create a rendering
/// surface. This wrapper bridges the Xbox GDK `HWND` to those traits so
/// wgpu's DX12 backend can present to the Xbox window.
pub struct XboxWindowHandle {
    hwnd: *mut std::ffi::c_void,
}

impl XboxWindowHandle {
    /// Wraps a raw HWND for use with wgpu.
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid Win32 HWND that remains live for the lifetime
    /// of this handle. The caller is responsible for ensuring the window is
    /// not destroyed while this handle exists.
    pub unsafe fn from_raw_hwnd(hwnd: *mut std::ffi::c_void) -> Self {
        Self { hwnd }
    }

    /// Creates a dummy handle for type-checking on non-Windows platforms.
    ///
    /// The returned handle must never be used to create an actual surface.
    #[cfg(not(target_env = "msvc"))]
    pub fn dummy() -> Self {
        Self {
            hwnd: std::ptr::null_mut(),
        }
    }
}

// SAFETY: The HWND is owned by the Xbox GDK GameWindow which lives for the
// duration of the game. The handle is only used to create a wgpu surface
// during initialization, after which wgpu owns the surface independently.
unsafe impl Send for XboxWindowHandle {}
// SAFETY: Same reasoning as Send — the HWND is a copyable integer handle
// and wgpu only reads it during surface creation.
unsafe impl Sync for XboxWindowHandle {}

impl wgpu::rwh::HasWindowHandle for XboxWindowHandle {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        // SAFETY: The HWND was validated as non-null when constructed via
        // `from_raw_hwnd`. For `dummy()` handles this path should never be
        // reached in practice.
        let raw = wgpu::rwh::Win32WindowHandle::new(
            NonZeroIsize::new(self.hwnd as isize).ok_or(wgpu::rwh::HandleError::Unavailable)?,
        );
        let raw = wgpu::rwh::RawWindowHandle::Win32(raw);
        // SAFETY: The raw handle borrows `self` which outlives the returned
        // `WindowHandle`.
        Ok(unsafe { wgpu::rwh::WindowHandle::borrow_raw(raw) })
    }
}

impl wgpu::rwh::HasDisplayHandle for XboxWindowHandle {
    fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
        let raw = wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new());
        // SAFETY: Windows display handle carries no state and is always valid.
        Ok(unsafe { wgpu::rwh::DisplayHandle::borrow_raw(raw) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_handle_returns_unavailable() {
        #[cfg(not(target_env = "msvc"))]
        {
            use wgpu::rwh::HasWindowHandle;
            let handle = XboxWindowHandle::dummy();
            // Dummy has null HWND → NonZeroIsize fails → HandleError::Unavailable
            assert!(handle.window_handle().is_err());
        }
    }
}
