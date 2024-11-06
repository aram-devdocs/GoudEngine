// gl_wrapper.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::mem;
use std::os::raw::*;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use cgmath::*;
use gl::types::*;
// use image::GenericImageView;

/// Vertex Array Object (VAO)
///
/// Manages the vertex array state.
///
/// # Example
/// ```
/// let vao = Vao::new();
/// vao.bind();
/// ```
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

/// Clears the screen.
pub fn clear() {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

/// Buffer Object (VBO/EBO)
///
/// Manages buffer data storage.
///
/// # Example
/// ```
/// let vbo = BufferObject::new(gl::ARRAY_BUFFER);
/// vbo.bind();
/// vbo.store_data(&data, gl::STATIC_DRAW);
/// ```
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
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                usage,
            );
        }
    }
}

/// Vertex Attribute Pointer
///
/// Describes vertex attribute data.
///
/// # Example
/// ```
/// VertexAttribute::enable(0);
/// VertexAttribute::pointer(0, 3, gl::FLOAT, false, stride, offset);
/// ```
pub struct VertexAttribute;

impl VertexAttribute {
    /// Enables a vertex attribute array.
    pub fn enable(index: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }

    /// Disables a vertex attribute array.
    // pub fn disable(index: GLuint) {
    //     unsafe {
    //         gl::DisableVertexAttribArray(index);
    //     }
    // }

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

/// Shader Program
///
/// Manages shader compilation and linking.
///
/// # Example
/// ```
/// let program = ShaderProgram::new("vertex.glsl", "fragment.glsl")?;
/// program.bind();
/// ```
#[derive(Debug)]
pub struct ShaderProgram {
    id: GLuint,
    uniform_locations: HashMap<String, GLint>,
}

impl ShaderProgram {
    /// Creates a new Shader Program from vertex and fragment shader files.
    pub fn new(vertex_path: &str, fragment_path: &str) -> Result<ShaderProgram, String> {
        let vertex_code = fs::read_to_string(vertex_path)
            .map_err(|_| format!("Failed to read vertex shader from {}", vertex_path))?;
        let fragment_code = fs::read_to_string(fragment_path)
            .map_err(|_| format!("Failed to read fragment shader from {}", fragment_path))?;

        let vertex_shader = ShaderProgram::compile_shader(&vertex_code, gl::VERTEX_SHADER)?;
        let fragment_shader = ShaderProgram::compile_shader(&fragment_code, gl::FRAGMENT_SHADER)?;

        let id = ShaderProgram::link_program(vertex_shader, fragment_shader)?;

        // Clean up shaders as they're linked into our program now and no longer necessary
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        Ok(ShaderProgram {
            id,
            uniform_locations: HashMap::new(),
        })
    }

    /// Compiles a shader from source code.
    fn compile_shader(source: &str, shader_type: GLenum) -> Result<GLuint, String> {
        let shader;
        unsafe {
            shader = gl::CreateShader(shader_type);
            let c_str = CString::new(source.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Check for compilation errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1); // Skip null terminator
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                let error = String::from_utf8_lossy(&buffer).into_owned();
                return Err(error);
            }
        }

        Ok(shader)
    }

    /// Links vertex and fragment shaders into a shader program.
    fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
        let program;
        unsafe {
            program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            // Check for linking errors
            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len((len as usize) - 1); // Skip null terminator
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                let error = String::from_utf8_lossy(&buffer).into_owned();
                return Err(error);
            }
        }
        Ok(program)
    }

    /// Activates the shader program.
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Deactivates any shader program.
    // pub fn unbind() {
    //     unsafe {
    //         gl::UseProgram(0);
    //     }
    // }

    /// Creates a uniform variable.
    pub fn create_uniform(&mut self, name: &str) -> Result<(), String> {
        let c_name = CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr()) };
        if location < 0 {
            return Err(format!("Uniform '{}' not found", name));
        }
        self.uniform_locations.insert(name.into(), location);
        Ok(())
    }

    /// Sets an integer uniform variable.
    pub fn set_uniform_int(&self, name: &str, value: GLint) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::Uniform1i(location, value);
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }

    /// Sets a 4x4 matrix uniform variable.
    pub fn set_uniform_mat4(&self, name: &str, matrix: &Matrix4<f32>) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }

    /// Sets a vec4 uniform variable.
    pub fn set_uniform_vec4(&self, name: &str, vector: &Vector4<f32>) -> Result<(), String> {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                gl::Uniform4f(location, vector.x, vector.y, vector.z, vector.w);
            }
            Ok(())
        } else {
            Err(format!("Uniform '{}' not found", name))
        }
    }
}

