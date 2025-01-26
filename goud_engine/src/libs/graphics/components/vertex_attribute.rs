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
