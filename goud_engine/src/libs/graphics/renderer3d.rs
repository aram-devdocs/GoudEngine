//! 3D Renderer Module
//!
//! Provides a complete 3D rendering system with:
//! - Primitive creation (cubes, spheres, planes, cylinders)
//! - Multiple light types (point, directional, spot)
//! - Camera control with position and rotation
//! - Grid rendering
//! - Skybox support

use cgmath::{perspective, Deg, Matrix, Matrix4, Rad, Vector3, Vector4};
use std::collections::HashMap;
use std::ffi::CString;

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of lights supported
pub const MAX_LIGHTS: usize = 8;

// ============================================================================
// Types
// ============================================================================

/// Type of 3D primitive
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Cube = 0,
    Sphere = 1,
    Plane = 2,
    Cylinder = 3,
}

/// Type of light source
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Point = 0,
    Directional = 1,
    Spot = 2,
}

/// Grid render mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridRenderMode {
    Blend = 0,
    Overlap = 1,
}

/// Primitive creation info
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PrimitiveCreateInfo {
    pub primitive_type: PrimitiveType,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub segments: u32,
    pub texture_id: u32,
}

/// A 3D object in the scene
#[derive(Debug)]
pub struct Object3D {
    vao: u32,
    vbo: u32,
    vertex_count: i32,
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
    texture_id: u32,
}

impl Drop for Object3D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

/// A light in the scene
#[derive(Debug, Clone)]
pub struct Light {
    pub light_type: LightType,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Point,
            position: Vector3::new(0.0, 5.0, 0.0),
            direction: Vector3::new(0.0, -1.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0,
            enabled: true,
        }
    }
}

/// Grid configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub enabled: bool,
    pub size: f32,
    pub divisions: u32,
    pub show_xz_plane: bool,
    pub show_xy_plane: bool,
    pub show_yz_plane: bool,
    pub render_mode: GridRenderMode,
    pub line_color: Vector3<f32>,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 20.0,
            divisions: 20,
            show_xz_plane: true,
            show_xy_plane: false,
            show_yz_plane: false,
            render_mode: GridRenderMode::Blend,
            line_color: Vector3::new(0.5, 0.5, 0.5),
        }
    }
}

/// Skybox configuration
#[derive(Debug, Clone)]
pub struct SkyboxConfig {
    pub enabled: bool,
    pub color: Vector4<f32>,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            color: Vector4::new(0.1, 0.1, 0.2, 1.0),
        }
    }
}

/// Fog configuration
#[derive(Debug, Clone)]
pub struct FogConfig {
    pub enabled: bool,
    pub color: Vector3<f32>,
    pub density: f32,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Vector3::new(0.05, 0.05, 0.1), // Match moody skybox
            density: 0.02,
        }
    }
}

/// 3D Camera
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, // pitch, yaw, roll in degrees
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 10.0, -20.0),
            rotation: Vector3::new(-30.0, 0.0, 0.0), // Looking slightly down
        }
    }
}

impl Camera3D {
    /// Get the view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let pitch = Rad::from(Deg(self.rotation.x));
        let yaw = Rad::from(Deg(self.rotation.y));

        // Calculate forward direction
        let cos_pitch = pitch.0.cos();
        let sin_pitch = pitch.0.sin();
        let cos_yaw = yaw.0.cos();
        let sin_yaw = yaw.0.sin();

        let forward = Vector3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch);

        let target = self.position + forward;
        let up = Vector3::new(0.0, 1.0, 0.0);

        Matrix4::look_at_rh(
            cgmath::Point3::new(self.position.x, self.position.y, self.position.z),
            cgmath::Point3::new(target.x, target.y, target.z),
            up,
        )
    }
}

// ============================================================================
// Shaders
// ============================================================================

const VERTEX_SHADER_3D: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoord = aTexCoord;
    
    gl_Position = projection * view * vec4(FragPos, 1.0);
}
"#;

const FRAGMENT_SHADER_3D: &str = r#"
#version 330 core

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoord;

out vec4 FragColor;

struct Light {
    int type;           // 0 = point, 1 = directional, 2 = spot
    vec3 position;
    vec3 direction;
    vec3 color;
    float intensity;
    float range;
    float spotAngle;
    bool enabled;
};

uniform sampler2D texture1;
uniform vec3 viewPos;
uniform int numLights;
uniform Light lights[8];
uniform bool useTexture;
uniform vec4 objectColor;

