//! Thread-local renderer state and accessor helpers for the 3D renderer FFI.

use crate::core::error::GoudError;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::{with_window_state, WindowState};
use crate::libs::graphics::renderer3d::Renderer3D;
use std::collections::HashMap;

thread_local! {
    pub(super) static RENDERER3D_STATE: std::cell::RefCell<HashMap<(u32, u32), Renderer3D>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Ensures 3D renderer state is initialized for a context.
pub(super) fn ensure_renderer3d_state(context_id: GoudContextId) -> Result<(), GoudError> {
    let context_key = (context_id.index(), context_id.generation());

    let already_initialized =
        RENDERER3D_STATE.with(|cell| cell.borrow().contains_key(&context_key));

    if already_initialized {
        return Ok(());
    }

    // Get window dimensions
    let (width, height) =
        with_window_state(context_id, |ws: &mut WindowState| ws.get_framebuffer_size())
            .ok_or(GoudError::InvalidContext)?;

    let backend = Box::new(
        with_window_state(context_id, |ws: &mut WindowState| ws.backend.clone())
            .ok_or(GoudError::InvalidContext)?,
    );
    let renderer =
        Renderer3D::new(backend, width, height).map_err(GoudError::InitializationFailed)?;

    RENDERER3D_STATE.with(|cell| {
        cell.borrow_mut().insert(context_key, renderer);
    });

    Ok(())
}

/// Calls `f` with a mutable reference to the renderer for `context_id`, if it exists.
pub(super) fn with_renderer<F, R>(context_id: GoudContextId, f: F) -> Option<R>
where
    F: FnOnce(&mut Renderer3D) -> R,
{
    let context_key = (context_id.index(), context_id.generation());
    RENDERER3D_STATE.with(|cell| {
        let mut states = cell.borrow_mut();
        states.get_mut(&context_key).map(f)
    })
}
