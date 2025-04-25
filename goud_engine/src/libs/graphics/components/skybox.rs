use gl::types::*;
use std::mem;

use super::buffer::BufferObject;
use super::shader::ShaderProgram;
use super::vao::Vao;
use super::vertex_attribute::VertexAttribute;
use crate::types::SkyboxConfig;

#[derive(Debug)]
pub struct Skybox {
    vao: Vao,
    shader: ShaderProgram,
    texture_id: GLuint,
    config: SkyboxConfig,
}

impl Skybox {
    pub fn new() -> Result<Self, String> {
        let config = SkyboxConfig::default();

        // Scale up the skybox to ensure it encompasses the entire scene
        let scale = config.size;
        let vertices: [f32; 108] = [
            // positions
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
            1.0 * scale,
            -1.0 * scale,
            1.0 * scale,
        ];

        let vao = Vao::new()?;
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        vbo.bind();
        vbo.store_data(&vertices, gl::STATIC_DRAW);

        // vertex positions
        VertexAttribute::enable(0);
        VertexAttribute::pointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<f32>() as GLsizei,
            0,
        );

        // Create shader program
        let shader = ShaderProgram::new_skybox()?;

        // Generate texture
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);

            // Generate a default gradient skybox
            for i in 0..6 {
                let size = config.texture_size; // Use configured texture size
                let mut data = vec![0u8; size as usize * size as usize * 3];

                // Create a gradient pattern using the configured colors
                for y in 0..size {
                    for x in 0..size {
                        let index = (y * size + x) * 3;
                        let t = y as f32 / size as f32;
                        let s = x as f32 / size as f32;

                        let face_color = config.face_colors[i as usize];
                        let blend = config.blend_factor;

                        // Different gradients for different faces
                        match i {
                            0 => {
                                // Right face
                                data[index as usize] =
                                    ((face_color.x * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 1] =
                                    ((face_color.y * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 2] =
                                    ((face_color.z * 255.0) * (1.0 - blend * t)) as u8;
                            }
                            1 => {
                                // Left face
                                data[index as usize] =
                                    ((face_color.x * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 1] =
                                    ((face_color.y * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 2] =
                                    ((face_color.z * 255.0) * (1.0 - blend * t)) as u8;
                            }
                            2 => {
                                // Top face
                                let radial = 1.0 - ((s - 0.5).powi(2) + (t - 0.5).powi(2)).sqrt();
                                let base = face_color * (1.0 + blend * radial);
                                data[index as usize] = (base.x * 255.0) as u8;
                                data[index as usize + 1] = (base.y * 255.0) as u8;
                                data[index as usize + 2] = (base.z * 255.0) as u8;
                            }
                            3 => {
                                // Bottom face
                                data[index as usize] = (face_color.x * 255.0) as u8;
                                data[index as usize + 1] = (face_color.y * 255.0) as u8;
                                data[index as usize + 2] = (face_color.z * 255.0) as u8;
                            }
                            4 => {
                                // Front face
                                data[index as usize] =
                                    ((face_color.x * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 1] =
                                    ((face_color.y * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 2] =
                                    ((face_color.z * 255.0) * (1.0 - blend * t)) as u8;
                            }
                            5 => {
                                // Back face
                                data[index as usize] =
                                    ((face_color.x * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 1] =
                                    ((face_color.y * 255.0) * (1.0 - blend * t)) as u8;
                                data[index as usize + 2] =
                                    ((face_color.z * 255.0) * (1.0 - blend * t)) as u8;
                            }
                            _ => unreachable!(),
                        }
                    }
                }

                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::RGB as i32,
                    size as i32,
                    size as i32,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as *const _,
                );
            }

            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );
        }

        Ok(Skybox {
            vao,
            shader,
            texture_id,
            config,
        })
    }

    pub fn configure(&mut self, config: SkyboxConfig) -> Result<(), String> {
        self.config = config;
        // Regenerate the skybox textures with new configuration
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture_id);

            if !self.config.use_custom_textures {
                // Regenerate default textures with new configuration
                for i in 0..6 {
                    let size = self.config.texture_size;
                    let mut data = vec![0u8; size as usize * size as usize * 3];

                    // Reuse the same gradient generation code as in new()
                    for y in 0..size {
                        for x in 0..size {
                            let index = (y * size + x) * 3;
                            let t = y as f32 / size as f32;
                            let s = x as f32 / size as f32;

                            let face_color = self.config.face_colors[i as usize];
                            let blend = self.config.blend_factor;

                            match i {
                                0..=5 => {
                                    let color = if i == 2 {
                                        // Top face
                                        let radial =
                                            1.0 - ((s - 0.5).powi(2) + (t - 0.5).powi(2)).sqrt();
                                        face_color * (1.0 + blend * radial)
                                    } else {
                                        face_color * (1.0 - blend * t)
                                    };

                                    data[index as usize] = (color.x * 255.0) as u8;
                                    data[index as usize + 1] = (color.y * 255.0) as u8;
                                    data[index as usize + 2] = (color.z * 255.0) as u8;
                                }
                                _ => unreachable!(),
                            }
                        }
                    }

                    gl::TexImage2D(
                        gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                        0,
                        gl::RGB as i32,
                        size as i32,
                        size as i32,
                        0,
                        gl::RGB,
                        gl::UNSIGNED_BYTE,
                        data.as_ptr() as *const _,
                    );
                }
            }
        }
        Ok(())
    }

    pub fn get_config(&self) -> SkyboxConfig {
        self.config.clone()
    }

    pub fn draw(
        &self,
        view: &cgmath::Matrix4<f32>,
        projection: &cgmath::Matrix4<f32>,
    ) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        let depth_func = unsafe {
            // Save current depth function
            let mut depth_func = 0;
            gl::GetIntegerv(gl::DEPTH_FUNC, &mut depth_func);

            // Configure depth testing for skybox
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthMask(gl::FALSE); // Disable depth writing
            depth_func
        };

        self.shader.bind();
        self.shader.set_uniform_mat4("view", view)?;
        self.shader.set_uniform_mat4("projection", projection)?;
        self.shader
            .set_uniform_vec3("minColor", &self.config.min_color)?;

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture_id);
        }

        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
        Vao::unbind();

        unsafe {
            // Restore previous depth function and enable depth writing
            gl::DepthFunc(depth_func as u32);
            gl::DepthMask(gl::TRUE);
        }

        Ok(())
    }

    pub fn terminate(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
        self.shader.terminate();
        self.vao.terminate();
    }
}