vec3 calculatePointLight(Light light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
    
    // Diffuse
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * light.color;
    
    // Specular
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = spec * light.color * 0.5;
    
    // Range falloff
    float rangeAttenuation = 1.0 - smoothstep(0.0, light.range, distance);
    
    return (diffuse + specular) * attenuation * rangeAttenuation * light.intensity;
}

vec3 calculateDirectionalLight(Light light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    
    // Diffuse
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * light.color;
    
    // Specular
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = spec * light.color * 0.5;
    
    return (diffuse + specular) * light.intensity;
}

vec3 calculateSpotLight(Light light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = cos(radians(light.spotAngle)) - cos(radians(light.spotAngle + 5.0));
    float intensity = clamp((theta - cos(radians(light.spotAngle + 5.0))) / epsilon, 0.0, 1.0);
    
    if (intensity > 0.0) {
        return calculatePointLight(light, normal, fragPos, viewDir) * intensity;
    }
    return vec3(0.0);
}

// Fog parameters
uniform bool fogEnabled;
uniform vec3 fogColor;
uniform float fogDensity;

float calculateFog(float distance) {
    // Exponential squared fog (more dramatic falloff)
    float fog = exp(-pow(fogDensity * distance, 2.0));
    return clamp(fog, 0.0, 1.0);
}

void main() {
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 result = vec3(0.0);
    
    // Ambient light base
    vec3 ambient = vec3(0.1);
    result += ambient;
    
    // Calculate contribution from each light
    for(int i = 0; i < numLights && i < 8; i++) {
        if (!lights[i].enabled) continue;
        
        vec3 lightContribution;
        if (lights[i].type == 0) {
            lightContribution = calculatePointLight(lights[i], norm, FragPos, viewDir);
        }
        else if (lights[i].type == 1) {
            lightContribution = calculateDirectionalLight(lights[i], norm, viewDir);
        }
        else if (lights[i].type == 2) {
            lightContribution = calculateSpotLight(lights[i], norm, FragPos, viewDir);
        }
        
        result += lightContribution;
    }
    
    vec4 baseColor;
    if (useTexture) {
        baseColor = texture(texture1, TexCoord);
    } else {
        baseColor = objectColor;
    }
    
    vec3 finalColor = result * baseColor.rgb;
    
    // Apply fog - fade to dark fog color at distance
    if (fogEnabled) {
        float distance = length(FragPos - viewPos);
        float visibility = calculateFog(distance);
        // Mix: low visibility = more fog color, high visibility = more scene color
        finalColor = mix(fogColor, finalColor, visibility);
    }
    
    FragColor = vec4(finalColor, baseColor.a);
}
"#;

const GRID_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

out vec3 vertexColor;
out vec3 fragPos;

uniform mat4 view;
uniform mat4 projection;

void main()
{
    fragPos = aPos;
    vertexColor = aColor;
    gl_Position = projection * view * vec4(aPos, 1.0);
}
"#;

const GRID_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec3 vertexColor;
in vec3 fragPos;

out vec4 FragColor;

uniform vec3 viewPos;
uniform bool fogEnabled;
uniform vec3 fogColor;
uniform float fogDensity;
uniform float alpha;

void main()
{
    vec3 color = vertexColor;
    
    // Apply fog
    if (fogEnabled) {
        float distance = length(viewPos - fragPos);
        float fogFactor = exp(-fogDensity * distance);
        fogFactor = clamp(fogFactor, 0.0, 1.0);
        color = mix(fogColor, color, fogFactor);
    }
    
    FragColor = vec4(color, alpha);
}
"#;

// Simple 2D HUD shader for on-screen overlays
const HUD_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec2 aPos;

uniform vec2 position;
uniform vec2 size;
uniform vec2 screenSize;

void main()
{
    // Convert from pixel coordinates to NDC (-1 to 1)
    vec2 pixelPos = position + aPos * size;
    vec2 ndc = (pixelPos / screenSize) * 2.0 - 1.0;
    ndc.y = -ndc.y; // Flip Y for screen coordinates (top-left origin)
    gl_Position = vec4(ndc, 0.0, 1.0);
}
"#;

const HUD_FRAGMENT_SHADER: &str = r#"
#version 330 core

out vec4 FragColor;

uniform vec4 color;

void main()
{
    FragColor = color;
}
"#;

// ============================================================================
// Renderer3D
// ============================================================================

