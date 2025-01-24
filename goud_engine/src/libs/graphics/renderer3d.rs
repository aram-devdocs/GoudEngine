use cgmath::{perspective, Matrix4, Point3, Vector3};
use gl::types::*;

use super::components::buffer::BufferObject;
use super::components::shader::ShaderProgram;
use super::components::vao::Vao;
use super::components::vertex_attribute::VertexAttribute;
use super::renderer::Renderer;
use crate::types::{SpriteMap, TextureManager};

#[repr(C)]
#[derive(Debug)]
pub struct Renderer3D {
    shader_program: ShaderProgram,
    vao: Vao,
    camera_position: Vector3<f32>,
    camera_zoom: f32,
    window_width: u32,
    window_height: u32,
    light_position: Vector3<f32>,
    light_color: Vector3<f32>,
}

impl Renderer3D {
    pub fn new(window_width: u32, window_height: u32) -> Result<Renderer3D, String> {
        // Initialize shader program
        let mut shader_program = ShaderProgram::new_3d()?;

        // Create VAO and VBO for cube
        let vao = Vao::new()?;
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        vbo.bind();
        vbo.store_data(&CUBE_VERTICES, gl::STATIC_DRAW);

        // Define vertex attributes
        let stride = 8 * std::mem::size_of::<f32>() as GLsizei;

        // Position attribute
        VertexAttribute::enable(0);
        VertexAttribute::pointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0);

        // Normal attribute
        VertexAttribute::enable(1);
        VertexAttribute::pointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            3 * std::mem::size_of::<f32>(),
        );

        // Texture coordinate attribute
        VertexAttribute::enable(2);
        VertexAttribute::pointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            6 * std::mem::size_of::<f32>(),
        );

        Vao::unbind();
        BufferObject::unbind(gl::ARRAY_BUFFER);

        // Set up uniforms
        shader_program.bind();
        shader_program.create_uniform("model")?;
        shader_program.create_uniform("view")?;
        shader_program.create_uniform("projection")?;
        shader_program.create_uniform("texture1")?;
        shader_program.create_uniform("lightPos")?;
        shader_program.create_uniform("viewPos")?;
        shader_program.create_uniform("lightColor")?;
        shader_program.create_uniform("ambientStrength")?;
        shader_program.create_uniform("specularStrength")?;

        shader_program.set_uniform_int("texture1", 0)?;

        // Create perspective projection matrix
        let aspect_ratio = window_width as f32 / window_height as f32;
        let projection = perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 100.0);
        shader_program.set_uniform_mat4("projection", &projection)?;

        Ok(Renderer3D {
            shader_program,
            vao,
            camera_position: Vector3::new(0.0, 0.0, 3.0),
            camera_zoom: 1.0,
            window_width,
            window_height,
            light_position: Vector3::new(2.0, 2.0, 2.0),
            light_color: Vector3::new(1.0, 1.0, 1.0),
        })
    }

    fn render_cube(&mut self, texture_manager: &TextureManager) -> Result<(), String> {
        self.shader_program.bind();
        self.vao.bind();

        // Create view matrix
        let view = Matrix4::look_at_rh(
            Point3::new(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_zoom, // Use zoom as Z coordinate
            ),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        // Set uniforms
        self.shader_program.set_uniform_mat4("view", &view)?;
        self.shader_program.set_uniform_vec3("lightPos", &self.light_position)?;
        self.shader_program.set_uniform_vec3("viewPos", &self.camera_position)?;
        self.shader_program.set_uniform_vec3("lightColor", &self.light_color)?;
        self.shader_program.set_uniform_float("ambientStrength", 0.3)?; // Increased ambient light
        self.shader_program.set_uniform_float("specularStrength", 0.7)?; // Increased specular

        // Create model matrix with rotation
        let model = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0))
            * Matrix4::from_angle_x(cgmath::Deg(self.camera_position.x * 45.0))
            * Matrix4::from_angle_y(cgmath::Deg(self.camera_position.y * 45.0));

        self.shader_program.set_uniform_mat4("model", &model)?;

        // Bind texture
        if let Some(texture) = texture_manager.textures.values().next() {
            texture.bind(gl::TEXTURE0);
        }

        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 36); // 36 vertices for a cube
        }

        Ok(())
    }
}

