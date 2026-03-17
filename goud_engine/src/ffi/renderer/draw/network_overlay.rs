use crate::core::error::GoudError;
use crate::ffi::context::GoudContextId;
use crate::ffi::network::network_overlay_snapshot_for_context;
use crate::ffi::window::WindowState;
use crate::sdk::network_debug_overlay::format_overlay_lines;

use super::super::immediate::ImmediateStateData;
use super::helpers::prepare_draw_state;
use super::internal::draw_quad_rotated_internal;

const CHAR_SCALE: f32 = 2.0;
const CHAR_ADVANCE: f32 = 12.0;
const LINE_HEIGHT: f32 = 18.0;
const PANEL_WIDTH: f32 = 280.0;
const PANEL_MARGIN: f32 = 12.0;

/// Renders network debug metrics for the current context.
pub(crate) fn render_network_debug_overlay(
    context_id: GoudContextId,
    window_state: &mut WindowState,
) -> Result<(), GoudError> {
    if !window_state.network_overlay.is_visible() {
        return Ok(());
    }

    let Some(snapshot) = network_overlay_snapshot_for_context(context_id) else {
        window_state.network_overlay.set_active_handle(None);
        return Ok(());
    };

    window_state
        .network_overlay
        .set_active_handle(Some(snapshot.handle));

    let lines = format_overlay_lines(snapshot.handle, snapshot.metrics);
    let state_data = prepare_draw_state(context_id)?;
    let (_, fb_height) = window_state.get_framebuffer_size();

    let panel_height = PANEL_MARGIN + (lines.len() as f32 * LINE_HEIGHT) + 6.0;
    draw_panel_background(
        window_state,
        state_data.clone(),
        fb_height as f32,
        panel_height,
    )?;

    let mut y = fb_height as f32 - PANEL_MARGIN - 18.0;
    for line in &lines {
        draw_embedded_text(
            window_state,
            state_data.clone(),
            PANEL_MARGIN + 8.0,
            y,
            line,
            [0.20, 0.95, 0.35, 0.95],
        )?;
        y -= LINE_HEIGHT;
    }

    Ok(())
}

fn draw_panel_background(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    fb_height: f32,
    panel_height: f32,
) -> Result<(), GoudError> {
    draw_quad_rotated_internal(
        window_state,
        state_data.clone(),
        PANEL_MARGIN + PANEL_WIDTH * 0.5,
        fb_height - PANEL_MARGIN - panel_height * 0.5,
        PANEL_WIDTH,
        panel_height,
        0.0,
        0.0,
        0.0,
        0.0,
        0.62,
    )
}

/// Draws fixed-size glyphs from an embedded 5x7 bitmap font.
fn draw_embedded_text(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    mut x: f32,
    y: f32,
    text: &str,
    color: [f32; 4],
) -> Result<(), GoudError> {
    for ch in text.chars() {
        draw_embedded_char(
            window_state,
            state_data.clone(),
            x,
            y,
            ch.to_ascii_uppercase(),
            color,
        )?;
        x += CHAR_ADVANCE;
    }
    Ok(())
}

fn draw_embedded_char(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    x: f32,
    y: f32,
    ch: char,
    color: [f32; 4],
) -> Result<(), GoudError> {
    let rows = glyph_rows(ch);
    for (row_index, row_bits) in rows.iter().enumerate() {
        for col in 0..5 {
            if (row_bits & (1 << (4 - col))) == 0 {
                continue;
            }

            draw_quad_rotated_internal(
                window_state,
                state_data.clone(),
                x + (col as f32 * CHAR_SCALE) + (CHAR_SCALE * 0.5),
                y - (row_index as f32 * CHAR_SCALE) - (CHAR_SCALE * 0.5),
                CHAR_SCALE,
                CHAR_SCALE,
                0.0,
                color[0],
                color[1],
                color[2],
                color[3],
            )?;
        }
    }

    Ok(())
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x1C],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        ' ' => [0x00; 7],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x12, 0x11, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        _ => [0x00; 7],
    }
}