/// The main 3D renderer
pub struct Renderer3D {
    shader_program: u32,
    grid_shader: u32,
    grid_vao: u32,
    grid_vbo: u32,
    grid_vertex_count: i32,
    axis_vao: u32,
    axis_vbo: u32,
    axis_vertex_count: i32,
    objects: HashMap<u32, Object3D>,
    lights: HashMap<u32, Light>,
    next_object_id: u32,
    next_light_id: u32,
    camera: Camera3D,
    window_width: u32,
    window_height: u32,
    grid_config: GridConfig,
    skybox_config: SkyboxConfig,
    fog_config: FogConfig,
    // Uniform locations
    u_model: i32,
    u_view: i32,
    u_projection: i32,
    u_view_pos: i32,
    u_num_lights: i32,
    u_use_texture: i32,
    u_object_color: i32,
    u_texture1: i32,
    u_fog_enabled: i32,
    u_fog_color: i32,
    u_fog_density: i32,
    // Grid uniform locations
    ug_view: i32,
    ug_projection: i32,
    ug_view_pos: i32,
    ug_fog_enabled: i32,
    ug_fog_color: i32,
    ug_fog_density: i32,
    ug_alpha: i32,
}

impl Renderer3D {
    /// Create a new 3D renderer
    pub fn new(window_width: u32, window_height: u32) -> Result<Self, String> {
        // Create main 3D shader
        let shader_program = Self::create_shader_program(VERTEX_SHADER_3D, FRAGMENT_SHADER_3D)?;

        // Get uniform locations
        let u_model = Self::get_uniform_location(shader_program, "model");
        let u_view = Self::get_uniform_location(shader_program, "view");
        let u_projection = Self::get_uniform_location(shader_program, "projection");
        let u_view_pos = Self::get_uniform_location(shader_program, "viewPos");
        let u_num_lights = Self::get_uniform_location(shader_program, "numLights");
        let u_use_texture = Self::get_uniform_location(shader_program, "useTexture");
        let u_object_color = Self::get_uniform_location(shader_program, "objectColor");
        let u_texture1 = Self::get_uniform_location(shader_program, "texture1");
        let u_fog_enabled = Self::get_uniform_location(shader_program, "fogEnabled");
        let u_fog_color = Self::get_uniform_location(shader_program, "fogColor");
        let u_fog_density = Self::get_uniform_location(shader_program, "fogDensity");

        // Create grid shader
        let grid_shader = Self::create_shader_program(GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER)?;
        let ug_view = Self::get_uniform_location(grid_shader, "view");
        let ug_projection = Self::get_uniform_location(grid_shader, "projection");
        let ug_view_pos = Self::get_uniform_location(grid_shader, "viewPos");
        let ug_fog_enabled = Self::get_uniform_location(grid_shader, "fogEnabled");
        let ug_fog_color = Self::get_uniform_location(grid_shader, "fogColor");
        let ug_fog_density = Self::get_uniform_location(grid_shader, "fogDensity");
        let ug_alpha = Self::get_uniform_location(grid_shader, "alpha");

        // Create grid VAO/VBO with all 3 planes
        let (grid_vao, grid_vbo, grid_vertex_count) = Self::create_grid_mesh_with_planes(20.0, 20)?;

        // Create axis markers VAO/VBO
        let (axis_vao, axis_vbo, axis_vertex_count) = Self::create_axis_markers(5.0)?;

        Ok(Self {
            shader_program,
            grid_shader,
            grid_vao,
            grid_vbo,
            grid_vertex_count,
            axis_vao,
            axis_vbo,
            axis_vertex_count,
            objects: HashMap::new(),
            lights: HashMap::new(),
            next_object_id: 1,
            next_light_id: 1,
            camera: Camera3D::default(),
            window_width,
            window_height,
            grid_config: GridConfig::default(),
            skybox_config: SkyboxConfig::default(),
            fog_config: FogConfig::default(),
            u_model,
            u_view,
            u_projection,
            u_view_pos,
            u_num_lights,
            u_use_texture,
            u_object_color,
            u_texture1,
            u_fog_enabled,
            u_fog_color,
            u_fog_density,
            ug_view,
            ug_projection,
            ug_view_pos,
            ug_fog_enabled,
            ug_fog_color,
            ug_fog_density,
            ug_alpha,
        })
    }