/// Texture
///
/// Manages texture loading and binding.
///
/// # Example
/// ```
/// let texture = Texture::new("path/to/texture.png")?;
/// texture.bind(gl::TEXTURE0);
/// ```
#[derive(Debug, Clone)]
pub struct Texture {
    id: GLuint,
    width: u32,
    height: u32,
}

impl Texture {
    /// Loads a texture from a file.
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Rc<Texture>, String> {
        let img = image::open(file_path.as_ref())
            .map_err(|_| format!("Failed to load texture from {:?}", file_path.as_ref()))?;
        let data = img.flipv().to_rgba8();
        let width = img.width();
        let height = img.height();

        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            if id == 0 {
                return Err("Failed to generate texture ID".into());
            }
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

        Ok(Rc::new(Texture { id, width, height }))
    }

    /// Binds the texture to a texture unit.
    pub fn bind(&self, unit: GLenum) {
        unsafe {
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    /// Returns the width of the texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the texture.
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Rectangle
///
/// Represents a rectangle, typically used for spritesheet source rectangles.
#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Base Renderer trait
///
/// Defines common functionality for renderers.
pub trait Renderer {
    /// Renders the scene.
    fn render(&mut self);
}

/// Renderer2D
///
/// A renderer specialized for 2D rendering.
///
/// # Example
/// ```
/// let mut renderer = Renderer2D::new()?;
/// let texture = Texture::new("spritesheet.png")?;
/// let sprite = Sprite::new(texture, position, scale, rotation, Some(source_rect));
/// renderer.add_sprite(sprite);
/// ```
#[derive(Debug)]
pub struct Renderer2D {
    shader_program: ShaderProgram,
    vao: Vao,
    // vbo: BufferObject,
    // ebo: BufferObject,
    pub sprites: Vec<Sprite>,
    model_uniform: String,
    source_rect_uniform: String,
}

impl Renderer2D {
    /// Creates a new Renderer2D.
    pub fn new(window_width: u32, window_height: u32) -> Result<Renderer2D, String> {
        // Initialize shader program

        // TODO: https://github.com/aram-devdocs/GoudEngine/issues/10
        let mut shader_program =
            ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl")?;

        // Create VAO, VBO, and EBO
        let vao = Vao::new()?;
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        vbo.bind();
        vbo.store_data(&QUAD_VERTICES, gl::STATIC_DRAW);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER)?;
        ebo.bind();
        ebo.store_data(&QUAD_INDICES, gl::STATIC_DRAW);

        // Define vertex attributes
        let stride = 5 * mem::size_of::<f32>() as GLsizei;

        VertexAttribute::enable(0);
        VertexAttribute::pointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0);

        VertexAttribute::enable(1);
        VertexAttribute::pointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            3 * mem::size_of::<f32>(),
        );

        Vao::unbind();
        BufferObject::unbind(gl::ARRAY_BUFFER);
        BufferObject::unbind(gl::ELEMENT_ARRAY_BUFFER);

        // Set up uniforms
        shader_program.bind();
        shader_program.create_uniform("model")?;
        shader_program.create_uniform("projection")?;
        shader_program.create_uniform("texture1")?;
        shader_program.create_uniform("sourceRect")?;
        shader_program.set_uniform_int("texture1", 0)?;

        // Create projection matrix
        use cgmath::{ortho, Matrix4};
        let projection = ortho(
            0.0,
            window_width as f32,
            window_height as f32,
            0.0,
            -1.0,
            1.0,
        );

        // Set the projection matrix
        shader_program.set_uniform_mat4("projection", &projection)?;

        Ok(Renderer2D {
            shader_program,
            vao,
            // vbo,
            // ebo,
            sprites: Vec::new(),
            model_uniform: "model".into(),
            source_rect_uniform: "sourceRect".into(),
        })
    }

