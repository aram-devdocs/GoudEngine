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

    pub fn create_cube(&mut self, texture_id: u32) -> Result<u32, String> {
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
            vertex_count: 36, // Cube has 36 vertices
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            texture_id,
        });

        Ok(object_id)
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
