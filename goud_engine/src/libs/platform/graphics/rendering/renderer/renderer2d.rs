use cgmath::{ortho, Matrix4, Vector3, Vector4};
use gl::types::*;
use std::ptr;

use crate::{
    libs::platform::graphics::rendering::{BufferObject, ShaderProgram, Vao, VertexAttribute},
    types::{Rectangle, Sprite, SpriteMap},
};

use super::Renderer;

#[repr(C)]
#[derive(Debug)]
pub struct Renderer2D {
    shader_program: ShaderProgram,
    vao: Vao,
    model_uniform: String,
    source_rect_uniform: String,
    window_width: u32,
    window_height: u32,
}

impl Renderer2D {
    /// Creates a new Renderer2D.
    pub fn new(window_width: u32, window_height: u32) -> Result<Renderer2D, String> {
        // Initialize shader program
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
        let stride = 5 * std::mem::size_of::<f32>() as GLsizei;

        VertexAttribute::enable(0);
        VertexAttribute::pointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0);

        VertexAttribute::enable(1);
        VertexAttribute::pointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            3 * std::mem::size_of::<f32>(),
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
            // sprites: Vec::new(),
            model_uniform: "model".into(),
            source_rect_uniform: "sourceRect".into(),
            window_width,
            window_height,
        })
    }

    /// Renders all added sprites.
    fn render_sprites(&mut self, sprites: Vec<Sprite>) -> Result<(), String> {
        self.shader_program.bind();
        self.vao.bind();

        for sprite in sprites {
            // Use positions and scales directly
            let position = Vector3::new(sprite.x, sprite.y, 0.0);
            let dimensions = Vector3::new(
                if sprite.dimension_x == 0.0 {
                    sprite.texture.width() as f32
                } else {
                    sprite.dimension_x
                },
                if sprite.dimension_y == 0.0 {
                    sprite.texture.height() as f32
                } else {
                    sprite.dimension_y
                },
                1.0,
            );
            let scale_x = sprite.scale_x;
            let scale_y = sprite.scale_y;
            let rotation = sprite.rotation;

            // Build the model matrix
            let model = Matrix4::from_translation(position)
                * Matrix4::from_angle_z(cgmath::Deg(rotation))
                * Matrix4::from_nonuniform_scale(
                    dimensions.x * scale_x,
                    dimensions.y * scale_y,
                    dimensions.z,
                );

            self.shader_program
                .set_uniform_mat4(&self.model_uniform, &model)?;

            // Bind texture
            sprite.texture.bind(gl::TEXTURE0);

            // Set source rectangle
            let source_rect = sprite.source_rect;
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
    fn render(&mut self, sprites: SpriteMap) {
        let sprites: Vec<Sprite> = sprites.into_iter().filter_map(|s| s).collect();
        if let Err(e) = self.render_sprites(sprites) {
            eprintln!("Error rendering sprites: {}", e);
        }
    }

    fn terminate(&self) {
        self.shader_program.terminate();
        self.vao.terminate();
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
