use cgmath::{perspective, Deg, Matrix4, One, Point3, Vector3, Vector4};
use gl::types::*;
use std::collections::HashMap;

use super::components::buffer::BufferObject;
use super::components::light::{Light, LightManager};
use super::components::shader::ShaderProgram;
use super::components::vao::Vao;
use super::components::vertex_attribute::VertexAttribute;
use super::renderer::Renderer;
use crate::types::{SpriteMap, TextureManager};

#[repr(C)]
pub enum PrimitiveType {
    Cube = 0,
    Sphere = 1,
    Plane = 2,
    Cylinder = 3,
    // Add more primitive types as needed
}

#[repr(C)]
pub struct PrimitiveCreateInfo {
    pub primitive_type: PrimitiveType,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub segments: u32, // For curved surfaces like spheres and cylinders
    pub texture_id: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Object3D {
    vao: Vao,
    vertex_count: i32,
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
    texture_id: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Renderer3D {
    shader_program: ShaderProgram,
    debug_shader: Option<ShaderProgram>,
    debug_vao: Option<Vao>,
    objects: HashMap<u32, Object3D>,
    next_object_id: u32,
    camera_position: Vector3<f32>,
    camera_zoom: f32,
    window_width: u32,
    window_height: u32,
    light_manager: LightManager,
    debug_mode: bool,
}

impl Renderer3D {
    pub fn new(window_width: u32, window_height: u32) -> Result<Renderer3D, String> {
        let mut shader_program = ShaderProgram::new_3d()?;

        // Set up uniforms
        shader_program.bind();
        shader_program.create_uniform("model")?;
        shader_program.create_uniform("view")?;
        shader_program.create_uniform("projection")?;
        shader_program.create_uniform("texture1")?;
        shader_program.create_uniform("viewPos")?;
        shader_program.create_uniform("numLights")?;

        // Create uniforms for multiple lights
        for i in 0..8 {
            shader_program.create_uniform(&format!("lights[{}].type", i))?;
            shader_program.create_uniform(&format!("lights[{}].position", i))?;
            shader_program.create_uniform(&format!("lights[{}].direction", i))?;
            shader_program.create_uniform(&format!("lights[{}].color", i))?;
            shader_program.create_uniform(&format!("lights[{}].intensity", i))?;
            shader_program.create_uniform(&format!("lights[{}].range", i))?;
            shader_program.create_uniform(&format!("lights[{}].spotAngle", i))?;
            shader_program.create_uniform(&format!("lights[{}].enabled", i))?;
        }

        shader_program.set_uniform_int("texture1", 0)?;

        // Create perspective projection matrix
        let aspect_ratio = window_width as f32 / window_height as f32;
        let projection = perspective(Deg(45.0), aspect_ratio, 0.1, 100.0);
        shader_program.set_uniform_mat4("projection", &projection)?;

        // Create renderer instance
        let mut renderer = Renderer3D {
            shader_program,
            debug_shader: None,
            debug_vao: None,
            objects: HashMap::new(),
            next_object_id: 1,
            camera_position: Vector3::new(0.0, 0.0, 3.0),
            camera_zoom: 1.0,
            window_width,
            window_height,
            light_manager: LightManager::new(),
            debug_mode: false,
        };

        // Initialize debug rendering
        renderer.setup_debug_rendering()?;

        Ok(renderer)
    }

    pub fn create_primitive(&mut self, create_info: PrimitiveCreateInfo) -> Result<u32, String> {
        let vertices = match create_info.primitive_type {
            PrimitiveType::Cube => self.generate_cube_vertices(
                create_info.width,
                create_info.height,
                create_info.depth,
            ),
            PrimitiveType::Plane => {
                self.generate_plane_vertices(create_info.width, create_info.depth)
            }
            PrimitiveType::Sphere => {
                self.generate_sphere_vertices(create_info.width, create_info.segments)
            }
            PrimitiveType::Cylinder => self.generate_cylinder_vertices(
                create_info.width,
                create_info.height,
                create_info.segments,
            ),
        };

        let vao = Vao::new()?;
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        vbo.bind();
        vbo.store_data(&vertices, gl::STATIC_DRAW);

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

        let object_id = self.next_object_id;
        self.next_object_id += 1;

        self.objects.insert(
            object_id,
            Object3D {
                vao,
                vertex_count: (vertices.len() / 8) as i32, // 8 components per vertex
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                texture_id: create_info.texture_id,
            },
        );

        Ok(object_id)
    }

    fn generate_cube_vertices(&self, width: f32, height: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let h = height / 2.0;
        let d = depth / 2.0;

        vec![
            // Front face
            -w, -h, d, 0.0, 0.0, -1.0, 0.0, 0.0, // Bottom-left
            w, -h, d, 0.0, 0.0, -1.0, 1.0, 0.0, // Bottom-right
            w, h, d, 0.0, 0.0, -1.0, 1.0, 1.0, // Top-right
            w, h, d, 0.0, 0.0, -1.0, 1.0, 1.0, // Top-right
            -w, h, d, 0.0, 0.0, -1.0, 0.0, 1.0, // Top-left
            -w, -h, d, 0.0, 0.0, -1.0, 0.0, 0.0, // Bottom-left
            // Back face
            -w, -h, -d, 0.0, 0.0, -1.0, 0.0, 0.0, w, -h, -d, 0.0, 0.0, -1.0, 1.0, 0.0, w, h, -d,
            0.0, 0.0, -1.0, 1.0, 1.0, w, h, -d, 0.0, 0.0, -1.0, 1.0, 1.0, -w, h, -d, 0.0, 0.0,
            -1.0, 0.0, 0.0, -w, -h, -d, 0.0, 0.0, -1.0, 0.0, 0.0, // Left face
            -w, h, d, -1.0, 0.0, 0.0, 1.0, 0.0, -w, h, -d, -1.0, 0.0, 0.0, 1.0, 1.0, -w, -h, -d,
            -1.0, 0.0, 0.0, 0.0, 1.0, -w, -h, -d, -1.0, 0.0, 0.0, 0.0, 1.0, -w, -h, d, -1.0, 0.0,
            0.0, 0.0, 0.0, -w, h, d, -1.0, 0.0, 0.0, 1.0, 0.0, // Right face
            w, h, d, 1.0, 0.0, 0.0, 1.0, 0.0, w, h, -d, 1.0, 0.0, 0.0, 1.0, 1.0, w, -h, -d, 1.0,
            0.0, 0.0, 0.0, 1.0, w, -h, -d, 1.0, 0.0, 0.0, 0.0, 1.0, w, -h, d, 1.0, 0.0, 0.0, 0.0,
            0.0, w, h, d, 1.0, 0.0, 0.0, 1.0, 0.0, // Bottom face
            -w, -h, -d, 0.0, -1.0, 0.0, 0.0, 1.0, w, -h, -d, 0.0, -1.0, 0.0, 1.0, 1.0, w, -h, d,
            0.0, -1.0, 0.0, 1.0, 0.0, w, -h, d, 0.0, -1.0, 0.0, 1.0, 0.0, -w, -h, d, 0.0, -1.0,
            0.0, 0.0, 0.0, -w, -h, -d, 0.0, -1.0, 0.0, 0.0, 1.0, // Top face
            -w, h, -d, 0.0, 1.0, 0.0, 0.0, 1.0, w, h, -d, 0.0, 1.0, 0.0, 1.0, 1.0, w, h, d, 0.0,
            1.0, 0.0, 1.0, 0.0, w, h, d, 0.0, 1.0, 0.0, 1.0, 0.0, -w, h, d, 0.0, 1.0, 0.0, 0.0,
            0.0, -w, h, -d, 0.0, 1.0, 0.0, 0.0, 1.0,
        ]
    }

    fn generate_plane_vertices(&self, width: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let d = depth / 2.0;

        vec![
            // Single face plane (facing up)
            -w, 0.0, -d, 0.0, 1.0, 0.0, 0.0, 0.0, w, 0.0, -d, 0.0, 1.0, 0.0, 1.0, 0.0, w, 0.0, d,
            0.0, 1.0, 0.0, 1.0, 1.0, w, 0.0, d, 0.0, 1.0, 0.0, 1.0, 1.0, -w, 0.0, d, 0.0, 1.0, 0.0,
            0.0, 1.0, -w, 0.0, -d, 0.0, 1.0, 0.0, 0.0, 0.0,
        ]
    }

    fn generate_sphere_vertices(&self, radius: f32, segments: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let segment_count = segments.max(3);

        for i in 0..segment_count {
            let lat0 = std::f32::consts::PI * (-0.5 + (i as f32) / segment_count as f32);
            let lat1 = std::f32::consts::PI * (-0.5 + ((i + 1) as f32) / segment_count as f32);

            for j in 0..segment_count {
                let lng0 = 2.0 * std::f32::consts::PI * (j as f32) / segment_count as f32;
                let lng1 = 2.0 * std::f32::consts::PI * ((j + 1) as f32) / segment_count as f32;

                // Calculate vertices
                let x0 = radius * lat0.cos() * lng0.cos();
                let y0 = radius * lat0.sin();
                let z0 = radius * lat0.cos() * lng0.sin();

                let x1 = radius * lat0.cos() * lng1.cos();
                let y1 = radius * lat0.sin();
                let z1 = radius * lat0.cos() * lng1.sin();

                let x2 = radius * lat1.cos() * lng1.cos();
                let y2 = radius * lat1.sin();
                let z2 = radius * lat1.cos() * lng1.sin();

                let x3 = radius * lat1.cos() * lng0.cos();
                let y3 = radius * lat1.sin();
                let z3 = radius * lat1.cos() * lng0.sin();

                // Add vertices with their normals and texture coordinates
                let vertices_data = [
                    // First triangle
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    j as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                    x1,
                    y1,
                    z1,
                    x1 / radius,
                    y1 / radius,
                    z1 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                    // Second triangle
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    j as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                    x3,
                    y3,
                    z3,
                    x3 / radius,
                    y3 / radius,
                    z3 / radius,
                    j as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                ];

                vertices.extend_from_slice(&vertices_data);
            }
        }

        vertices
    }

    fn generate_cylinder_vertices(&self, radius: f32, height: f32, segments: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let segment_count = segments.max(3);
        let h = height / 2.0;

        // Generate vertices for the sides
        for i in 0..segment_count {
            let angle0 = 2.0 * std::f32::consts::PI * (i as f32) / segment_count as f32;
            let angle1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segment_count as f32;

            let x0 = radius * angle0.cos();
            let z0 = radius * angle0.sin();
            let x1 = radius * angle1.cos();
            let z1 = radius * angle1.sin();

            // Add vertices for the side faces
            let vertices_data = [
                // Bottom to top quad (two triangles)
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                0.0,
                x1,
                -h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                0.0,
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                1.0,
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                0.0,
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                1.0,
                x0,
                h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                1.0,
            ];
            vertices.extend_from_slice(&vertices_data);

            // Add vertices for top and bottom caps
            let cap_vertices = [
                // Top cap
                0.0,
                h,
                0.0,
                0.0,
                1.0,
                0.0,
                0.5,
                0.5,
                x0,
                h,
                z0,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * angle0.cos(),
                0.5 + 0.5 * angle0.sin(),
                x1,
                h,
                z1,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * angle1.cos(),
                0.5 + 0.5 * angle1.sin(),
                // Bottom cap
                0.0,
                -h,
                0.0,
                0.0,
                -1.0,
                0.0,
                0.5,
                0.5,
                x1,
                -h,
                z1,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * angle1.cos(),
                0.5 + 0.5 * angle1.sin(),
                x0,
                -h,
                z0,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * angle0.cos(),
                0.5 + 0.5 * angle0.sin(),
            ];
            vertices.extend_from_slice(&cap_vertices);
        }

        vertices
    }

    pub fn set_object_position(
        &mut self,
        object_id: u32,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.position = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    pub fn set_object_rotation(
        &mut self,
        object_id: u32,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.rotation = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    pub fn set_object_scale(
        &mut self,
        object_id: u32,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.scale = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    pub fn add_light(&mut self, light: Light) -> u32 {
        self.light_manager.add_light(light)
    }

    pub fn remove_light(&mut self, light_id: u32) {
        self.light_manager.remove_light(light_id);
    }

    pub fn update_light(&mut self, light_id: u32, new_light: Light) -> Result<(), String> {
        if let Some(light) = self.light_manager.get_light_mut(light_id) {
            *light = new_light;
            Ok(())
        } else {
            Err("Light not found".to_string())
        }
    }

    fn update_shader_lights(&self) -> Result<(), String> {
        let lights = self.light_manager.get_all_lights();
        self.shader_program
            .set_uniform_int("numLights", lights.len() as i32)?;

        for (i, light) in lights.iter().enumerate() {
            if i >= 8 {
                break;
            } // Maximum 8 lights supported

            let base = format!("lights[{}]", i);

            self.shader_program
                .set_uniform_int(&format!("{}.type", base), light.light_type as i32)?;
            self.shader_program
                .set_uniform_vec3(&format!("{}.position", base), &light.position)?;
            self.shader_program
                .set_uniform_vec3(&format!("{}.direction", base), &light.direction)?;
            self.shader_program.set_uniform_vec3(
                &format!("{}.color", base),
                &light.get_color_with_temperature(),
            )?;
            self.shader_program
                .set_uniform_float(&format!("{}.intensity", base), light.intensity)?;
            self.shader_program
                .set_uniform_float(&format!("{}.range", base), light.range)?;
            self.shader_program
                .set_uniform_float(&format!("{}.spotAngle", base), light.spot_angle)?;
            self.shader_program
                .set_uniform_int(&format!("{}.enabled", base), light.enabled as i32)?;
        }

        Ok(())
    }

    pub fn set_debug_mode(&mut self, enabled: bool) -> Result<(), String> {
        self.debug_mode = enabled;
        if enabled && self.debug_shader.is_none() {
            self.setup_debug_rendering()?;
        }
        Ok(())
    }

    fn render_debug(&self) -> Result<(), String> {
        let debug_shader = self.debug_shader.as_ref().unwrap();
        let debug_vao = self.debug_vao.as_ref().unwrap();

        debug_shader.bind();
        debug_vao.bind();

        unsafe {
            // Save current state
            let mut line_width = 0.0f32;
            gl::GetFloatv(gl::LINE_WIDTH, &mut line_width);
            
            // Setup grid rendering state
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::LineWidth(1.0);  // Thinner lines for grid
        }

        // Create view matrix with adjusted camera position
        let camera_pos = Point3::new(
            self.camera_position.x,
            self.camera_position.y,
            self.camera_position.z + self.camera_zoom,
        );
        let view = Matrix4::look_at_rh(
            camera_pos,
            Point3::new(
                self.camera_position.x,
                0.0,
                self.camera_position.z,
            ),
            Vector3::new(0.0, 1.0, 0.0),
        );

        let aspect_ratio = self.window_width as f32 / self.window_height as f32;
        let projection = perspective(Deg(60.0), aspect_ratio, 0.1, 1000.0);

        debug_shader.set_uniform_mat4("view", &view)?;
        debug_shader.set_uniform_mat4("projection", &projection)?;
        debug_shader.set_uniform_mat4("model", &Matrix4::one())?;

        unsafe {
            // Draw main grid with subtle color
            debug_shader.set_uniform_vec4("color", &Vector4::new(0.3, 0.3, 0.3, 0.5))?;
            let num_grid_lines = (10 * 2 + 1) * 2;  // Main grid lines
            gl::DrawArrays(gl::LINES, 0, num_grid_lines * 2);

            // Draw coordinate axes with thicker lines
            gl::LineWidth(2.0);

            // X-axis (red)
            debug_shader.set_uniform_vec4("color", &Vector4::new(0.8, 0.2, 0.2, 1.0))?;
            gl::DrawArrays(gl::LINES, num_grid_lines * 2, 2);

            // Y-axis (green)
            debug_shader.set_uniform_vec4("color", &Vector4::new(0.2, 0.8, 0.2, 1.0))?;
            gl::DrawArrays(gl::LINES, num_grid_lines * 2 + 2, 2);

            // Z-axis (blue)
            debug_shader.set_uniform_vec4("color", &Vector4::new(0.2, 0.2, 0.8, 1.0))?;
            gl::DrawArrays(gl::LINES, num_grid_lines * 2 + 4, 2);

            // Restore state
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::LineWidth(1.0);
        }

        Ok(())
    }

    fn setup_debug_rendering(&mut self) -> Result<(), String> {
        // Create and set up debug shader
        let mut debug_shader = ShaderProgram::new_debug()?;
        
        debug_shader.bind();
        debug_shader.create_uniform("model")?;
        debug_shader.create_uniform("view")?;
        debug_shader.create_uniform("projection")?;
        debug_shader.create_uniform("color")?;

        // Create VAO for debug grid
        let vao = Vao::new()?;
        vao.bind();

        // Generate grid vertices
        let mut vertices = Vec::new();
        let grid_size = 1.0;  // Unity-like grid size
        let grid_count = 10;
        let grid_extent = grid_size * grid_count as f32;

        // Add grid lines for XZ plane (floor)
        for i in -grid_count..=grid_count {
            let pos = i as f32 * grid_size;
            // Lines along X axis
            vertices.extend_from_slice(&[
                -grid_extent, 0.0, pos,
                grid_extent, 0.0, pos,
            ]);
            // Lines along Z axis
            vertices.extend_from_slice(&[
                pos, 0.0, -grid_extent,
                pos, 0.0, grid_extent,
            ]);
        }

        // Add coordinate axes (slightly longer than grid)
        let axis_length = grid_extent * 1.2;
        vertices.extend_from_slice(&[
            // X axis (red)
            0.0, 0.0, 0.0,
            axis_length, 0.0, 0.0,
            // Y axis (green)
            0.0, 0.0, 0.0,
            0.0, axis_length, 0.0,
            // Z axis (blue)
            0.0, 0.0, 0.0,
            0.0, 0.0, axis_length,
        ]);

        let vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
        vbo.bind();
        vbo.store_data(&vertices, gl::STATIC_DRAW);

        VertexAttribute::enable(0);
        VertexAttribute::pointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as GLsizei, 0);

        self.debug_shader = Some(debug_shader);
        self.debug_vao = Some(vao);

        Ok(())
    }

    fn render_objects(&mut self, texture_manager: &TextureManager) -> Result<(), String> {
        unsafe {
            // Set clear color to a dark gray like Unity
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // First render the grid if debug mode is enabled
        if self.debug_mode {
            if let Err(e) = self.render_debug() {
                eprintln!("Failed to render debug grid: {}", e);
            }
        }

        // Then render the regular objects with proper depth testing
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }

        self.shader_program.bind();

        // Create view matrix with adjusted camera position
        let camera_pos = Point3::new(
            self.camera_position.x,
            self.camera_position.y,
            self.camera_position.z + self.camera_zoom, // Add zoom to Z position
        );
        let view = Matrix4::look_at_rh(
            camera_pos,
            Point3::new(
                self.camera_position.x,
                0.0, // Look at Y=0 plane
                self.camera_position.z,
            ),
            Vector3::new(0.0, 1.0, 0.0),
        );

        // Set common uniforms
        self.shader_program.set_uniform_mat4("view", &view)?;
        self.shader_program
            .set_uniform_vec3("viewPos", &Vector3::new(camera_pos.x, camera_pos.y, camera_pos.z))?;

        // Update lights in shader
        self.update_shader_lights()?;

        // Render each object
        for object in self.objects.values() {
            object.vao.bind();

            let model = Matrix4::from_translation(object.position)
                * Matrix4::from_angle_x(Deg(object.rotation.x))
                * Matrix4::from_angle_y(Deg(object.rotation.y))
                * Matrix4::from_angle_z(Deg(object.rotation.z))
                * Matrix4::from_nonuniform_scale(object.scale.x, object.scale.y, object.scale.z);

            self.shader_program.set_uniform_mat4("model", &model)?;

            if let Some(texture) = texture_manager.textures.get(&object.texture_id) {
                texture.bind(gl::TEXTURE0);
            }

            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, object.vertex_count);
            }
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

        if let Err(e) = self.render_objects(texture_manager) {
            eprintln!("Error rendering objects: {}", e);
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
        for object in self.objects.values() {
            object.vao.terminate();
        }
        if let Some(debug_shader) = &self.debug_shader {
            debug_shader.terminate();
        }
        if let Some(debug_vao) = &self.debug_vao {
            debug_vao.terminate();
        }
    }

    fn set_debug_mode(&mut self, enabled: bool) {
        // Directly set the debug mode flag first
        self.debug_mode = enabled;

        // Then try to setup debug rendering if needed
        if enabled && self.debug_shader.is_none() {
            if let Err(e) = self.setup_debug_rendering() {
                eprintln!("Failed to setup debug rendering: {}", e);
                // Even if setup fails, keep debug_mode true as it may succeed next frame
            }
        }
    }
}
