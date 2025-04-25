use cgmath::{perspective, Deg, Matrix3, Matrix4, Point3, Vector3};
use gl::types::*;
use std::collections::HashMap;

use super::components::buffer::BufferObject;
use super::components::light::{Light, LightManager};
use super::components::shader::ShaderProgram;
use super::components::skybox::Skybox;
use super::components::vao::Vao;
use super::components::vertex_attribute::VertexAttribute;
use super::renderer::Renderer;
use crate::types::{GridConfig, GridRenderMode, SpriteMap, TextureManager};

#[repr(C)]
#[allow(dead_code)]
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
    objects: HashMap<u32, Object3D>,
    next_object_id: u32,
    camera_position: Vector3<f32>,
    camera_zoom: f32,
    window_width: u32,
    window_height: u32,
    light_manager: LightManager,
    grid_config: GridConfig, // Using the GridConfig from types.rs
    skybox: Option<Skybox>,  // Add skybox field
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

        // Create skybox
        println!("Creating skybox...");
        let skybox = match Skybox::new() {
            Ok(sb) => {
                println!("Skybox created successfully");
                Some(sb)
            }
            Err(e) => {
                println!("Failed to create skybox: {}", e);
                None
            }
        };

        Ok(Renderer3D {
            shader_program,
            objects: HashMap::new(),
            next_object_id: 1,
            camera_position: Vector3::new(0.0, 0.0, 3.0),
            camera_zoom: 1.0,
            window_width,
            window_height,
            light_manager: LightManager::new(),
            grid_config: GridConfig::default(),
            skybox,
        })
    }

    // Combined grid configuration method that takes a GridConfig with partial values
    pub fn configure_grid(&mut self, config: GridConfig) {
        // Only update fields that differ from defaults, preserving existing values
        if config.enabled != self.grid_config.enabled {
            self.grid_config.enabled = config.enabled;
        }

        if config.size != self.grid_config.size {
            self.grid_config.size = config.size;
        }

        if config.divisions != self.grid_config.divisions {
            self.grid_config.divisions = config.divisions;
        }

        // Update colors if they're different from the current values
        if config.xz_color != self.grid_config.xz_color {
            self.grid_config.xz_color = config.xz_color;
        }

        if config.xy_color != self.grid_config.xy_color {
            self.grid_config.xy_color = config.xy_color;
        }

        if config.yz_color != self.grid_config.yz_color {
            self.grid_config.yz_color = config.yz_color;
        }

        if config.x_axis_color != self.grid_config.x_axis_color {
            self.grid_config.x_axis_color = config.x_axis_color;
        }

        if config.y_axis_color != self.grid_config.y_axis_color {
            self.grid_config.y_axis_color = config.y_axis_color;
        }

        if config.z_axis_color != self.grid_config.z_axis_color {
            self.grid_config.z_axis_color = config.z_axis_color;
        }

        if config.line_width != self.grid_config.line_width {
            self.grid_config.line_width = config.line_width;
        }

        if config.axis_line_width != self.grid_config.axis_line_width {
            self.grid_config.axis_line_width = config.axis_line_width;
        }

        if config.show_axes != self.grid_config.show_axes {
            self.grid_config.show_axes = config.show_axes;
        }

        if config.show_xz_plane != self.grid_config.show_xz_plane {
            self.grid_config.show_xz_plane = config.show_xz_plane;
        }

        if config.show_xy_plane != self.grid_config.show_xy_plane {
            self.grid_config.show_xy_plane = config.show_xy_plane;
        }

        if config.show_yz_plane != self.grid_config.show_yz_plane {
            self.grid_config.show_yz_plane = config.show_yz_plane;
        }

        // Update render mode if different
        if config.render_mode != self.grid_config.render_mode {
            println!(
                "Changing grid render mode from {:?} to {:?}",
                self.grid_config.render_mode, config.render_mode
            );
            self.grid_config.render_mode = config.render_mode;
        }
    }

    // Helper method to get the current grid configuration
    pub fn get_grid_config(&self) -> GridConfig {
        self.grid_config.clone()
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
            -1.0, 0.0, 1.0, -w, -h, -d, 0.0, 0.0, -1.0, 0.0, 0.0, // Left face
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

    fn generate_grid_vertices(&self, size: f32, divisions: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let step = size / divisions as f32;
        let half_size = size * 0.5;

        // Grid lines along X axis (horizontal floor grid)
        if self.grid_config.show_xz_plane {
            for i in 0..=divisions {
                let pos = -half_size + i as f32 * step;
                let color = &self.grid_config.xz_color;

                // Line along X axis (varying Z)
                vertices.extend_from_slice(&[
                    -half_size, 0.0, pos, color.x, color.y, color.z, 0.0, 0.0, // Start point
                    half_size, 0.0, pos, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);

                // Line along Z axis (varying X)
                vertices.extend_from_slice(&[
                    pos, 0.0, -half_size, color.x, color.y, color.z, 0.0, 0.0, // Start point
                    pos, 0.0, half_size, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);
            }
        }

        // Calculate the number of Y divisions to maintain square proportions
        let y_divisions = divisions; // Use same number of divisions as X and Z
        let y_step = step; // Use same step size as X and Z

        // Add vertical grid lines along Y axis at X=0 (YZ plane)
        if self.grid_config.show_yz_plane {
            for i in 0..=divisions {
                let pos = -half_size + i as f32 * step;
                let color = &self.grid_config.yz_color;

                // Vertical lines along Y axis (varying Z at X=0)
                vertices.extend_from_slice(&[
                    0.0, -half_size, pos, color.x, color.y, color.z, 0.0, 0.0, // Start point
                    0.0, half_size, pos, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);
            }

            // Add horizontal cross lines on YZ plane (at X=0)
            for i in 0..=y_divisions {
                let y_pos = -half_size + i as f32 * y_step;
                let color = &self.grid_config.yz_color;

                vertices.extend_from_slice(&[
                    0.0, y_pos, -half_size, color.x, color.y, color.z, 0.0,
                    0.0, // Start point
                    0.0, y_pos, half_size, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);
            }
        }

        // Add vertical grid lines along Y axis at Z=0 (XY plane)
        if self.grid_config.show_xy_plane {
            for i in 0..=divisions {
                let pos = -half_size + i as f32 * step;
                let color = &self.grid_config.xy_color;

                // Vertical lines along Y axis (varying X at Z=0)
                vertices.extend_from_slice(&[
                    pos, -half_size, 0.0, color.x, color.y, color.z, 0.0, 0.0, // Start point
                    pos, half_size, 0.0, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);
            }

            // Add horizontal cross lines on XY plane (at Z=0)
            for i in 0..=y_divisions {
                let y_pos = -half_size + i as f32 * y_step;
                let color = &self.grid_config.xy_color;

                vertices.extend_from_slice(&[
                    -half_size, y_pos, 0.0, color.x, color.y, color.z, 0.0,
                    0.0, // Start point
                    half_size, y_pos, 0.0, color.x, color.y, color.z, 0.0, 0.0, // End point
                ]);
            }
        }

        // Highlight the central X, Y, and Z axes if show_axes is true
        if self.grid_config.show_axes {
            // X axis highlight
            vertices.extend_from_slice(&[
                -half_size,
                0.001,
                0.0,
                self.grid_config.x_axis_color.x,
                self.grid_config.x_axis_color.y,
                self.grid_config.x_axis_color.z,
                0.0,
                0.0, // Start point
                half_size,
                0.001,
                0.0,
                self.grid_config.x_axis_color.x,
                self.grid_config.x_axis_color.y,
                self.grid_config.x_axis_color.z,
                0.0,
                0.0, // End point
            ]);

            // Z axis highlight
            vertices.extend_from_slice(&[
                0.0,
                0.001,
                -half_size,
                self.grid_config.z_axis_color.x,
                self.grid_config.z_axis_color.y,
                self.grid_config.z_axis_color.z,
                0.0,
                0.0, // Start point
                0.0,
                0.001,
                half_size,
                self.grid_config.z_axis_color.x,
                self.grid_config.z_axis_color.y,
                self.grid_config.z_axis_color.z,
                0.0,
                0.0, // End point
            ]);

            // Y axis highlight
            vertices.extend_from_slice(&[
                0.0,
                -half_size,
                0.0,
                self.grid_config.y_axis_color.x,
                self.grid_config.y_axis_color.y,
                self.grid_config.y_axis_color.z,
                0.0,
                0.0, // Start point
                0.0,
                half_size,
                0.0,
                self.grid_config.y_axis_color.x,
                self.grid_config.y_axis_color.y,
                self.grid_config.y_axis_color.z,
                0.0,
                0.0, // End point
            ]);
        }

        vertices
    }

    fn generate_axis_vertices(&self, size: f32) -> Vec<f32> {
        let mut vertices = Vec::new();

        // X axis
        vertices.extend_from_slice(&[
            0.0,
            0.0,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0, // Origin
            size,
            0.0,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0, // X direction
        ]);

        // Y axis
        vertices.extend_from_slice(&[
            0.0,
            0.0,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0, // Origin
            0.0,
            size,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0, // Y direction
        ]);

        // Z axis
        vertices.extend_from_slice(&[
            0.0,
            0.0,
            0.0,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0, // Origin
            0.0,
            0.0,
            size,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0, // Z direction
        ]);

        // Add arrow heads (simple cones as triangles)
        // X-axis arrow head
        vertices.extend_from_slice(&[
            size,
            0.0,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
            size - 0.2,
            0.1,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
            size - 0.2,
            -0.1,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
        ]);

        vertices.extend_from_slice(&[
            size,
            0.0,
            0.0,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
            size - 0.2,
            0.0,
            0.1,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
            size - 0.2,
            0.0,
            -0.1,
            self.grid_config.x_axis_color.x,
            self.grid_config.x_axis_color.y,
            self.grid_config.x_axis_color.z,
            0.0,
            0.0,
        ]);

        // Y-axis arrow head
        vertices.extend_from_slice(&[
            0.0,
            size,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
            0.1,
            size - 0.2,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
            -0.1,
            size - 0.2,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
        ]);

        vertices.extend_from_slice(&[
            0.0,
            size,
            0.0,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
            0.0,
            size - 0.2,
            0.1,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
            0.0,
            size - 0.2,
            -0.1,
            self.grid_config.y_axis_color.x,
            self.grid_config.y_axis_color.y,
            self.grid_config.y_axis_color.z,
            0.0,
            0.0,
        ]);

        // Z-axis arrow head
        vertices.extend_from_slice(&[
            0.0,
            0.0,
            size,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
            0.1,
            0.0,
            size - 0.2,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
            -0.1,
            0.0,
            size - 0.2,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
        ]);

        vertices.extend_from_slice(&[
            0.0,
            0.0,
            size,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
            0.0,
            0.1,
            size - 0.2,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
            0.0,
            -0.1,
            size - 0.2,
            self.grid_config.z_axis_color.x,
            self.grid_config.z_axis_color.y,
            self.grid_config.z_axis_color.z,
            0.0,
            0.0,
        ]);

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

    pub fn render_objects(&mut self, texture_manager: &TextureManager) -> Result<(), String> {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS); // Set default depth function
            gl::ClearColor(0.1, 0.1, 0.2, 1.0); // Set a dark blue clear color
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Create view and projection matrices
        let view = Matrix4::look_at_rh(
            Point3::new(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_zoom,
            ),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        let aspect_ratio = self.window_width as f32 / self.window_height as f32;
        let projection = perspective(Deg(45.0), aspect_ratio, 0.1, 1000.0); // Increased far plane for skybox

        // Render skybox first
        if let Some(skybox) = &self.skybox {
            println!("Rendering skybox...");
            // Remove translation from view matrix for skybox
            let skybox_view = Matrix4::from(Matrix3::from_cols(
                view.x.truncate(),
                view.y.truncate(),
                view.z.truncate(),
            ));

            if let Err(e) = skybox.draw(&skybox_view, &projection) {
                println!("Failed to render skybox: {}", e);
            }
        } else {
            println!("No skybox available to render");
        }

        // Now render the rest of the scene
        self.shader_program.bind();

        // Set common uniforms
        self.shader_program.set_uniform_mat4("view", &view)?;
        self.shader_program
            .set_uniform_mat4("projection", &projection)?;
        self.shader_program
            .set_uniform_vec3("viewPos", &self.camera_position)?;

        // Update lights in shader
        self.update_shader_lights()?;

        // Render grid first if enabled and in Blend mode
        if self.grid_config.enabled && self.grid_config.render_mode == GridRenderMode::Blend {
            self.render_grid()?;
        }

        unsafe {
            gl::DepthFunc(gl::LESS); // Ensure proper depth testing for objects
        }

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

        // Render grid last if enabled and in Overlap mode
        if self.grid_config.enabled && self.grid_config.render_mode == GridRenderMode::Overlap {
            self.render_grid()?;
        }

        Ok(())
    }

    fn render_grid(&self) -> Result<(), String> {
        unsafe {
            // Set line width for grid
            gl::LineWidth(self.grid_config.line_width);

            // Handle depth testing based on render mode
            if self.grid_config.render_mode == GridRenderMode::Overlap {
                // Disable depth test for grid so it's always visible (original behavior)
                gl::Disable(gl::DEPTH_TEST);
            } else {
                // For Blend mode, ensure depth test is enabled
                gl::Enable(gl::DEPTH_TEST);
            }

            // Create and bind temporary VAO for grid
            let grid_vao = Vao::new()?;
            grid_vao.bind();

            // Generate grid vertices with configured settings
            let grid_vertices =
                self.generate_grid_vertices(self.grid_config.size, self.grid_config.divisions);
            let grid_vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
            grid_vbo.bind();
            grid_vbo.store_data(&grid_vertices, gl::STATIC_DRAW);

            // Define vertex attributes for position and color
            VertexAttribute::enable(0);
            VertexAttribute::pointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as GLsizei,
                0,
            );

            VertexAttribute::enable(1);
            VertexAttribute::pointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as GLsizei,
                3 * std::mem::size_of::<f32>(),
            );

            VertexAttribute::enable(2);
            VertexAttribute::pointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as GLsizei,
                6 * std::mem::size_of::<f32>(),
            );

            // Draw grid lines
            let model = Matrix4::<f32>::from_scale(1.0);
            self.shader_program.set_uniform_mat4("model", &model)?;

            gl::DrawArrays(gl::LINES, 0, grid_vertices.len() as GLint / 8);

            // Clean up grid resources
            BufferObject::unbind(gl::ARRAY_BUFFER);
            Vao::unbind();

            // Only render axes if they are enabled
            if self.grid_config.show_axes {
                // Set proper depth test handling for axes based on render mode
                if self.grid_config.render_mode == GridRenderMode::Blend {
                    // For Blend mode, ensure depth test is enabled
                    gl::Enable(gl::DEPTH_TEST);
                }

                gl::LineWidth(self.grid_config.axis_line_width); // Make axes significantly thicker

                // Create and bind temporary VAO for axes
                let axes_vao = Vao::new()?;
                axes_vao.bind();

                // Generate axis vertices with larger size
                let axes_vertices = self.generate_axis_vertices(self.grid_config.size * 0.2); // Axes are 20% of grid size
                let axes_vbo = BufferObject::new(gl::ARRAY_BUFFER)?;
                axes_vbo.bind();
                axes_vbo.store_data(&axes_vertices, gl::STATIC_DRAW);

                // Define vertex attributes for position and color
                VertexAttribute::enable(0);
                VertexAttribute::pointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    8 * std::mem::size_of::<f32>() as GLsizei,
                    0,
                );

                VertexAttribute::enable(1);
                VertexAttribute::pointer(
                    1,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    8 * std::mem::size_of::<f32>() as GLsizei,
                    3 * std::mem::size_of::<f32>(),
                );

                VertexAttribute::enable(2);
                VertexAttribute::pointer(
                    2,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    8 * std::mem::size_of::<f32>() as GLsizei,
                    6 * std::mem::size_of::<f32>(),
                );

                // Draw axis lines
                gl::DrawArrays(gl::LINES, 0, 6); // Draw just the main axis lines first

                // Draw arrow heads as triangles
                gl::DrawArrays(gl::TRIANGLES, 6, axes_vertices.len() as GLint / 8 - 6);

                // Clean up axis resources
                BufferObject::unbind(gl::ARRAY_BUFFER);
                Vao::unbind();

                // Clean up VAO to avoid resource leaks
                axes_vao.terminate();
            }

            // Reset line width to default
            gl::LineWidth(1.0);

            // Re-enable depth testing if it was disabled
            gl::Enable(gl::DEPTH_TEST);

            // Clean up grid VAO to avoid resource leaks
            grid_vao.terminate();
        }

        Ok(())
    }

    pub fn terminate(&self) {
        self.shader_program.terminate();
        for object in self.objects.values() {
            object.vao.terminate();
        }
        if let Some(skybox) = &self.skybox {
            skybox.terminate();
        }
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
        if let Some(skybox) = &self.skybox {
            skybox.terminate();
        }
    }
}
