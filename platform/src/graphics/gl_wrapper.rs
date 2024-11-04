// gl_wrapper.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::os::raw::*;
use std::ptr;

use cgmath::*;
use gl::types::*;
use image::GenericImageView;

/// # Vertex Array Object
///
/// ## Example
/// ```
/// let vao = Vao::new();
/// vao.bind();
/// ```
pub struct Vao {
    id: GLuint,
}

impl Vao {
    pub fn new() -> Vao {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }

        Vao { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

/// # Buffer Object
/// An object for storing data
///
/// ## Example
/// ```
/// let vbo = BufferObject::new();
/// vbo.bind();
/// vbo.store_f32_data(&float32_array);
/// ```
pub struct BufferObject {
    id: GLuint,
    r#type: GLenum,
    usage: GLenum,
}

impl BufferObject {
    pub fn new() -> BufferObject {
        let r#type = gl::ARRAY_BUFFER;
        let usage = gl::STATIC_DRAW;
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        BufferObject { id, r#type, usage }
    }

    pub fn new_element_buffer() -> BufferObject {
        let r#type = gl::ELEMENT_ARRAY_BUFFER;
        let usage = gl::STATIC_DRAW;
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        BufferObject { id, r#type, usage }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.r#type, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.r#type, 0);
        }
    }

    pub fn store_f32_data(&self, data: &[f32]) {
        unsafe {
            gl::BufferData(
                self.r#type,
                (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &data[0] as *const f32 as *const c_void,
                self.usage,
            );
        }
    }

    pub fn store_u32_data(&self, data: &[u32]) {
        unsafe {
            gl::BufferData(
                self.r#type,
                (data.len() * mem::size_of::<u32>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                self.usage,
            );
        }
    }
}

/// # Vertex Attribute
/// Describes vertex data
///
/// ## Example
/// ```
/// let position_attribute = VertexAttribute::new(VertexAttributeProps { ... });
/// position_attribute.enable();
/// ```
pub struct VertexAttribute {
    index: GLuint,
}

pub struct VertexAttributeProps {
    pub index: u32,
    pub size: i32,
    pub stride: GLsizei,
    pub pointer: *const c_void,
}

impl VertexAttribute {
    pub fn new(props: VertexAttributeProps) -> VertexAttribute {
        let VertexAttributeProps {
            index,
            size,
            stride,
            pointer,
        } = props;

        let r#type = gl::FLOAT;
        let normalized = gl::FALSE;
        unsafe {
            gl::VertexAttribPointer(index, size, r#type, normalized, stride, pointer);
        }

        VertexAttribute { index }
    }

    pub fn enable(&self) {
        unsafe {
            gl::EnableVertexAttribArray(self.index);
        }
    }

    pub fn disable(&self) {
        unsafe {
            gl::DisableVertexAttribArray(self.index);
        }
    }
}

/// # Shader Program
/// ## Examples
/// ```
/// let program = ShaderProgram::new("/path/to/vertex_shader.glsl", "/path/to/fragment_shader.glsl");
/// program.bind();
///
/// program.create_uniform("transform");
///
/// program.set_matrix4fv_uniform("transform", some_matrix);
/// ```
pub struct ShaderProgram {
    program_handle: u32,
    uniform_ids: HashMap<String, GLint>,
}

impl ShaderProgram {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> ShaderProgram {
        let mut vertex_shader_file = File::open(vertex_shader_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", vertex_shader_path));
        let mut fragment_shader_file = File::open(fragment_shader_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", fragment_shader_path));

        let mut vertex_shader_source = String::new();
        let mut fragment_shader_source = String::new();

        vertex_shader_file
            .read_to_string(&mut vertex_shader_source)
            .expect("Failed to read vertex shader");

        fragment_shader_file
            .read_to_string(&mut fragment_shader_source)
            .expect("Failed to read fragment shader");

        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);

            // Check for compilation errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(
                    vertex_shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "Failed to compile vertex shader: {}",
                    String::from_utf8_lossy(&buffer)
                );
            }

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);

            // Check for compilation errors
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(
                    fragment_shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "Failed to compile fragment shader: {}",
                    String::from_utf8_lossy(&buffer)
                );
            }

            let program_handle = gl::CreateProgram();
            gl::AttachShader(program_handle, vertex_shader);
            gl::AttachShader(program_handle, fragment_shader);
            gl::LinkProgram(program_handle);

            // Check for linking errors
            gl::GetProgramiv(program_handle, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(program_handle, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1);
                gl::GetProgramInfoLog(
                    program_handle,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "Failed to link program: {}",
                    String::from_utf8_lossy(&buffer)
                );
            }

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            ShaderProgram {
                program_handle,
                uniform_ids: HashMap::new(),
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_handle);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn create_uniform(&mut self, uniform_name: &str) {
        let uniform_location = unsafe {
            gl::GetUniformLocation(
                self.program_handle,
                CString::new(uniform_name).unwrap().as_ptr(),
            )
        };
        if uniform_location < 0 {
            panic!("Cannot locate uniform: {}", uniform_name);
        } else {
            self.uniform_ids
                .insert(uniform_name.to_string(), uniform_location);
        }
    }

    pub fn set_matrix4fv_uniform(&self, uniform_name: &str, matrix: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(
                self.uniform_ids[uniform_name],
                1,
                gl::FALSE,
                matrix.as_ptr(),
            )
        }
    }

    pub fn set_int_uniform(&self, uniform_name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.uniform_ids[uniform_name], value)
        }
    }
}

/// # Texture
/// Represents an OpenGL texture object.
///
/// ## Example
/// ```
/// let texture = Texture::new("path/to/texture.png");
/// texture.bind(gl::TEXTURE0);
/// ```
pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(file_path: &str) -> Texture {
        let img = image::open(file_path).expect("Failed to load texture");
        let data = img.flipv().to_rgba8();
        let width = img.width();
        let height = img.height();

        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Texture { id }
    }

