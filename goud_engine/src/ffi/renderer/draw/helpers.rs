use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use super::super::immediate::{ensure_immediate_state, ImmediateStateData, IMMEDIATE_STATE};
use super::super::texture::{GoudTextureHandle, GOUD_INVALID_TEXTURE};

pub(super) fn prepare_draw_state(
    context_id: GoudContextId,
) -> Result<ImmediateStateData, GoudError> {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return Err(GoudError::InvalidContext);
    }

    ensure_immediate_state(context_id)?;
    extract_state(context_id).ok_or(GoudError::InvalidContext)
}

pub(super) fn prepare_textured_draw_state(
    context_id: GoudContextId,
    texture: GoudTextureHandle,
) -> Result<ImmediateStateData, GoudError> {
    if texture == GOUD_INVALID_TEXTURE {
        return Err(GoudError::InvalidHandle);
    }

    prepare_draw_state(context_id)
}

pub(super) fn map_draw_result(result: Option<Result<(), GoudError>>) -> bool {
    match result {
        Some(Ok(())) => true,
        Some(Err(e)) => {
            set_last_error(e);
            false
        }
        None => {
            set_last_error(GoudError::InvalidContext);
            false
        }
    }
}

fn extract_state(context_id: GoudContextId) -> Option<ImmediateStateData> {
    let context_key = (context_id.index(), context_id.generation());
    IMMEDIATE_STATE.with(|cell| {
        let states = cell.borrow();
        states.get(&context_key).map(|s| {
            (
                s.shader,
                s.vertex_buffer,
                s.index_buffer,
                s.vao,
                s.u_projection,
                s.u_model,
                s.u_color,
                s.u_use_texture,
                s.u_texture,
                s.u_uv_offset,
                s.u_uv_scale,
            )
        })
    })
}
