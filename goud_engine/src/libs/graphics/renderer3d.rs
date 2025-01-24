use cgmath::{perspective, Matrix4, Point3, Vector3, Deg};
use gl::types::*;
use std::collections::HashMap;

use super::components::buffer::BufferObject;
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
    pub segments: u32,  // For curved surfaces like spheres and cylinders
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
    light_position: Vector3<f32>,
    light_color: Vector3<f32>,
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
        shader_program.create_uniform("lightPos")?;
        shader_program.create_uniform("viewPos")?;
        shader_program.create_uniform("lightColor")?;
        shader_program.create_uniform("ambientStrength")?;
        shader_program.create_uniform("specularStrength")?;

        shader_program.set_uniform_int("texture1", 0)?;

        // Create perspective projection matrix
        let aspect_ratio = window_width as f32 / window_height as f32;
        let projection = perspective(Deg(45.0), aspect_ratio, 0.1, 100.0);
        shader_program.set_uniform_mat4("projection", &projection)?;

        Ok(Renderer3D {
            shader_program,
            objects: HashMap::new(),
            next_object_id: 1,
            camera_position: Vector3::new(0.0, 0.0, 3.0),
            camera_zoom: 1.0,
            window_width,
            window_height,
            light_position: Vector3::new(2.0, 2.0, 2.0),
            light_color: Vector3::new(1.0, 1.0, 1.0),
        })
    }

    pub fn create_primitive(&mut self, create_info: PrimitiveCreateInfo) -> Result<u32, String> {
        let vertices = match create_info.primitive_type {
            PrimitiveType::Cube => self.generate_cube_vertices(create_info.width, create_info.height, create_info.depth),
            PrimitiveType::Plane => self.generate_plane_vertices(create_info.width, create_info.depth),
            PrimitiveType::Sphere => self.generate_sphere_vertices(create_info.width, create_info.segments),
            PrimitiveType::Cylinder => self.generate_cylinder_vertices(create_info.width, create_info.height, create_info.segments),
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
        VertexAttribute::pointer(1, 3, gl::FLOAT, gl::FALSE, stride, 3 * std::mem::size_of::<f32>());

        // Texture coordinate attribute
        VertexAttribute::enable(2);
        VertexAttribute::pointer(2, 2, gl::FLOAT, gl::FALSE, stride, 6 * std::mem::size_of::<f32>());

        Vao::unbind();
        BufferObject::unbind(gl::ARRAY_BUFFER);

        let object_id = self.next_object_id;
        self.next_object_id += 1;

        self.objects.insert(object_id, Object3D {
            vao,
            vertex_count: (vertices.len() / 8) as i32, // 8 components per vertex
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            texture_id: create_info.texture_id,
        });

        Ok(object_id)
    }

    fn generate_cube_vertices(&self, width: f32, height: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let h = height / 2.0;
        let d = depth / 2.0;

        vec![
            // Front face
            -w, -h, d,    0.0,  0.0, -1.0,    0.0, 0.0,  // Bottom-left
            w, -h, d,     0.0,  0.0, -1.0,    1.0, 0.0,  // Bottom-right
            w, h, d,      0.0,  0.0, -1.0,    1.0, 1.0,  // Top-right
            w, h, d,      0.0,  0.0, -1.0,    1.0, 1.0,  // Top-right
            -w, h, d,     0.0,  0.0, -1.0,    0.0, 1.0,  // Top-left
            -w, -h, d,    0.0,  0.0, -1.0,    0.0, 0.0,  // Bottom-left

            // Back face
            -w, -h, -d,   0.0,  0.0, -1.0,   0.0, 0.0,
            w, -h, -d,    0.0,  0.0, -1.0,   1.0, 0.0,
            w, h, -d,     0.0,  0.0, -1.0,   1.0, 1.0,
            w, h, -d,     0.0,  0.0, -1.0,   1.0, 1.0,
            -w, h, -d,    0.0,  0.0, -1.0,   0.0, 1.0,
            -w, -h, -d,   0.0,  0.0, -1.0,   0.0, 0.0,

            // Left face
            -w, h, d,     -1.0,  0.0,  0.0,   1.0, 0.0,
            -w, h, -d,    -1.0,  0.0,  0.0,   1.0, 1.0,
            -w, -h, -d,   -1.0,  0.0,  0.0,   0.0, 1.0,
            -w, -h, -d,   -1.0,  0.0,  0.0,   0.0, 1.0,
            -w, -h, d,    -1.0,  0.0,  0.0,   0.0, 0.0,
            -w, h, d,     -1.0,  0.0,  0.0,   1.0, 0.0,

            // Right face
            w, h, d,      1.0,  0.0,  0.0,    1.0, 0.0,
            w, h, -d,     1.0,  0.0,  0.0,    1.0, 1.0,
            w, -h, -d,    1.0,  0.0,  0.0,    0.0, 1.0,
            w, -h, -d,    1.0,  0.0,  0.0,    0.0, 1.0,
            w, -h, d,     1.0,  0.0,  0.0,    0.0, 0.0,
            w, h, d,      1.0,  0.0,  0.0,    1.0, 0.0,

            // Bottom face
            -w, -h, -d,   0.0, -1.0,  0.0,   0.0, 1.0,
            w, -h, -d,    0.0, -1.0,  0.0,   1.0, 1.0,
            w, -h, d,     0.0, -1.0,  0.0,   1.0, 0.0,
            w, -h, d,     0.0, -1.0,  0.0,   1.0, 0.0,
            -w, -h, d,    0.0, -1.0,  0.0,   0.0, 0.0,
            -w, -h, -d,   0.0, -1.0,  0.0,   0.0, 1.0,

            // Top face
            -w, h, -d,    0.0,  1.0,  0.0,    0.0, 1.0,
            w, h, -d,     0.0,  1.0,  0.0,    1.0, 1.0,
            w, h, d,      0.0,  1.0,  0.0,    1.0, 0.0,
            w, h, d,      0.0,  1.0,  0.0,    1.0, 0.0,
            -w, h, d,     0.0,  1.0,  0.0,    0.0, 0.0,
            -w, h, -d,    0.0,  1.0,  0.0,    0.0, 1.0,
        ]
    }

    fn generate_plane_vertices(&self, width: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let d = depth / 2.0;

        vec![
            // Single face plane (facing up)
            -w, 0.0, -d,   0.0, 1.0, 0.0,   0.0, 0.0,
            w, 0.0, -d,    0.0, 1.0, 0.0,   1.0, 0.0,
            w, 0.0, d,     0.0, 1.0, 0.0,   1.0, 1.0,
            w, 0.0, d,     0.0, 1.0, 0.0,   1.0, 1.0,
            -w, 0.0, d,    0.0, 1.0, 0.0,   0.0, 1.0,
            -w, 0.0, -d,   0.0, 1.0, 0.0,   0.0, 0.0,
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
                    x0, y0, z0,    x0/radius, y0/radius, z0/radius,    j as f32/segment_count as f32, i as f32/segment_count as f32,
                    x1, y1, z1,    x1/radius, y1/radius, z1/radius,    (j+1) as f32/segment_count as f32, i as f32/segment_count as f32,
                    x2, y2, z2,    x2/radius, y2/radius, z2/radius,    (j+1) as f32/segment_count as f32, (i+1) as f32/segment_count as f32,
                    
                    // Second triangle
                    x0, y0, z0,    x0/radius, y0/radius, z0/radius,    j as f32/segment_count as f32, i as f32/segment_count as f32,
                    x2, y2, z2,    x2/radius, y2/radius, z2/radius,    (j+1) as f32/segment_count as f32, (i+1) as f32/segment_count as f32,
                    x3, y3, z3,    x3/radius, y3/radius, z3/radius,    j as f32/segment_count as f32, (i+1) as f32/segment_count as f32,
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
                x0, -h, z0,    x0/radius, 0.0, z0/radius,    i as f32/segment_count as f32, 0.0,
                x1, -h, z1,    x1/radius, 0.0, z1/radius,    (i+1) as f32/segment_count as f32, 0.0,
                x1, h, z1,     x1/radius, 0.0, z1/radius,    (i+1) as f32/segment_count as f32, 1.0,
                
                x0, -h, z0,    x0/radius, 0.0, z0/radius,    i as f32/segment_count as f32, 0.0,
                x1, h, z1,     x1/radius, 0.0, z1/radius,    (i+1) as f32/segment_count as f32, 1.0,
                x0, h, z0,     x0/radius, 0.0, z0/radius,    i as f32/segment_count as f32, 1.0,
            ];
            vertices.extend_from_slice(&vertices_data);

            // Add vertices for top and bottom caps
            let cap_vertices = [
                // Top cap
                0.0, h, 0.0,     0.0, 1.0, 0.0,    0.5, 0.5,
                x0, h, z0,       0.0, 1.0, 0.0,    0.5 + 0.5 * angle0.cos(), 0.5 + 0.5 * angle0.sin(),
                x1, h, z1,       0.0, 1.0, 0.0,    0.5 + 0.5 * angle1.cos(), 0.5 + 0.5 * angle1.sin(),

                // Bottom cap
                0.0, -h, 0.0,    0.0, -1.0, 0.0,   0.5, 0.5,
                x1, -h, z1,      0.0, -1.0, 0.0,   0.5 + 0.5 * angle1.cos(), 0.5 + 0.5 * angle1.sin(),
                x0, -h, z0,      0.0, -1.0, 0.0,   0.5 + 0.5 * angle0.cos(), 0.5 + 0.5 * angle0.sin(),
            ];
            vertices.extend_from_slice(&cap_vertices);
        }

        vertices
    }

    pub fn set_object_position(&mut self, object_id: u32, x: f32, y: f32, z: f32) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.position = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    pub fn set_object_rotation(&mut self, object_id: u32, x: f32, y: f32, z: f32) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.rotation = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    pub fn set_object_scale(&mut self, object_id: u32, x: f32, y: f32, z: f32) -> Result<(), String> {
        if let Some(object) = self.objects.get_mut(&object_id) {
            object.scale = Vector3::new(x, y, z);
            Ok(())
        } else {
            Err("Object not found".to_string())
        }
    }

    fn render_objects(&mut self, texture_manager: &TextureManager) -> Result<(), String> {
        self.shader_program.bind();

        // Create view matrix
        let view = Matrix4::look_at_rh(
            Point3::new(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_zoom,
            ),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );

        // Set common uniforms
        self.shader_program.set_uniform_mat4("view", &view)?;
        self.shader_program.set_uniform_vec3("lightPos", &self.light_position)?;
        self.shader_program.set_uniform_vec3("viewPos", &self.camera_position)?;
        self.shader_program.set_uniform_vec3("lightColor", &self.light_color)?;
        self.shader_program.set_uniform_float("ambientStrength", 0.3)?;
        self.shader_program.set_uniform_float("specularStrength", 0.7)?;

        // Render each object
        for object in self.objects.values() {
            object.vao.bind();

            // Create model matrix with object's transformation
            let model = Matrix4::from_translation(object.position)
                * Matrix4::from_angle_x(Deg(object.rotation.x))
                * Matrix4::from_angle_y(Deg(object.rotation.y))
                * Matrix4::from_angle_z(Deg(object.rotation.z))
                * Matrix4::from_nonuniform_scale(object.scale.x, object.scale.y, object.scale.z);

            self.shader_program.set_uniform_mat4("model", &model)?;

            // Bind texture
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
    }
}