    fn create_shader_program(vertex_src: &str, fragment_src: &str) -> Result<u32, String> {
        unsafe {
            // Vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str = CString::new(vertex_src).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let mut success = 0;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(
                    vertex_shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut _,
                );
                return Err(format!(
                    "Vertex shader error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            // Fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str = CString::new(fragment_src).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);

            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(
                    fragment_shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut _,
                );
                return Err(format!(
                    "Fragment shader error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            // Link program
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut _,
                );
                return Err(format!(
                    "Shader link error: {}",
                    String::from_utf8_lossy(&buffer)
                ));
            }

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(program)
        }
    }

    fn get_uniform_location(program: u32, name: &str) -> i32 {
        unsafe {
            let c_str = CString::new(name).unwrap();
            gl::GetUniformLocation(program, c_str.as_ptr())
        }
    }

    fn create_grid_mesh_with_planes(size: f32, divisions: u32) -> Result<(u32, u32, i32), String> {
        // Each vertex: position (3) + color (3) = 6 floats
        let mut vertices = Vec::new();
        let step = size / divisions as f32;
        let half = size / 2.0;

        // Grid colors
        let xz_color = [0.3, 0.3, 0.3]; // Floor grid - dark gray
        let xy_color = [0.2, 0.25, 0.3]; // XY plane - blue tint
        let yz_color = [0.3, 0.2, 0.25]; // YZ plane - red tint

        // XZ plane (floor) - Y = 0
        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            // Line along Z axis
            vertices.extend_from_slice(&[pos, 0.0, -half]);
            vertices.extend_from_slice(&xz_color);
            vertices.extend_from_slice(&[pos, 0.0, half]);
            vertices.extend_from_slice(&xz_color);

            // Line along X axis
            vertices.extend_from_slice(&[-half, 0.0, pos]);
            vertices.extend_from_slice(&xz_color);
            vertices.extend_from_slice(&[half, 0.0, pos]);
            vertices.extend_from_slice(&xz_color);
        }

        // XY plane (back wall) - Z = 0
        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            // Line along Y axis
            vertices.extend_from_slice(&[pos, -half, 0.0]);
            vertices.extend_from_slice(&xy_color);
            vertices.extend_from_slice(&[pos, half, 0.0]);
            vertices.extend_from_slice(&xy_color);

            // Line along X axis
            vertices.extend_from_slice(&[-half, pos, 0.0]);
            vertices.extend_from_slice(&xy_color);
            vertices.extend_from_slice(&[half, pos, 0.0]);
            vertices.extend_from_slice(&xy_color);
        }

        // YZ plane (side wall) - X = 0
        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            // Line along Z axis
            vertices.extend_from_slice(&[0.0, pos, -half]);
            vertices.extend_from_slice(&yz_color);
            vertices.extend_from_slice(&[0.0, pos, half]);
            vertices.extend_from_slice(&yz_color);

