use crate::core::error::GoudError;
use crate::core::providers::types::DebugShape;
use crate::ffi::context::GoudContextId;
use crate::ffi::window::WindowState;

use super::super::immediate::ImmediateStateData;
use super::helpers::prepare_draw_state;
use super::internal::draw_quad_rotated_internal;

/// Renders physics debug wireframes for the current context.
pub(crate) fn render_physics_debug_overlay(
    context_id: GoudContextId,
    window_state: &mut WindowState,
) -> Result<(), GoudError> {
    if !window_state.physics_debug_enabled {
        return Ok(());
    }

    let shapes = crate::ffi::providers::physics_debug_shapes(context_id);
    if shapes.is_empty() {
        return Ok(());
    }

    let state_data = prepare_draw_state(context_id)?;

    for shape in shapes {
        draw_debug_shape(window_state, state_data, &shape)?;
    }

    Ok(())
}

fn draw_debug_shape(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    shape: &DebugShape,
) -> Result<(), GoudError> {
    match shape.shape_type {
        0 => draw_circle_outline(window_state, state_data, shape),
        1 => draw_box_outline(window_state, state_data, shape),
        2 => draw_line_shape(window_state, state_data, shape),
        _ => Ok(()),
    }
}

fn draw_box_outline(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    shape: &DebugShape,
) -> Result<(), GoudError> {
    let half_w = shape.size[0] * 0.5;
    let half_h = shape.size[1] * 0.5;
    let corners = [
        rotate_point([-half_w, -half_h], shape.rotation, shape.position),
        rotate_point([half_w, -half_h], shape.rotation, shape.position),
        rotate_point([half_w, half_h], shape.rotation, shape.position),
        rotate_point([-half_w, half_h], shape.rotation, shape.position),
    ];

    for segment in corners.windows(2) {
        draw_line_segment(
            window_state,
            state_data,
            segment[0],
            segment[1],
            shape.color,
            2.0,
        )?;
    }

    draw_line_segment(
        window_state,
        state_data,
        corners[3],
        corners[0],
        shape.color,
        2.0,
    )
}

fn draw_circle_outline(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    shape: &DebugShape,
) -> Result<(), GoudError> {
    let radius = shape.size[0].max(shape.size[1]).max(0.5);
    let segments = 24;
    let mut previous = [
        shape.position[0] + radius * shape.rotation.cos(),
        shape.position[1] + radius * shape.rotation.sin(),
    ];

    for index in 1..=segments {
        let theta = shape.rotation + (index as f32 / segments as f32) * std::f32::consts::TAU;
        let current = [
            shape.position[0] + radius * theta.cos(),
            shape.position[1] + radius * theta.sin(),
        ];
        draw_line_segment(
            window_state,
            state_data,
            previous,
            current,
            shape.color,
            2.0,
        )?;
        previous = current;
    }

    Ok(())
}

fn draw_line_shape(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    shape: &DebugShape,
) -> Result<(), GoudError> {
    let half_length = shape.size[0] * 0.5;
    let start = rotate_point([-half_length, 0.0], shape.rotation, shape.position);
    let end = rotate_point([half_length, 0.0], shape.rotation, shape.position);
    draw_line_segment(
        window_state,
        state_data,
        start,
        end,
        shape.color,
        shape.size[1].max(2.0),
    )
}

fn draw_line_segment(
    window_state: &mut WindowState,
    state_data: ImmediateStateData,
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    thickness: f32,
) -> Result<(), GoudError> {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return Ok(());
    }

    draw_quad_rotated_internal(
        window_state,
        state_data,
        (start[0] + end[0]) * 0.5,
        (start[1] + end[1]) * 0.5,
        length,
        thickness,
        dy.atan2(dx),
        color[0],
        color[1],
        color[2],
        color[3],
    )
}

fn rotate_point(local: [f32; 2], rotation: f32, center: [f32; 2]) -> [f32; 2] {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    [
        center[0] + local[0] * cos_r - local[1] * sin_r,
        center[1] + local[0] * sin_r + local[1] * cos_r,
    ]
}
