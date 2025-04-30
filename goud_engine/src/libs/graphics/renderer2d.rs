use cgmath::{ortho, Matrix4, Vector3, Vector4};
use gl::types::*;
use std::ptr;

use super::components::buffer::BufferObject;
use super::components::camera::Camera;
use super::components::shader::ShaderProgram;
use super::components::vao::Vao;
use super::components::vertex_attribute::VertexAttribute;
use crate::types::{Camera2D, Rectangle, Sprite, SpriteMap, TextureManager};

use super::renderer::Renderer;

#[repr(C)]
#[derive(Debug)]
pub struct Renderer2D {
    shader_program: ShaderProgram,
    vao: Vao,
    model_uniform: String,
    source_rect_uniform: String,
    window_width: u32,
    window_height: u32,
    camera: Camera2D,
}

impl Renderer2D {
    /// Creates a new Renderer2D.
    pub fn new(window_width: u32, window_height: u32) -> Result<Renderer2D, String> {
        // Initialize shader program
        let mut shader_program = ShaderProgram::new()?;

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
        shader_program.create_uniform("view")?;
        shader_program.create_uniform("texture1")?;
        shader_program.create_uniform("sourceRect")?;
        shader_program.set_uniform_int("texture1", 0)?;

        // Set up DEBUG outline uniforms
        shader_program.create_uniform("outlineColor")?;
        shader_program.create_uniform("outlineWidth")?;

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
            model_uniform: "model".into(),
            source_rect_uniform: "sourceRect".into(),
            window_width,
            window_height,
            camera: Camera2D::new(),
        })
    }

    /// Sets the camera position.
    pub fn set_camera_position(&mut self, x: f32, y: f32) {
        self.camera.set_position_xy(x, y);
    }

    /// Sets the camera zoom level.
    pub fn set_camera_zoom(&mut self, zoom: f32) {
        self.camera.set_zoom(zoom);
    }

    /// Gets the camera position.
    pub fn get_camera_position(&self) -> cgmath::Vector3<f32> {
        self.camera.get_position()
    }

    /// Gets the camera zoom level.
    pub fn get_camera_zoom(&self) -> f32 {
        self.camera.get_zoom()
    }

    /// Renders all added sprites.
    fn render_sprites(
        &mut self,
        sprites: Vec<Sprite>,
        texture_manager: &TextureManager,
    ) -> Result<(), String> {
        self.shader_program.bind();
        self.vao.bind();

        // Get the view matrix from the camera
        let view = self.camera.get_view_matrix();

        // Set the view matrix
        self.shader_program.set_uniform_mat4("view", &view)?;

        for sprite in sprites {
            let position = Vector3::new(sprite.x, sprite.y, 0.0);
            let mut scale_x = sprite.scale_x;
            let mut scale_y = sprite.scale_y;
            let rotation = sprite.rotation;
            let texture = texture_manager.get_texture(sprite.texture_id).clone();

            // Normalize frame coordinates and dimensions based on texture size
            let mut source_rect = if sprite.frame.width > 0.0 && sprite.frame.height > 0.0 {
                Rectangle {
                    x: sprite.frame.x / texture.width as f32,
                    y: (texture.height as f32 - sprite.frame.y - sprite.frame.height)
                        / texture.height as f32,
                    width: sprite.frame.width / texture.width as f32,
                    height: sprite.frame.height / texture.height as f32,
                }
            } else {
                Rectangle {
                    x: sprite.source_rect.x,
                    y: 1.0 - sprite.source_rect.y - sprite.source_rect.height,
                    width: sprite.source_rect.width,
                    height: sprite.source_rect.height,
                }
            };
            // Adjust the source rectangle if scale_x or scale_y is negative
            if scale_x < 0.0 {
                scale_x = -scale_x;
                source_rect.x += source_rect.width;
                source_rect.width = -source_rect.width;
            }

            if scale_y < 0.0 {
                scale_y = -scale_y;
                source_rect.y += source_rect.height;
                source_rect.height = -source_rect.height;
            }

            self.shader_program.set_uniform_vec4(
                &self.source_rect_uniform,
                &Vector4::new(
                    source_rect.x,
                    source_rect.y,
                    source_rect.width,
                    source_rect.height,
                ),
            )?;

            let dimensions = if sprite.frame.width > 0.0 && sprite.frame.height > 0.0 {
                Vector3::new(sprite.frame.width, sprite.frame.height, 1.0)
            } else {
                Vector3::new(sprite.dimension_x, sprite.dimension_y, 1.0)
            };

            let center_offset = Vector3::new(dimensions.x * 0.5, dimensions.y * 0.5, 0.0);

            let model = Matrix4::from_translation(position)
                * Matrix4::from_translation(center_offset)
                * Matrix4::from_angle_z(cgmath::Deg(rotation))
                * Matrix4::from_translation(-center_offset)
                * Matrix4::from_nonuniform_scale(
                    dimensions.x * scale_x,
                    dimensions.y * scale_y,
                    dimensions.z,
                );
            self.shader_program
                .set_uniform_mat4(&self.model_uniform, &model)?;

            texture.bind(gl::TEXTURE0);

            if sprite.debug {
                self.shader_program
                    .set_uniform_vec4("outlineColor", &Vector4::new(1.0, 0.0, 0.0, 1.0))?;
                self.shader_program
                    .set_uniform_float("outlineWidth", 0.02)?;
            }

            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    QUAD_INDICES.len() as GLsizei,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }

            if sprite.debug {
                self.shader_program
                    .set_uniform_vec4("outlineColor", &Vector4::new(0.0, 0.0, 0.0, 0.0))?;
                self.shader_program.set_uniform_float("outlineWidth", 0.0)?;
            }
        }
        Ok(())
    }
}

impl Renderer for Renderer2D {
    /// Renders the 2D scene.
    fn render(&mut self, sprites: SpriteMap, texture_manager: &TextureManager) {
        let sprites: Vec<Sprite> = sprites.into_iter().flat_map(|(_, v)| v).collect();
        if let Err(e) = self.render_sprites(sprites, texture_manager) {
            eprintln!("Error rendering sprites: {}", e);
        }
    }

    // fn set_camera_position(&mut self, x: f32, y: f32) {
    //     self.set_camera_position(x, y);
    // }

    // fn set_camera_zoom(&mut self, zoom: f32) {
    //     self.set_camera_zoom(zoom);
    // }

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
