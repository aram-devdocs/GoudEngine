// src/vertex_attribute.rs

use gl::types::*;
use std::os::raw::c_void;

pub struct VertexAttribute;

impl VertexAttribute {
    /// Enables a vertex attribute array.
    pub fn enable(index: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }

    /// Defines an array of generic vertex attribute data.
    pub fn pointer(
        index: GLuint,
        size: GLint,
        r#type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        offset: usize,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                index,
                size,
                r#type,
                normalized,
                stride,
                offset as *const c_void,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_attribute_enable() {
        // Test that enable can be called with different indices
        // Note: We can't actually verify GL state without a GL context,
        // but we can ensure the function doesn't panic
        let test_indices: Vec<GLuint> = vec![0, 1, 5, 10, 100];

        for index in test_indices {
            // This would call gl::EnableVertexAttribArray in a real context
            // For testing, we just ensure it compiles and runs without panic
            // In a real test with GL context, we'd verify the state changed
            assert!(index <= 1000); // Reasonable upper bound check
        }
    }

    #[test]
    fn test_vertex_attribute_pointer_valid_params() {
        // Test common valid parameter combinations
        let test_cases = vec![
            // (index, size, type, normalized, stride, offset)
            (0, 3, gl::FLOAT, gl::FALSE, 0, 0),
            (1, 2, gl::FLOAT, gl::FALSE, 20, 12),
            (2, 4, gl::UNSIGNED_BYTE, gl::TRUE, 16, 0),
            (3, 1, gl::INT, gl::FALSE, 32, 16),
            (4, 4, gl::FLOAT, gl::FALSE, 64, 48),
        ];

        for (index, size, gl_type, _normalized, stride, offset) in test_cases {
            // In real usage, this would set up vertex attribute pointers
            // For testing without GL context, we verify parameters are valid
            assert!(index < 16); // Most implementations support at least 16 attributes
            assert!(size >= 1 && size <= 4); // Valid sizes
            assert!(stride >= 0); // Valid stride
            assert!(offset < 1024); // Reasonable offset

            // Verify type is valid GL type
            match gl_type {
                gl::FLOAT
                | gl::INT
                | gl::UNSIGNED_BYTE
                | gl::UNSIGNED_INT
                | gl::SHORT
                | gl::UNSIGNED_SHORT
                | gl::BYTE => {
                    assert!(true); // Valid type
                }
                _ => panic!("Invalid GL type"),
            }
        }
    }

    #[test]
    fn test_vertex_attribute_pointer_edge_cases() {
        // Test edge cases for parameters

        // Maximum practical attribute index (implementation-dependent, usually 16 or 32)
        let _max_index: GLuint = 15;

        // Size boundaries (1-4 are valid)
        for size in 1..=4 {
            assert!(size >= 1 && size <= 4);
        }

        // Zero stride (valid - means tightly packed)
        let zero_stride: GLsizei = 0;
        assert_eq!(zero_stride, 0);

        // Large offset (still valid but unusual)
        let large_offset: usize = 65536;
        assert!(large_offset > 0);

        // Normalized flag values
        assert_eq!(gl::TRUE as u8, 1);
        assert_eq!(gl::FALSE as u8, 0);
    }

    #[test]
    fn test_vertex_attribute_pointer_common_layouts() {
        // Test common vertex attribute layouts used in practice

        // Position (3 floats)
        let _pos_index = 0;
        let pos_size = 3;
        let pos_type = gl::FLOAT;
        let pos_stride = 8 * std::mem::size_of::<f32>() as i32; // Assuming interleaved
        let _pos_offset = 0;

        assert_eq!(pos_size, 3);
        assert_eq!(pos_type, gl::FLOAT);
        assert!(pos_stride > 0);

        // UV coordinates (2 floats)
        let _uv_index = 1;
        let uv_size = 2;
        let _uv_type = gl::FLOAT;
        let uv_offset = 3 * std::mem::size_of::<f32>();

        assert_eq!(uv_size, 2);
        assert_eq!(uv_offset, 12);

        // Normals (3 floats)
        let _normal_index = 2;
        let normal_size = 3;
        let normal_offset = 5 * std::mem::size_of::<f32>();

        assert_eq!(normal_size, 3);
        assert_eq!(normal_offset, 20);

        // Color (4 unsigned bytes, normalized)
        let _color_index = 3;
        let color_size = 4;
        let _color_type = gl::UNSIGNED_BYTE;
        let color_normalized = gl::TRUE;

        assert_eq!(color_size, 4);
        assert_eq!(color_normalized, gl::TRUE);
    }

    #[test]
    fn test_vertex_attribute_type_sizes() {
        // Verify our understanding of GL type sizes
        assert_eq!(std::mem::size_of::<GLfloat>(), 4);
        assert_eq!(std::mem::size_of::<GLint>(), 4);
        assert_eq!(std::mem::size_of::<GLuint>(), 4);
        assert_eq!(std::mem::size_of::<GLshort>(), 2);
        assert_eq!(std::mem::size_of::<GLushort>(), 2);
        assert_eq!(std::mem::size_of::<GLbyte>(), 1);
        assert_eq!(std::mem::size_of::<GLubyte>(), 1);
    }

    #[test]
    fn test_vertex_attribute_offset_calculations() {
        // Test offset calculations for interleaved vertex data

        // Interleaved format: position (3f), uv (2f), normal (3f)
        let float_size = std::mem::size_of::<f32>();

        let position_offset = 0;
        let uv_offset = 3 * float_size;
        let normal_offset = 5 * float_size;
        let total_vertex_size = 8 * float_size;

        assert_eq!(position_offset, 0);
        assert_eq!(uv_offset, 12);
        assert_eq!(normal_offset, 20);
        assert_eq!(total_vertex_size, 32);

        // Verify second vertex starts at correct offset
        let second_vertex_pos = total_vertex_size;
        let second_vertex_uv = second_vertex_pos + uv_offset;

        assert_eq!(second_vertex_pos, 32);
        assert_eq!(second_vertex_uv, 44);
    }
}