            // Line along Y axis
            vertices.extend_from_slice(&[0.0, -half, pos]);
            vertices.extend_from_slice(&yz_color);
            vertices.extend_from_slice(&[0.0, half, pos]);
            vertices.extend_from_slice(&yz_color);
        }

        let mut vao = 0u32;
        let mut vbo = 0u32;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = 6 * std::mem::size_of::<f32>() as i32;

            // Position attribute (location 0)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

            // Color attribute (location 1)
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as *const _,
            );

            gl::BindVertexArray(0);
        }

        // 6 floats per vertex, 2 vertices per line
        Ok((vao, vbo, (vertices.len() / 6) as i32))
    }

    fn create_axis_markers(size: f32) -> Result<(u32, u32, i32), String> {
        // Each vertex: position (3) + color (3) = 6 floats
        // Create colored axis lines: X=red, Y=green, Z=blue
        let vertices: Vec<f32> = vec![
            // X axis (red)
            0.0, 0.0, 0.0, 1.0, 0.2, 0.2, size, 0.0, 0.0, 1.0, 0.2, 0.2, // Negative X
            0.0, 0.0, 0.0, 0.5, 0.1, 0.1, -size, 0.0, 0.0, 0.5, 0.1, 0.1,
            // Y axis (green)
            0.0, 0.0, 0.0, 0.2, 1.0, 0.2, 0.0, size, 0.0, 0.2, 1.0, 0.2, // Negative Y
            0.0, 0.0, 0.0, 0.1, 0.5, 0.1, 0.0, -size, 0.0, 0.1, 0.5, 0.1, // Z axis (blue)
            0.0, 0.0, 0.0, 0.2, 0.2, 1.0, 0.0, 0.0, size, 0.2, 0.2, 1.0, // Negative Z
            0.0, 0.0, 0.0, 0.1, 0.1, 0.5, 0.0, 0.0, -size, 0.1, 0.1, 0.5,
            // Origin marker (small cross)
            -0.2, 0.0, 0.0, 1.0, 1.0, 1.0, 0.2, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, -0.2, 0.0, 1.0, 1.0,
            1.0, 0.0, 0.2, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, -0.2, 1.0, 1.0, 1.0, 0.0, 0.0, 0.2, 1.0,
            1.0, 1.0,
        ];

        let mut vao = 0u32;
        let mut vbo = 0u32;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = 6 * std::mem::size_of::<f32>() as i32;

            // Position attribute (location 0)
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

            // Color attribute (location 1)
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as *const _,
            );

            gl::BindVertexArray(0);
        }

        Ok((vao, vbo, (vertices.len() / 6) as i32))
    }

    /// Create a primitive object
    pub fn create_primitive(&mut self, info: PrimitiveCreateInfo) -> u32 {
        let vertices = match info.primitive_type {
            PrimitiveType::Cube => {
                Self::generate_cube_vertices(info.width, info.height, info.depth)
            }
            PrimitiveType::Plane => Self::generate_plane_vertices(info.width, info.depth),
            PrimitiveType::Sphere => {
                Self::generate_sphere_vertices(info.width / 2.0, info.segments)
            }
            PrimitiveType::Cylinder => {
                Self::generate_cylinder_vertices(info.width / 2.0, info.height, info.segments)
            }
        };

        let mut vao = 0u32;
        let mut vbo = 0u32;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = 8 * std::mem::size_of::<f32>() as i32;

            // Position
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

            // Normal
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as *const _,
            );

            // Texcoord
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (6 * std::mem::size_of::<f32>()) as *const _,
            );

            gl::BindVertexArray(0);
        }

        let id = self.next_object_id;
        self.next_object_id += 1;

        self.objects.insert(
            id,
            Object3D {
                vao,
                vbo,
                vertex_count: (vertices.len() / 8) as i32,
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                texture_id: info.texture_id,
            },
        );

        id
    }

    fn generate_cube_vertices(width: f32, height: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let h = height / 2.0;
        let d = depth / 2.0;

        vec![
            // Front face (z+)
            -w, -h, d, 0.0, 0.0, 1.0, 0.0, 0.0, w, -h, d, 0.0, 0.0, 1.0, 1.0, 0.0, w, h, d, 0.0,
            0.0, 1.0, 1.0, 1.0, w, h, d, 0.0, 0.0, 1.0, 1.0, 1.0, -w, h, d, 0.0, 0.0, 1.0, 0.0,
            1.0, -w, -h, d, 0.0, 0.0, 1.0, 0.0, 0.0, // Back face (z-)
            w, -h, -d, 0.0, 0.0, -1.0, 0.0, 0.0, -w, -h, -d, 0.0, 0.0, -1.0, 1.0, 0.0, -w, h, -d,
            0.0, 0.0, -1.0, 1.0, 1.0, -w, h, -d, 0.0, 0.0, -1.0, 1.0, 1.0, w, h, -d, 0.0, 0.0,
            -1.0, 0.0, 1.0, w, -h, -d, 0.0, 0.0, -1.0, 0.0, 0.0, // Left face (x-)
            -w, -h, -d, -1.0, 0.0, 0.0, 0.0, 0.0, -w, -h, d, -1.0, 0.0, 0.0, 1.0, 0.0, -w, h, d,
            -1.0, 0.0, 0.0, 1.0, 1.0, -w, h, d, -1.0, 0.0, 0.0, 1.0, 1.0, -w, h, -d, -1.0, 0.0,
            0.0, 0.0, 1.0, -w, -h, -d, -1.0, 0.0, 0.0, 0.0, 0.0, // Right face (x+)
            w, -h, d, 1.0, 0.0, 0.0, 0.0, 0.0, w, -h, -d, 1.0, 0.0, 0.0, 1.0, 0.0, w, h, -d, 1.0,
            0.0, 0.0, 1.0, 1.0, w, h, -d, 1.0, 0.0, 0.0, 1.0, 1.0, w, h, d, 1.0, 0.0, 0.0, 0.0,
            1.0, w, -h, d, 1.0, 0.0, 0.0, 0.0, 0.0, // Bottom face (y-)
            -w, -h, -d, 0.0, -1.0, 0.0, 0.0, 0.0, w, -h, -d, 0.0, -1.0, 0.0, 1.0, 0.0, w, -h, d,
            0.0, -1.0, 0.0, 1.0, 1.0, w, -h, d, 0.0, -1.0, 0.0, 1.0, 1.0, -w, -h, d, 0.0, -1.0,
            0.0, 0.0, 1.0, -w, -h, -d, 0.0, -1.0, 0.0, 0.0, 0.0, // Top face (y+)
            -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0, w, h, d, 0.0, 1.0, 0.0, 1.0, 0.0, w, h, -d, 0.0, 1.0,
            0.0, 1.0, 1.0, w, h, -d, 0.0, 1.0, 0.0, 1.0, 1.0, -w, h, -d, 0.0, 1.0, 0.0, 0.0, 1.0,
            -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0,
        ]
    }

    fn generate_plane_vertices(width: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let d = depth / 2.0;

        vec![
            // Top face (facing up in Y direction) - CCW when viewed from above
            -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0, w, 0.0, d, 0.0, 1.0, 0.0, 1.0, 1.0, w, 0.0, -d,
            0.0, 1.0, 0.0, 1.0, 0.0, w, 0.0, -d, 0.0, 1.0, 0.0, 1.0, 0.0, -w, 0.0, -d, 0.0, 1.0,
            0.0, 0.0, 0.0, -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0,
            // Bottom face (facing down) - for double-sided rendering
            -w, 0.0, -d, 0.0, -1.0, 0.0, 0.0, 0.0, w, 0.0, -d, 0.0, -1.0, 0.0, 1.0, 0.0, w, 0.0, d,
            0.0, -1.0, 0.0, 1.0, 1.0, w, 0.0, d, 0.0, -1.0, 0.0, 1.0, 1.0, -w, 0.0, d, 0.0, -1.0,
            0.0, 0.0, 1.0, -w, 0.0, -d, 0.0, -1.0, 0.0, 0.0, 0.0,
        ]
    }

    fn generate_sphere_vertices(radius: f32, segments: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let segment_count = segments.max(8);

        for i in 0..segment_count {
            let lat0 = std::f32::consts::PI * (-0.5 + (i as f32) / segment_count as f32);
            let lat1 = std::f32::consts::PI * (-0.5 + ((i + 1) as f32) / segment_count as f32);

            for j in 0..segment_count {
                let lng0 = 2.0 * std::f32::consts::PI * (j as f32) / segment_count as f32;
                let lng1 = 2.0 * std::f32::consts::PI * ((j + 1) as f32) / segment_count as f32;

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

                // First triangle
                vertices.extend_from_slice(&[
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    j as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                ]);
                vertices.extend_from_slice(&[
                    x1,
                    y1,
                    z1,
                    x1 / radius,
                    y1 / radius,
                    z1 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                ]);
                vertices.extend_from_slice(&[
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                ]);

                // Second triangle
                vertices.extend_from_slice(&[
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    j as f32 / segment_count as f32,
                    i as f32 / segment_count as f32,
                ]);
                vertices.extend_from_slice(&[
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    (j + 1) as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                ]);
                vertices.extend_from_slice(&[
                    x3,
                    y3,
                    z3,
                    x3 / radius,
                    y3 / radius,
                    z3 / radius,
                    j as f32 / segment_count as f32,
                    (i + 1) as f32 / segment_count as f32,
                ]);
            }
        }

        vertices
    }

    fn generate_cylinder_vertices(radius: f32, height: f32, segments: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let segment_count = segments.max(8);
        let h = height / 2.0;

        for i in 0..segment_count {
            let angle0 = 2.0 * std::f32::consts::PI * (i as f32) / segment_count as f32;
            let angle1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segment_count as f32;

            let x0 = radius * angle0.cos();
            let z0 = radius * angle0.sin();
            let x1 = radius * angle1.cos();
            let z1 = radius * angle1.sin();

            // Side faces
            vertices.extend_from_slice(&[
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                0.0,
            ]);
            vertices.extend_from_slice(&[
                x1,
                -h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                0.0,
            ]);
            vertices.extend_from_slice(&[
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                1.0,
            ]);

            vertices.extend_from_slice(&[
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                0.0,
            ]);
            vertices.extend_from_slice(&[
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                (i + 1) as f32 / segment_count as f32,
                1.0,
            ]);
            vertices.extend_from_slice(&[
                x0,
                h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                i as f32 / segment_count as f32,
                1.0,
            ]);

            // Top cap
            vertices.extend_from_slice(&[0.0, h, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5]);
            vertices.extend_from_slice(&[
                x1,
                h,
                z1,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * angle1.cos(),
                0.5 + 0.5 * angle1.sin(),
            ]);
            vertices.extend_from_slice(&[
                x0,
                h,
                z0,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * angle0.cos(),
                0.5 + 0.5 * angle0.sin(),
            ]);

            // Bottom cap
            vertices.extend_from_slice(&[0.0, -h, 0.0, 0.0, -1.0, 0.0, 0.5, 0.5]);
            vertices.extend_from_slice(&[
                x0,
                -h,
                z0,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * angle0.cos(),
                0.5 + 0.5 * angle0.sin(),
            ]);
            vertices.extend_from_slice(&[
                x1,
                -h,
                z1,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * angle1.cos(),
                0.5 + 0.5 * angle1.sin(),
            ]);
        }

        vertices
    }

    /// Set object position
    pub fn set_object_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.position = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set object rotation (in degrees)
    pub fn set_object_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.rotation = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set object scale
    pub fn set_object_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.scale = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Remove an object
    pub fn remove_object(&mut self, id: u32) -> bool {
        self.objects.remove(&id).is_some()
    }

    /// Add a light
    pub fn add_light(&mut self, light: Light) -> u32 {
        let id = self.next_light_id;
        self.next_light_id += 1;
        self.lights.insert(id, light);
        id
    }

    /// Update a light
    pub fn update_light(&mut self, id: u32, light: Light) -> bool {
        if self.lights.contains_key(&id) {
            self.lights.insert(id, light);
            true
        } else {
            false
        }
    }

    /// Remove a light
    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id).is_some()
    }

    /// Set camera position
    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera.position = Vector3::new(x, y, z);
    }

    /// Set camera rotation (pitch, yaw, roll in degrees)
    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.camera.rotation = Vector3::new(pitch, yaw, roll);
    }

    /// Configure grid
    pub fn configure_grid(&mut self, config: GridConfig) {
        // Regenerate grid mesh if size or divisions changed
        if config.size != self.grid_config.size || config.divisions != self.grid_config.divisions {
            unsafe {
                gl::DeleteVertexArrays(1, &self.grid_vao);
                gl::DeleteBuffers(1, &self.grid_vbo);
            }
            if let Ok((vao, vbo, count)) =
                Self::create_grid_mesh_with_planes(config.size, config.divisions)
            {
                self.grid_vao = vao;
                self.grid_vbo = vbo;
                self.grid_vertex_count = count;
            }
        }
        self.grid_config = config;
    }

    /// Set grid enabled state
    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_config.enabled = enabled;
    }

    /// Configure skybox
    pub fn configure_skybox(&mut self, config: SkyboxConfig) {
        self.skybox_config = config;
    }

    /// Configure fog
    pub fn configure_fog(&mut self, config: FogConfig) {
        self.fog_config = config;
    }

    /// Set fog enabled state
    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_config.enabled = enabled;
    }

    /// Render the scene
    pub fn render(&self, texture_manager: Option<&dyn TextureManagerTrait>) {
        unsafe {
            // Enable depth testing
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            // Enable back-face culling
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            // Clear with skybox color if enabled
            if self.skybox_config.enabled {
                gl::ClearColor(
                    self.skybox_config.color.x,
                    self.skybox_config.color.y,
                    self.skybox_config.color.z,
                    self.skybox_config.color.w,
                );
            }
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            // Create projection matrix
            let aspect = self.window_width as f32 / self.window_height as f32;
            let projection: Matrix4<f32> = perspective(Deg(45.0), aspect, 0.1, 1000.0);

            // Get view matrix from camera
            let view = self.camera.view_matrix();

            // Render grid first
            if self.grid_config.enabled {
                self.render_grid(&view, &projection);
            }

            // Use main shader
            gl::UseProgram(self.shader_program);

            // Set common uniforms
            gl::UniformMatrix4fv(self.u_view, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.u_projection, 1, gl::FALSE, projection.as_ptr());
            gl::Uniform3f(
                self.u_view_pos,
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            );

            // Set fog uniforms
            gl::Uniform1i(
                self.u_fog_enabled,
                if self.fog_config.enabled { 1 } else { 0 },
            );
            gl::Uniform3f(
                self.u_fog_color,
                self.fog_config.color.x,
                self.fog_config.color.y,
                self.fog_config.color.z,
            );
            gl::Uniform1f(self.u_fog_density, self.fog_config.density);

            // Set light uniforms
            let light_count = self.lights.len().min(MAX_LIGHTS) as i32;
            gl::Uniform1i(self.u_num_lights, light_count);

            for (i, (_, light)) in self.lights.iter().enumerate().take(MAX_LIGHTS) {
                let prefix = format!("lights[{}]", i);

                let loc =
                    Self::get_uniform_location(self.shader_program, &format!("{}.type", prefix));
                gl::Uniform1i(loc, light.light_type as i32);

                let loc = Self::get_uniform_location(
                    self.shader_program,
                    &format!("{}.position", prefix),
                );
                gl::Uniform3f(loc, light.position.x, light.position.y, light.position.z);

                let loc = Self::get_uniform_location(
                    self.shader_program,
                    &format!("{}.direction", prefix),
                );
                gl::Uniform3f(loc, light.direction.x, light.direction.y, light.direction.z);

                let loc =
                    Self::get_uniform_location(self.shader_program, &format!("{}.color", prefix));
                gl::Uniform3f(loc, light.color.x, light.color.y, light.color.z);

                let loc = Self::get_uniform_location(
                    self.shader_program,
                    &format!("{}.intensity", prefix),
                );
                gl::Uniform1f(loc, light.intensity);

                let loc =
                    Self::get_uniform_location(self.shader_program, &format!("{}.range", prefix));
                gl::Uniform1f(loc, light.range);

                let loc = Self::get_uniform_location(
                    self.shader_program,
                    &format!("{}.spotAngle", prefix),
                );
                gl::Uniform1f(loc, light.spot_angle);

                let loc =
                    Self::get_uniform_location(self.shader_program, &format!("{}.enabled", prefix));
                gl::Uniform1i(loc, if light.enabled { 1 } else { 0 });
            }

            // Set texture sampler
            gl::Uniform1i(self.u_texture1, 0);

            // Render objects
            for (_, obj) in &self.objects {
                // Create model matrix
                let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
                gl::UniformMatrix4fv(self.u_model, 1, gl::FALSE, model.as_ptr());

                // Handle texture
                if obj.texture_id > 0 {
                    if let Some(tm) = texture_manager {
                        tm.bind_texture(obj.texture_id, 0);
                    }
                    gl::Uniform1i(self.u_use_texture, 1);
                    gl::Uniform4f(self.u_object_color, 1.0, 1.0, 1.0, 1.0);
                } else {
                    gl::Uniform1i(self.u_use_texture, 0);
                    gl::Uniform4f(self.u_object_color, 0.8, 0.8, 0.8, 1.0);
                }

                gl::BindVertexArray(obj.vao);
                gl::DrawArrays(gl::TRIANGLES, 0, obj.vertex_count);
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    fn render_grid(&self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        unsafe {
            gl::UseProgram(self.grid_shader);

            gl::UniformMatrix4fv(self.ug_view, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.ug_projection, 1, gl::FALSE, projection.as_ptr());
            gl::Uniform3f(
                self.ug_view_pos,
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            );
            gl::Uniform1f(self.ug_alpha, 0.4);

            // Fog uniforms
            gl::Uniform1i(
                self.ug_fog_enabled,
                if self.fog_config.enabled { 1 } else { 0 },
            );
            gl::Uniform3f(
                self.ug_fog_color,
                self.fog_config.color.x,
                self.fog_config.color.y,
                self.fog_config.color.z,
            );
            gl::Uniform1f(self.ug_fog_density, self.fog_config.density);

            // Enable blending for grid
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Disable depth writing but keep depth test for grid
            gl::DepthMask(gl::FALSE);

            // Draw grid planes
            gl::BindVertexArray(self.grid_vao);
            gl::DrawArrays(gl::LINES, 0, self.grid_vertex_count);

            // Draw axis markers (thicker lines)
            gl::LineWidth(3.0);
            gl::BindVertexArray(self.axis_vao);
            gl::Uniform1f(self.ug_alpha, 1.0); // Axis markers are fully opaque
            gl::DrawArrays(gl::LINES, 0, self.axis_vertex_count);
            gl::LineWidth(1.0);

            gl::BindVertexArray(0);

            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
            gl::UseProgram(0);
        }
    }

    fn create_model_matrix(
        position: Vector3<f32>,
        rotation: Vector3<f32>,
        scale: Vector3<f32>,
    ) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(position);

        let rot_x = Matrix4::from_angle_x(Deg(rotation.x));
        let rot_y = Matrix4::from_angle_y(Deg(rotation.y));
        let rot_z = Matrix4::from_angle_z(Deg(rotation.z));
        let rotation_matrix = rot_z * rot_y * rot_x;

        let scale_matrix = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        translation * rotation_matrix * scale_matrix
    }
}

impl Drop for Renderer3D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteProgram(self.grid_shader);
            gl::DeleteVertexArrays(1, &self.grid_vao);
            gl::DeleteBuffers(1, &self.grid_vbo);
        }
    }
}

/// Trait for texture manager integration
pub trait TextureManagerTrait {
    fn bind_texture(&self, texture_id: u32, slot: u32);
}