    /// Adds a sprite to be rendered.
    pub fn add_sprite(&mut self, sprite: Sprite) -> usize {
        self.sprites.push(sprite);
        self.sprites.len() - 1
    }

    /// Updates a sprite at a given index.
    pub fn update_sprite(&mut self, index: usize, sprite: Sprite) -> Result<(), String> {
        if index < self.sprites.len() {
            self.sprites[index] = sprite;
            Ok(())
        } else {
            Err("Sprite index out of bounds".into())
        }
    }

    /// Renders all added sprites.
    fn render_sprites(&mut self) -> Result<(), String> {
        self.shader_program.bind();
        self.vao.bind();

        for sprite in &self.sprites {
            // Use positions and scales directly
            let position = Vector3::new(sprite.position.x, sprite.position.y, 0.0);
            let dimmensions = Vector3::new(sprite.dimmensions.x, sprite.dimmensions.y, 1.0);

            // Build the model matrix
            let model = Matrix4::from_translation(position)
                * Matrix4::from_nonuniform_scale(dimmensions.x, dimmensions.y, dimmensions.z);

            self.shader_program
                .set_uniform_mat4(&self.model_uniform, &model)?;

            // Bind texture
            sprite.texture.bind(gl::TEXTURE0);

            // Set source rectangle
            let source_rect = sprite.source_rect.unwrap_or(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            });
            self.shader_program.set_uniform_vec4(
                &self.source_rect_uniform,
                &Vector4::new(
                    source_rect.x,
                    source_rect.y,
                    source_rect.width,
                    source_rect.height,
                ),
            )?;

            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    QUAD_INDICES.len() as GLsizei,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
        Ok(())
    }
}

impl Renderer for Renderer2D {
    /// Renders the 2D scene.
    fn render(&mut self) {
        if let Err(e) = self.render_sprites() {
            eprintln!("Error rendering sprites: {}", e);
        }
    }
}

/// Sprite
///
/// Represents a 2D sprite with position, scale, rotation, and optional source rectangle.
#[derive(Debug, Clone)]
pub struct Sprite {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub dimmensions: Vector2<f32>,
    pub rotation: f32,
    pub texture: Rc<Texture>,
    pub source_rect: Option<Rectangle>,
}

impl Sprite {
    /// Creates a new Sprite.
    pub fn new(
        texture: Rc<Texture>,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        dimmensions: Vector2<f32>,
        rotation: f32,
        source_rect: Option<Rectangle>,
    ) -> Sprite {
        Sprite {
            position,
            scale,
            dimmensions,
            rotation,
            texture,
            source_rect,
        }
    }
}

// Constants for quad vertices and indices
const QUAD_VERTICES: [f32; 20] = [
    // positions    // texture coords
    1.0, 1.0, 0.0, 1.0, 1.0, // top right
    1.0, 0.0, 0.0, 1.0, 0.0, // bottom right
    0.0, 0.0, 0.0, 0.0, 0.0, // bottom left
    0.0, 1.0, 0.0, 0.0, 1.0, // top left
];
const QUAD_INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

/// Renderer3D
///
/// A placeholder for a 3D renderer that inherits from the base Renderer trait.
#[derive(Debug)]
pub struct Renderer3D {
    // Implementation details for 3D rendering
}

impl Renderer3D {
    // Creates a new Renderer3D.
    // pub fn new() -> Result<Renderer3D, String> {
    //     // Initialize shaders, buffers, etc.
    //     Ok(Renderer3D {
    //         // Initialization
    //     })
    // }

    // Additional methods for 3D rendering
}

impl Renderer for Renderer3D {
    /// Renders the 3D scene.
    fn render(&mut self) {
        // Implement 3D rendering logic
    }
}