    pub fn bind(&self, unit: GLenum) {
        unsafe {
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

/// # Renderer
/// Handles rendering of sprites.
pub struct Renderer {
    shader_program: ShaderProgram,
    sprites: Vec<Sprite>,
}

pub struct Sprite {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    texture: Texture,
    indices_count: i32,
}

impl Renderer {
    pub fn new(
        // vertex_shader_path: &str, fragment_shader_path: &str
    ) -> Renderer {
        let vertex_shader_path = "platform/src/graphics/shaders/vertex_shader.glsl";
        let fragment_shader_path = "platform/src/graphics/shaders/fragment_shader.glsl";
        let shader_program = ShaderProgram::new(vertex_shader_path, fragment_shader_path);
        Renderer {
            shader_program,
            sprites: Vec::new(),
        }
    }

    pub fn add_sprite(&mut self, vertices: &[f32], indices: &[u32], texture_path: &str) -> usize {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new();
        vbo.bind();
        vbo.store_f32_data(vertices);

        let ebo = BufferObject::new_element_buffer();
        ebo.bind();
        ebo.store_u32_data(indices);

        let stride = 5 * mem::size_of::<f32>() as i32;

        let position_attribute = VertexAttribute::new(VertexAttributeProps {
            index: 0,
            size: 3,
            stride,
            pointer: ptr::null(),
        });
        position_attribute.enable();

        let tex_coord_attribute = VertexAttribute::new(VertexAttributeProps {
            index: 1,
            size: 2,
            stride,
            pointer: (3 * mem::size_of::<f32>()) as *const c_void,
        });
        tex_coord_attribute.enable();

        vao.unbind();
        vbo.unbind();
        ebo.unbind();

        let texture = Texture::new(texture_path);

        let sprite = Sprite {
            vao,
            vbo,
            ebo,
            texture,
            indices_count: indices.len() as i32,
        };

        self.sprites.push(sprite);
        self.sprites.len() - 1
    }

    pub fn update_sprite_position(&mut self, index: usize, position: Vector2<f32>) {
        let sprite = &mut self.sprites[index];
        let vertices = [
            position.x + 0.5,
            position.y + 0.5,
            0.0,
            1.0,
            1.0, // top right
            position.x + 0.5,
            position.y - 0.5,
            0.0,
            1.0,
            0.0, // bottom right
            position.x - 0.5,
            position.y - 0.5,
            0.0,
            0.0,
            0.0, // bottom left
            position.x - 0.5,
            position.y + 0.5,
            0.0,
            0.0,
            1.0, // top left
        ];

        sprite.vao.bind();
        sprite.vbo.bind();
        sprite.vbo.store_f32_data(&vertices);
    }

    pub fn render(&mut self) {
        self.shader_program.bind();

        self.shader_program.create_uniform("texture1");
        self.shader_program.set_int_uniform("texture1", 0);

        for sprite in &self.sprites {
            sprite.texture.bind(gl::TEXTURE0);

            sprite.vao.bind();
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    sprite.indices_count,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }
}