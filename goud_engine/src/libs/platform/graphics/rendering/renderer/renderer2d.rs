use cgmath::{ortho, Matrix4, Vector3, Vector4};
use gl::types::*;
use std::{ffi::c_void, ptr};

use crate::{
    libs::platform::graphics::rendering::{
        font::font_manager::FontManager, BufferObject, ShaderProgram, Vao, VertexAttribute,
    },
    types::{Sprite, SpriteMap, Text, TextMap, TextureManager},
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
    text_shader_program: Option<ShaderProgram>,
    text_vao: Option<Vao>,
    text_vbo: Option<GLuint>,
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
            // sprites: Vec::new(),
            model_uniform: "model".into(),
            source_rect_uniform: "sourceRect".into(),
            window_width,
            window_height,
            text_shader_program: None,
            text_vao: None,
            text_vbo: None,
        })
    }

    /// Renders all added sprites.
    fn render_sprites(
        &mut self,
        sprites: Vec<Sprite>,
        texture_manager: &TextureManager,
    ) -> Result<(), String> {
        self.shader_program.bind();
        self.vao.bind();

        for sprite in sprites {
            // Use positions and scales directly
            let position = Vector3::new(sprite.x, sprite.y, 0.0);
            let dimensions = Vector3::new(sprite.dimension_x, sprite.dimension_y, 1.0);
            let scale_x = sprite.scale_x;
            let scale_y = sprite.scale_y;
            let rotation = sprite.rotation;
            let texture = texture_manager.get_texture(sprite.texture_id).clone();

            // Calculate the center offset
            let center_offset = Vector3::new(dimensions.x * 0.5, dimensions.y * 0.5, 0.0);

            // Build the model matrix with center rotation
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

            // Bind texture
            texture.bind(gl::TEXTURE0);

            // Set the outline color to red and width to a small value for debugging
            if sprite.debug {
                self.shader_program
                    .set_uniform_vec4("outlineColor", &Vector4::new(1.0, 0.0, 0.0, 1.0))?;
                self.shader_program
                    .set_uniform_float("outlineWidth", 0.02)?; // Adjust width as needed
            }
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

            if sprite.debug {
                // clean up debug outline to ensure other sprites are not affected
                self.shader_program
                    .set_uniform_vec4("outlineColor", &Vector4::new(0.0, 0.0, 0.0, 0.0))?;
                self.shader_program.set_uniform_float("outlineWidth", 0.0)?;
            }
        }
        Ok(())
    }

    fn render_texts(&mut self, texts: Vec<Text>, font_manager: &FontManager) -> Result<(), String> {
        // Initialize text shader if not done
        if self.text_shader_program.is_none() {
            self.init_text_shader()?;
        }

        let shader_program = self.text_shader_program.as_mut().unwrap();
        shader_program.bind();

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.text_vao.as_ref().unwrap().bind();

        for text in texts {
            let font = font_manager
                .get_font(text.font_id)
                .ok_or("Font not found")?;

            // Set text color
            shader_program.set_uniform_vec3(
                "textColor",
                &Vector3::new(text.color.0, text.color.1, text.color.2),
            )?;

            let mut x = text.x;
            let y = text.y;

            for c in text.content.chars() {
                let ch = font
                    .characters
                    .get(&c)
                    .ok_or(format!("Character '{}' not found in font", c))?;
                let xpos = x + ch.bearing.0 as f32 * text.scale;
                let ypos = y - (ch.size.1 as i32 - ch.bearing.1) as f32 * text.scale;

                let w = ch.size.0 as f32 * text.scale;
                let h = ch.size.1 as f32 * text.scale;

                // Update VBO for each character
                let vertices: [f32; 6 * 4] = [
                    xpos,
                    ypos + h,
                    0.0,
                    0.0,
                    xpos,
                    ypos,
                    0.0,
                    1.0,
                    xpos + w,
                    ypos,
                    1.0,
                    1.0,
                    xpos,
                    ypos + h,
                    0.0,
                    0.0,
                    xpos + w,
                    ypos,
                    1.0,
                    1.0,
                    xpos + w,
                    ypos + h,
                    1.0,
                    0.0,
                ];

                // Render quad
                unsafe {
                    // Update content of VBO memory
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.text_vbo.unwrap());
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                        vertices.as_ptr() as *const c_void,
                    );
                }

                // Bind texture
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
                }

                // Draw quad
                unsafe {
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                }

                // Advance cursors for next glyph
                x += (ch.advance as f32) * text.scale;
            }
        }

        // Unbind
        Vao::unbind();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::Disable(gl::BLEND);
        }

        Ok(())
    }

    fn init_text_shader(&mut self) -> Result<(), String> {
        let mut shader_program = ShaderProgram::new_text_shader()?;
        shader_program.bind();
        shader_program.create_uniform("projection")?;
        shader_program.create_uniform("textColor")?;
        shader_program.create_uniform("text")?;
        shader_program.set_uniform_int("text", 0)?;

        let projection = ortho(
            0.0,
            self.window_width as f32,
            self.window_height as f32,
            0.0,
            -1.0,
            1.0,
        );
        shader_program.set_uniform_mat4("projection", &projection)?;

        let vao = Vao::new()?;
        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        unsafe {
            gl::BindVertexArray(vao.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (6 * 4 * std::mem::size_of::<f32>()) as GLsizeiptr,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            let stride = 4 * std::mem::size_of::<f32>() as GLsizei;

            VertexAttribute::enable(0);
            VertexAttribute::pointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0);

            VertexAttribute::enable(1);
            VertexAttribute::pointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                2 * std::mem::size_of::<f32>(),
            );
        }

        self.text_shader_program = Some(shader_program);
        self.text_vao = Some(vao);
        self.text_vbo = Some(vbo.id);

        Ok(())
    }
}

impl Renderer for Renderer2D {
    /// Renders the 2D scene.
    fn render(
        &mut self,
        sprites: SpriteMap,
        texts: TextMap,
        texture_manager: &TextureManager,
        font_manager: &FontManager,
    ) {
        let sprites: Vec<Sprite> = sprites.into_iter().flat_map(|(_, v)| v).collect();
        if let Err(e) = self.render_sprites(sprites, texture_manager) {
            eprintln!("Error rendering sprites: {}", e);
        }

        let texts: Vec<Text> = texts.into_iter().flat_map(|(_, v)| v).collect();

        // debug texts
        if let Err(e) = self.render_texts(texts, font_manager) {
            eprintln!("Error rendering texts: {}", e);
        }
    }

    fn terminate(&self) {
        self.shader_program.terminate();
        self.vao.terminate();
        if let Some(shader_program) = &self.text_shader_program {
            shader_program.terminate();
        }
        if let Some(vao) = &self.text_vao {
            vao.terminate();
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
