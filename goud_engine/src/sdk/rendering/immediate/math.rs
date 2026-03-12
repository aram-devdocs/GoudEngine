/// Creates an orthographic projection matrix.
pub(crate) fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
    let near = -1.0f32;
    let far = 1.0f32;
    [
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (far - near),
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        -(far + near) / (far - near),
        1.0,
    ]
}

/// Creates a model matrix for sprite transformation.
pub(crate) fn model_matrix(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> [f32; 16] {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    [
        width * cos_r,
        width * sin_r,
        0.0,
        0.0,
        -height * sin_r,
        height * cos_r,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        x,
        y,
        0.0,
        1.0,
    ]
}