impl Renderer for Renderer3D {
    fn render(&mut self, _sprites: SpriteMap, texture_manager: &TextureManager) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        if let Err(e) = self.render_cube(texture_manager) {
            eprintln!("Error rendering cube: {}", e);
        }
    }

    fn set_camera_position(&mut self, x: f32, y: f32) {
        self.camera_position.x = x;
        self.camera_position.y = y;
    }

    fn set_camera_zoom(&mut self, zoom: f32) {
        self.camera_zoom = zoom;
    }

    fn terminate(&self) {
        self.shader_program.terminate();
        self.vao.terminate();
    }
}

// Cube vertices with positions, normals, and texture coordinates
const CUBE_VERTICES: [f32; 288] = [
    // positions          // normals           // texture coords
    -0.5, -0.5, -0.5,    0.0,  0.0, -1.0,    0.0, 0.0,
     0.5, -0.5, -0.5,    0.0,  0.0, -1.0,    1.0, 0.0,
     0.5,  0.5, -0.5,    0.0,  0.0, -1.0,    1.0, 1.0,
     0.5,  0.5, -0.5,    0.0,  0.0, -1.0,    1.0, 1.0,
    -0.5,  0.5, -0.5,    0.0,  0.0, -1.0,    0.0, 1.0,
    -0.5, -0.5, -0.5,    0.0,  0.0, -1.0,    0.0, 0.0,

    -0.5, -0.5,  0.5,    0.0,  0.0,  1.0,    0.0, 0.0,
     0.5, -0.5,  0.5,    0.0,  0.0,  1.0,    1.0, 0.0,
     0.5,  0.5,  0.5,    0.0,  0.0,  1.0,    1.0, 1.0,
     0.5,  0.5,  0.5,    0.0,  0.0,  1.0,    1.0, 1.0,
    -0.5,  0.5,  0.5,    0.0,  0.0,  1.0,    0.0, 1.0,
    -0.5, -0.5,  0.5,    0.0,  0.0,  1.0,    0.0, 0.0,

    -0.5,  0.5,  0.5,   -1.0,  0.0,  0.0,    1.0, 0.0,
    -0.5,  0.5, -0.5,   -1.0,  0.0,  0.0,    1.0, 1.0,
    -0.5, -0.5, -0.5,   -1.0,  0.0,  0.0,    0.0, 1.0,
    -0.5, -0.5, -0.5,   -1.0,  0.0,  0.0,    0.0, 1.0,
    -0.5, -0.5,  0.5,   -1.0,  0.0,  0.0,    0.0, 0.0,
    -0.5,  0.5,  0.5,   -1.0,  0.0,  0.0,    1.0, 0.0,

     0.5,  0.5,  0.5,    1.0,  0.0,  0.0,    1.0, 0.0,
     0.5,  0.5, -0.5,    1.0,  0.0,  0.0,    1.0, 1.0,
     0.5, -0.5, -0.5,    1.0,  0.0,  0.0,    0.0, 1.0,
     0.5, -0.5, -0.5,    1.0,  0.0,  0.0,    0.0, 1.0,
     0.5, -0.5,  0.5,    1.0,  0.0,  0.0,    0.0, 0.0,
     0.5,  0.5,  0.5,    1.0,  0.0,  0.0,    1.0, 0.0,

    -0.5, -0.5, -0.5,    0.0, -1.0,  0.0,    0.0, 1.0,
     0.5, -0.5, -0.5,    0.0, -1.0,  0.0,    1.0, 1.0,
     0.5, -0.5,  0.5,    0.0, -1.0,  0.0,    1.0, 0.0,
     0.5, -0.5,  0.5,    0.0, -1.0,  0.0,    1.0, 0.0,
    -0.5, -0.5,  0.5,    0.0, -1.0,  0.0,    0.0, 0.0,
    -0.5, -0.5, -0.5,    0.0, -1.0,  0.0,    0.0, 1.0,

    -0.5,  0.5, -0.5,    0.0,  1.0,  0.0,    0.0, 1.0,
     0.5,  0.5, -0.5,    0.0,  1.0,  0.0,    1.0, 1.0,
     0.5,  0.5,  0.5,    0.0,  1.0,  0.0,    1.0, 0.0,
     0.5,  0.5,  0.5,    0.0,  1.0,  0.0,    1.0, 0.0,
    -0.5,  0.5,  0.5,    0.0,  1.0,  0.0,    0.0, 0.0,
    -0.5,  0.5, -0.5,    0.0,  1.0,  0.0,    0.0, 1.0
];
