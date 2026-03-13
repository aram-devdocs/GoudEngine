use crate::core::providers::types::DebugShape3D;

const SHAPE_SPHERE: u32 = 0;
const SHAPE_BOX: u32 = 1;
const SHAPE_LINE: u32 = 2;

pub(super) fn shape_to_line_vertices(shape: &DebugShape3D) -> Vec<f32> {
    match shape.shape_type {
        SHAPE_LINE => line_vertices(shape),
        SHAPE_BOX => box_vertices(shape),
        SHAPE_SPHERE => Vec::new(),
        _ => Vec::new(),
    }
}

pub(super) fn build_debug_draw_vertices(shapes: &[DebugShape3D]) -> Vec<f32> {
    let mut vertices = Vec::new();
    for shape in shapes {
        vertices.extend(shape_to_line_vertices(shape));
    }
    vertices
}

fn line_vertices(shape: &DebugShape3D) -> Vec<f32> {
    let half_length = shape.size[0] * 0.5;
    let start =
        rotate_point_by_quaternion([-half_length, 0.0, 0.0], shape.rotation, shape.position);
    let end = rotate_point_by_quaternion([half_length, 0.0, 0.0], shape.rotation, shape.position);

    let mut vertices = Vec::with_capacity(12);
    push_line(
        &mut vertices,
        start,
        end,
        [shape.color[0], shape.color[1], shape.color[2]],
    );
    vertices
}

fn box_vertices(shape: &DebugShape3D) -> Vec<f32> {
    let hx = shape.size[0];
    let hy = shape.size[1];
    let hz = shape.size[2];

    let local_corners = [
        [-hx, -hy, -hz], // 0
        [hx, -hy, -hz],  // 1
        [hx, hy, -hz],   // 2
        [-hx, hy, -hz],  // 3
        [-hx, -hy, hz],  // 4
        [hx, -hy, hz],   // 5
        [hx, hy, hz],    // 6
        [-hx, hy, hz],   // 7
    ];

    let corners = local_corners
        .map(|corner| rotate_point_by_quaternion(corner, shape.rotation, shape.position));

    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let mut vertices = Vec::with_capacity(144);
    let color = [shape.color[0], shape.color[1], shape.color[2]];
    for (a, b) in edges {
        push_line(&mut vertices, corners[a], corners[b], color);
    }
    vertices
}

fn push_line(vertices: &mut Vec<f32>, start: [f32; 3], end: [f32; 3], color: [f32; 3]) {
    vertices.extend_from_slice(&[start[0], start[1], start[2], color[0], color[1], color[2]]);
    vertices.extend_from_slice(&[end[0], end[1], end[2], color[0], color[1], color[2]]);
}

fn rotate_point_by_quaternion(local: [f32; 3], rotation: [f32; 4], center: [f32; 3]) -> [f32; 3] {
    let qx = rotation[0];
    let qy = rotation[1];
    let qz = rotation[2];
    let qw = rotation[3];

    let uv = [
        qy * local[2] - qz * local[1],
        qz * local[0] - qx * local[2],
        qx * local[1] - qy * local[0],
    ];
    let uuv = [
        qy * uv[2] - qz * uv[1],
        qz * uv[0] - qx * uv[2],
        qx * uv[1] - qy * uv[0],
    ];

    let rotated = [
        local[0] + 2.0 * (qw * uv[0] + uuv[0]),
        local[1] + 2.0 * (qw * uv[1] + uuv[1]),
        local[2] + 2.0 * (qw * uv[2] + uuv[2]),
    ];

    [
        center[0] + rotated[0],
        center[1] + rotated[1],
        center[2] + rotated[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_shape_generates_two_colored_vertices() {
        let shape = DebugShape3D {
            shape_type: SHAPE_LINE,
            position: [1.0, 2.0, 3.0],
            size: [4.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [0.2, 0.4, 0.6, 1.0],
        };
        let vertices = shape_to_line_vertices(&shape);
        assert_eq!(vertices.len(), 12);
        assert_eq!(vertices[3], 0.2);
        assert_eq!(vertices[4], 0.4);
        assert_eq!(vertices[5], 0.6);
    }

    #[test]
    fn box_shape_generates_twelve_line_segments() {
        let shape = DebugShape3D {
            shape_type: SHAPE_BOX,
            position: [0.0, 0.0, 0.0],
            size: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        };
        let vertices = shape_to_line_vertices(&shape);
        assert_eq!(vertices.len(), 144);
        assert_eq!(vertices.chunks_exact(6).len(), 24);
    }

    #[test]
    fn multiple_shapes_append_line_vertices() {
        let line = DebugShape3D {
            shape_type: SHAPE_LINE,
            position: [0.0, 0.0, 0.0],
            size: [2.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let bx = DebugShape3D {
            shape_type: SHAPE_BOX,
            position: [0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        };

        let vertices = build_debug_draw_vertices(&[line, bx]);
        assert_eq!(vertices.len(), 12 + 144);
        assert_eq!(vertices.chunks_exact(6).len(), 26);
    }
}
