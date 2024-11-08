// src/vao.rs

use gl::types::*;
use std::ptr;

#[derive(Debug)]
pub struct Vao {
    id: GLuint,
}

impl Vao {
    /// Creates a new Vertex Array Object.
    pub fn new() -> Result<Vao, String> {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
            if id == 0 {
                return Err("Failed to generate VAO".into());
            }
        }
        Ok(Vao { id })
    }

    /// Binds the VAO.
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    /// Unbinds any VAO.
    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}