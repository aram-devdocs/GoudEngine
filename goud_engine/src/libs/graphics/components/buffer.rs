use gl;
use gl::types::*;

#[derive(Debug)]
pub struct BufferObject {
    id: GLuint,
    buffer_type: GLenum,
}

impl BufferObject {
    /// Creates a new Buffer Object of a specified type.
    pub fn new(buffer_type: GLenum) -> Result<BufferObject, String> {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            if id == 0 {
                return Err("Failed to generate Buffer Object".into());
            }
        }
        Ok(BufferObject { id, buffer_type })
    }

    /// Binds the buffer.
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type, self.id);
        }
    }

    /// Unbinds any buffer of the same type.
    pub fn unbind(buffer_type: GLenum) {
        unsafe {
            gl::BindBuffer(buffer_type, 0);
        }
    }

    /// Stores data in the buffer.
    pub fn store_data<T>(&self, data: &[T], usage: GLenum) {
        unsafe {
            gl::BufferData(
                self.buffer_type,
                std::mem::size_of_val(data) as GLsizeiptr,
                data.as_ptr() as *const _,
                usage,
            );
        }
    }
}

impl Drop for BufferObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
