//! 3D Renderer Module
//!
//! Provides a complete 3D rendering system with:
//! - Primitive creation (cubes, spheres, planes, cylinders)
//! - Multiple light types (point, directional, spot)
//! - Camera control with position and rotation
//! - Grid rendering
//! - Skybox support
//!
//! All GPU operations go through the [`RenderBackend`] trait, enabling
//! backend-agnostic rendering (OpenGL, wgpu, etc.).

use crate::libs::graphics::backend::{
    BlendFactor, CullFace, DepthFunc, FrontFace, PrimitiveTopology, RenderBackend, ShaderHandle,
    VertexAttribute, VertexAttributeType, VertexLayout,
};
use cgmath::{perspective, Deg, Matrix, Matrix4, Rad, Vector3, Vector4};
use std::collections::HashMap;

use super::backend::BufferHandle;

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
#[allow(missing_docs)]
pub enum PrimitiveType {
    /// A cube primitive
    Cube = 0,
    /// A sphere primitive
    Sphere = 1,
    /// A plane primitive
    Plane = 2,
    /// A cylinder primitive
    Cylinder = 3,
}

/// Type of light source
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum LightType {
    /// Point light (emits in all directions)
    Point = 0,
    /// Directional light (parallel rays)
    Directional = 1,
    /// Spot light (cone of light)
    Spot = 2,
}

/// Grid render mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum GridRenderMode {
    /// Blend grid with scene
    Blend = 0,
    /// Overlap grid on scene
    Overlap = 1,
}

/// Primitive creation info
#[repr(C)]
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct PrimitiveCreateInfo {
    /// Type of primitive to create
    pub primitive_type: PrimitiveType,
    /// Width of the primitive
    pub width: f32,
    /// Height of the primitive
    pub height: f32,
    /// Depth of the primitive
    pub depth: f32,
    /// Number of segments (for spheres/cylinders)
    pub segments: u32,
    /// Texture ID to apply
    pub texture_id: u32,
}

/// A 3D object in the scene
#[derive(Debug)]
struct Object3D {
    buffer: BufferHandle,
    vertex_count: i32,
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
    texture_id: u32,
}

/// A light in the scene
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Light {
    /// Type of light
    pub light_type: LightType,
    /// Position in world space
    pub position: Vector3<f32>,
    /// Direction (for directional/spot lights)
    pub direction: Vector3<f32>,
    /// Light color (RGB)
    pub color: Vector3<f32>,
    /// Light intensity
    pub intensity: f32,
    /// Light range (for point/spot lights)
    pub range: f32,
    /// Spot light angle in degrees
    pub spot_angle: f32,
    /// Whether the light is enabled
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
#[allow(missing_docs)]
pub struct GridConfig {
    /// Whether grid is enabled
    pub enabled: bool,
    /// Size of the grid
    pub size: f32,
    /// Number of divisions
    pub divisions: u32,
    /// Show XZ plane
    pub show_xz_plane: bool,
    /// Show XY plane
    pub show_xy_plane: bool,
    /// Show YZ plane
    pub show_yz_plane: bool,
    /// Render mode
    pub render_mode: GridRenderMode,
    /// Line color
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
#[allow(missing_docs)]
pub struct SkyboxConfig {
    /// Whether skybox is enabled
    pub enabled: bool,
    /// Skybox color (RGBA)
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
#[allow(missing_docs)]
pub struct FogConfig {
    /// Whether fog is enabled
    pub enabled: bool,
    /// Fog color (RGB)
    pub color: Vector3<f32>,
    /// Fog density
    pub density: f32,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Vector3::new(0.05, 0.05, 0.1),
            density: 0.02,
        }
    }
}

/// 3D Camera
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Camera3D {
    /// Camera position in world space
    pub position: Vector3<f32>,
    /// Camera rotation (pitch, yaw, roll) in degrees
    pub rotation: Vector3<f32>,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 10.0, -20.0),
            rotation: Vector3::new(-30.0, 0.0, 0.0),
        }
    }
}

impl Camera3D {
    /// Get the view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let pitch = Rad::from(Deg(self.rotation.x));
        let yaw = Rad::from(Deg(self.rotation.y));

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
    
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * light.color;
    
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = spec * light.color * 0.5;
    
    float rangeAttenuation = 1.0 - smoothstep(0.0, light.range, distance);
    
    return (diffuse + specular) * attenuation * rangeAttenuation * light.intensity;
}

vec3 calculateDirectionalLight(Light light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * light.color;
    
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

uniform bool fogEnabled;
uniform vec3 fogColor;
uniform float fogDensity;

float calculateFog(float distance) {
    float fog = exp(-pow(fogDensity * distance, 2.0));
    return clamp(fog, 0.0, 1.0);
}

void main() {
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 result = vec3(0.0);
    
    vec3 ambient = vec3(0.1);
    result += ambient;
    
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
    
    if (fogEnabled) {
        float distance = length(FragPos - viewPos);
        float visibility = calculateFog(distance);
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
    
    if (fogEnabled) {
        float distance = length(viewPos - fragPos);
        float fogFactor = exp(-fogDensity * distance);
        fogFactor = clamp(fogFactor, 0.0, 1.0);
        color = mix(fogColor, color, fogFactor);
    }
    
    FragColor = vec4(color, alpha);
}
"#;

// ============================================================================
// Cached uniform locations
// ============================================================================

struct LightUniforms {
    light_type: i32,
    position: i32,
    direction: i32,
    color: i32,
    intensity: i32,
    range: i32,
    spot_angle: i32,
    enabled: i32,
}

struct MainUniforms {
    model: i32,
    view: i32,
    projection: i32,
    view_pos: i32,
    num_lights: i32,
    use_texture: i32,
    object_color: i32,
    texture1: i32,
    fog_enabled: i32,
    fog_color: i32,
    fog_density: i32,
    lights: Vec<LightUniforms>,
}

struct GridUniforms {
    view: i32,
    projection: i32,
    view_pos: i32,
    fog_enabled: i32,
    fog_color: i32,
    fog_density: i32,
    alpha: i32,
}

fn resolve_main_uniforms(backend: &dyn RenderBackend, shader: ShaderHandle) -> MainUniforms {
    let loc = |name: &str| -> i32 { backend.get_uniform_location(shader, name).unwrap_or(-1) };

    let mut lights = Vec::with_capacity(MAX_LIGHTS);
    for i in 0..MAX_LIGHTS {
        lights.push(LightUniforms {
            light_type: loc(&format!("lights[{i}].type")),
            position: loc(&format!("lights[{i}].position")),
            direction: loc(&format!("lights[{i}].direction")),
            color: loc(&format!("lights[{i}].color")),
            intensity: loc(&format!("lights[{i}].intensity")),
            range: loc(&format!("lights[{i}].range")),
            spot_angle: loc(&format!("lights[{i}].spotAngle")),
            enabled: loc(&format!("lights[{i}].enabled")),
        });
    }

    MainUniforms {
        model: loc("model"),
        view: loc("view"),
        projection: loc("projection"),
        view_pos: loc("viewPos"),
        num_lights: loc("numLights"),
        use_texture: loc("useTexture"),
        object_color: loc("objectColor"),
        texture1: loc("texture1"),
        fog_enabled: loc("fogEnabled"),
        fog_color: loc("fogColor"),
        fog_density: loc("fogDensity"),
        lights,
    }
}

fn resolve_grid_uniforms(backend: &dyn RenderBackend, shader: ShaderHandle) -> GridUniforms {
    let loc = |name: &str| -> i32 { backend.get_uniform_location(shader, name).unwrap_or(-1) };

    GridUniforms {
        view: loc("view"),
        projection: loc("projection"),
        view_pos: loc("viewPos"),
        fog_enabled: loc("fogEnabled"),
        fog_color: loc("fogColor"),
        fog_density: loc("fogDensity"),
        alpha: loc("alpha"),
    }
}

// ============================================================================
// Vertex layouts
// ============================================================================

fn object_vertex_layout() -> VertexLayout {
    // position (3 floats) + normal (3 floats) + texcoord (2 floats) = 32 bytes
    VertexLayout::new(32)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float3,
            12,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            2,
            VertexAttributeType::Float2,
            24,
            false,
        ))
}

fn grid_vertex_layout() -> VertexLayout {
    // position (3 floats) + color (3 floats) = 24 bytes
    VertexLayout::new(24)
        .with_attribute(VertexAttribute::new(
            0,
            VertexAttributeType::Float3,
            0,
            false,
        ))
        .with_attribute(VertexAttribute::new(
            1,
            VertexAttributeType::Float3,
            12,
            false,
        ))
}

// ============================================================================
// Renderer3D
// ============================================================================

/// The main 3D renderer.
///
/// Owns a [`RenderBackend`] and manages all GPU resources (shaders, buffers)
/// through it. No direct graphics API calls are made outside the backend.
pub struct Renderer3D {
    backend: Box<dyn RenderBackend>,
    shader_handle: ShaderHandle,
    grid_shader_handle: ShaderHandle,
    grid_buffer: BufferHandle,
    grid_vertex_count: i32,
    axis_buffer: BufferHandle,
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
    uniforms: MainUniforms,
    grid_uniforms: GridUniforms,
    object_layout: VertexLayout,
    grid_layout: VertexLayout,
}

impl Renderer3D {
    /// Create a new 3D renderer with the given backend.
    pub fn new(
        mut backend: Box<dyn RenderBackend>,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let shader_handle = backend
            .create_shader(VERTEX_SHADER_3D, FRAGMENT_SHADER_3D)
            .map_err(|e| format!("Main 3D shader: {e}"))?;
        let uniforms = resolve_main_uniforms(backend.as_ref(), shader_handle);

        let grid_shader_handle = backend
            .create_shader(GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER)
            .map_err(|e| format!("Grid shader: {e}"))?;
        let grid_uniforms = resolve_grid_uniforms(backend.as_ref(), grid_shader_handle);

        let grid_layout = grid_vertex_layout();
        let (grid_buffer, grid_vertex_count) = Self::create_grid_mesh(backend.as_mut(), 20.0, 20)?;
        let (axis_buffer, axis_vertex_count) = Self::create_axis_mesh(backend.as_mut(), 5.0)?;

        Ok(Self {
            backend,
            shader_handle,
            grid_shader_handle,
            grid_buffer,
            grid_vertex_count,
            axis_buffer,
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
            uniforms,
            grid_uniforms,
            object_layout: object_vertex_layout(),
            grid_layout,
        })
    }

    // ========================================================================
    // Mesh creation helpers (data → backend buffer)
    // ========================================================================

    fn upload_buffer(
        backend: &mut dyn RenderBackend,
        vertices: &[f32],
    ) -> Result<BufferHandle, String> {
        use crate::libs::graphics::backend::types::{BufferType, BufferUsage};
        backend
            .create_buffer(
                BufferType::Vertex,
                BufferUsage::Static,
                bytemuck::cast_slice(vertices),
            )
            .map_err(|e| format!("Buffer creation failed: {e}"))
    }

    fn create_grid_mesh(
        backend: &mut dyn RenderBackend,
        size: f32,
        divisions: u32,
    ) -> Result<(BufferHandle, i32), String> {
        let vertices = Self::generate_grid_vertices(size, divisions);
        let count = (vertices.len() / 6) as i32;
        let handle = Self::upload_buffer(backend, &vertices)?;
        Ok((handle, count))
    }

    fn create_axis_mesh(
        backend: &mut dyn RenderBackend,
        size: f32,
    ) -> Result<(BufferHandle, i32), String> {
        let vertices = Self::generate_axis_vertices(size);
        let count = (vertices.len() / 6) as i32;
        let handle = Self::upload_buffer(backend, &vertices)?;
        Ok((handle, count))
    }

    // ========================================================================
    // Vertex data generation (pure math, no GPU calls)
    // ========================================================================

    fn generate_grid_vertices(size: f32, divisions: u32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let step = size / divisions as f32;
        let half = size / 2.0;

        let xz_color = [0.3, 0.3, 0.3];
        let xy_color = [0.2, 0.25, 0.3];
        let yz_color = [0.3, 0.2, 0.25];

        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            vertices.extend_from_slice(&[pos, 0.0, -half]);
            vertices.extend_from_slice(&xz_color);
            vertices.extend_from_slice(&[pos, 0.0, half]);
            vertices.extend_from_slice(&xz_color);

            vertices.extend_from_slice(&[-half, 0.0, pos]);
            vertices.extend_from_slice(&xz_color);
            vertices.extend_from_slice(&[half, 0.0, pos]);
            vertices.extend_from_slice(&xz_color);
        }

        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            vertices.extend_from_slice(&[pos, -half, 0.0]);
            vertices.extend_from_slice(&xy_color);
            vertices.extend_from_slice(&[pos, half, 0.0]);
            vertices.extend_from_slice(&xy_color);

            vertices.extend_from_slice(&[-half, pos, 0.0]);
            vertices.extend_from_slice(&xy_color);
            vertices.extend_from_slice(&[half, pos, 0.0]);
            vertices.extend_from_slice(&xy_color);
        }

        for i in 0..=divisions {
            let pos = -half + step * i as f32;

            vertices.extend_from_slice(&[0.0, pos, -half]);
            vertices.extend_from_slice(&yz_color);
            vertices.extend_from_slice(&[0.0, pos, half]);
            vertices.extend_from_slice(&yz_color);

            vertices.extend_from_slice(&[0.0, -half, pos]);
            vertices.extend_from_slice(&yz_color);
            vertices.extend_from_slice(&[0.0, half, pos]);
            vertices.extend_from_slice(&yz_color);
        }

        vertices
    }

    #[rustfmt::skip]
    fn generate_axis_vertices(size: f32) -> Vec<f32> {
        vec![
            // X axis (red)
            0.0, 0.0, 0.0, 1.0, 0.2, 0.2,  size, 0.0, 0.0, 1.0, 0.2, 0.2,
            0.0, 0.0, 0.0, 0.5, 0.1, 0.1, -size, 0.0, 0.0, 0.5, 0.1, 0.1,
            // Y axis (green)
            0.0, 0.0, 0.0, 0.2, 1.0, 0.2,  0.0, size, 0.0, 0.2, 1.0, 0.2,
            0.0, 0.0, 0.0, 0.1, 0.5, 0.1,  0.0,-size, 0.0, 0.1, 0.5, 0.1,
            // Z axis (blue)
            0.0, 0.0, 0.0, 0.2, 0.2, 1.0,  0.0, 0.0, size, 0.2, 0.2, 1.0,
            0.0, 0.0, 0.0, 0.1, 0.1, 0.5,  0.0, 0.0,-size, 0.1, 0.1, 0.5,
            // Origin marker (small cross)
            -0.2, 0.0, 0.0, 1.0, 1.0, 1.0,  0.2, 0.0, 0.0, 1.0, 1.0, 1.0,
             0.0,-0.2, 0.0, 1.0, 1.0, 1.0,  0.0, 0.2, 0.0, 1.0, 1.0, 1.0,
             0.0, 0.0,-0.2, 1.0, 1.0, 1.0,  0.0, 0.0, 0.2, 1.0, 1.0, 1.0,
        ]
    }

    fn generate_cube_vertices(width: f32, height: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let h = height / 2.0;
        let d = depth / 2.0;

        #[rustfmt::skip]
        let v = vec![
            // Front face (z+)
            -w,-h, d, 0.0, 0.0, 1.0, 0.0, 0.0,  w,-h, d, 0.0, 0.0, 1.0, 1.0, 0.0,
             w, h, d, 0.0, 0.0, 1.0, 1.0, 1.0,   w, h, d, 0.0, 0.0, 1.0, 1.0, 1.0,
            -w, h, d, 0.0, 0.0, 1.0, 0.0, 1.0,  -w,-h, d, 0.0, 0.0, 1.0, 0.0, 0.0,
            // Back face (z-)
             w,-h,-d, 0.0, 0.0,-1.0, 0.0, 0.0,  -w,-h,-d, 0.0, 0.0,-1.0, 1.0, 0.0,
            -w, h,-d, 0.0, 0.0,-1.0, 1.0, 1.0,  -w, h,-d, 0.0, 0.0,-1.0, 1.0, 1.0,
             w, h,-d, 0.0, 0.0,-1.0, 0.0, 1.0,   w,-h,-d, 0.0, 0.0,-1.0, 0.0, 0.0,
            // Left face (x-)
            -w,-h,-d,-1.0, 0.0, 0.0, 0.0, 0.0,  -w,-h, d,-1.0, 0.0, 0.0, 1.0, 0.0,
            -w, h, d,-1.0, 0.0, 0.0, 1.0, 1.0,  -w, h, d,-1.0, 0.0, 0.0, 1.0, 1.0,
            -w, h,-d,-1.0, 0.0, 0.0, 0.0, 1.0,  -w,-h,-d,-1.0, 0.0, 0.0, 0.0, 0.0,
            // Right face (x+)
             w,-h, d, 1.0, 0.0, 0.0, 0.0, 0.0,   w,-h,-d, 1.0, 0.0, 0.0, 1.0, 0.0,
             w, h,-d, 1.0, 0.0, 0.0, 1.0, 1.0,   w, h,-d, 1.0, 0.0, 0.0, 1.0, 1.0,
             w, h, d, 1.0, 0.0, 0.0, 0.0, 1.0,   w,-h, d, 1.0, 0.0, 0.0, 0.0, 0.0,
            // Bottom face (y-)
            -w,-h,-d, 0.0,-1.0, 0.0, 0.0, 0.0,   w,-h,-d, 0.0,-1.0, 0.0, 1.0, 0.0,
             w,-h, d, 0.0,-1.0, 0.0, 1.0, 1.0,   w,-h, d, 0.0,-1.0, 0.0, 1.0, 1.0,
            -w,-h, d, 0.0,-1.0, 0.0, 0.0, 1.0,  -w,-h,-d, 0.0,-1.0, 0.0, 0.0, 0.0,
            // Top face (y+)
            -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0,   w, h, d, 0.0, 1.0, 0.0, 1.0, 0.0,
             w, h,-d, 0.0, 1.0, 0.0, 1.0, 1.0,   w, h,-d, 0.0, 1.0, 0.0, 1.0, 1.0,
            -w, h,-d, 0.0, 1.0, 0.0, 0.0, 1.0,  -w, h, d, 0.0, 1.0, 0.0, 0.0, 0.0,
        ];
        v
    }

    fn generate_plane_vertices(width: f32, depth: f32) -> Vec<f32> {
        let w = width / 2.0;
        let d = depth / 2.0;

        #[rustfmt::skip]
        let v = vec![
            // Top face
            -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0,   w, 0.0, d, 0.0, 1.0, 0.0, 1.0, 1.0,
             w, 0.0,-d, 0.0, 1.0, 0.0, 1.0, 0.0,    w, 0.0,-d, 0.0, 1.0, 0.0, 1.0, 0.0,
            -w, 0.0,-d, 0.0, 1.0, 0.0, 0.0, 0.0,   -w, 0.0, d, 0.0, 1.0, 0.0, 0.0, 1.0,
            // Bottom face (double-sided)
            -w, 0.0,-d, 0.0,-1.0, 0.0, 0.0, 0.0,    w, 0.0,-d, 0.0,-1.0, 0.0, 1.0, 0.0,
             w, 0.0, d, 0.0,-1.0, 0.0, 1.0, 1.0,    w, 0.0, d, 0.0,-1.0, 0.0, 1.0, 1.0,
            -w, 0.0, d, 0.0,-1.0, 0.0, 0.0, 1.0,   -w, 0.0,-d, 0.0,-1.0, 0.0, 0.0, 0.0,
        ];
        v
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

                let u0 = j as f32 / segment_count as f32;
                let u1 = (j + 1) as f32 / segment_count as f32;
                let v0 = i as f32 / segment_count as f32;
                let v1 = (i + 1) as f32 / segment_count as f32;

                vertices.extend_from_slice(&[
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    u0,
                    v0,
                    x1,
                    y1,
                    z1,
                    x1 / radius,
                    y1 / radius,
                    z1 / radius,
                    u1,
                    v0,
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    u1,
                    v1,
                    x0,
                    y0,
                    z0,
                    x0 / radius,
                    y0 / radius,
                    z0 / radius,
                    u0,
                    v0,
                    x2,
                    y2,
                    z2,
                    x2 / radius,
                    y2 / radius,
                    z2 / radius,
                    u1,
                    v1,
                    x3,
                    y3,
                    z3,
                    x3 / radius,
                    y3 / radius,
                    z3 / radius,
                    u0,
                    v1,
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
            let a0 = 2.0 * std::f32::consts::PI * (i as f32) / segment_count as f32;
            let a1 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / segment_count as f32;

            let x0 = radius * a0.cos();
            let z0 = radius * a0.sin();
            let x1 = radius * a1.cos();
            let z1 = radius * a1.sin();

            let u0 = i as f32 / segment_count as f32;
            let u1 = (i + 1) as f32 / segment_count as f32;

            // Side faces (two triangles)
            vertices.extend_from_slice(&[
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                u0,
                0.0,
                x1,
                -h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                u1,
                0.0,
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                u1,
                1.0,
                x0,
                -h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                u0,
                0.0,
                x1,
                h,
                z1,
                x1 / radius,
                0.0,
                z1 / radius,
                u1,
                1.0,
                x0,
                h,
                z0,
                x0 / radius,
                0.0,
                z0 / radius,
                u0,
                1.0,
            ]);

            // Top cap
            vertices.extend_from_slice(&[
                0.0,
                h,
                0.0,
                0.0,
                1.0,
                0.0,
                0.5,
                0.5,
                x1,
                h,
                z1,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * a1.cos(),
                0.5 + 0.5 * a1.sin(),
                x0,
                h,
                z0,
                0.0,
                1.0,
                0.0,
                0.5 + 0.5 * a0.cos(),
                0.5 + 0.5 * a0.sin(),
            ]);

            // Bottom cap
            vertices.extend_from_slice(&[
                0.0,
                -h,
                0.0,
                0.0,
                -1.0,
                0.0,
                0.5,
                0.5,
                x0,
                -h,
                z0,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * a0.cos(),
                0.5 + 0.5 * a0.sin(),
                x1,
                -h,
                z1,
                0.0,
                -1.0,
                0.0,
                0.5 + 0.5 * a1.cos(),
                0.5 + 0.5 * a1.sin(),
            ]);
        }

        vertices
    }

    // ========================================================================
    // Primitive creation
    // ========================================================================

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

        let buffer = match Self::upload_buffer(self.backend.as_mut(), &vertices) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to create primitive buffer: {e}");
                return 0;
            }
        };

        let id = self.next_object_id;
        self.next_object_id += 1;

        self.objects.insert(
            id,
            Object3D {
                buffer,
                vertex_count: (vertices.len() / 8) as i32,
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                texture_id: info.texture_id,
            },
        );

        id
    }

    // ========================================================================
    // Object manipulation
    // ========================================================================

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
        if let Some(obj) = self.objects.remove(&id) {
            self.backend.destroy_buffer(obj.buffer);
            true
        } else {
            false
        }
    }

    // ========================================================================
    // Lighting
    // ========================================================================

    /// Add a light
    pub fn add_light(&mut self, light: Light) -> u32 {
        let id = self.next_light_id;
        self.next_light_id += 1;
        self.lights.insert(id, light);
        id
    }

    /// Update a light
    pub fn update_light(&mut self, id: u32, light: Light) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut e) = self.lights.entry(id) {
            e.insert(light);
            true
        } else {
            false
        }
    }

    /// Remove a light
    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id).is_some()
    }

    // ========================================================================
    // Camera
    // ========================================================================

    /// Set camera position
    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera.position = Vector3::new(x, y, z);
    }

    /// Set camera rotation (pitch, yaw, roll in degrees)
    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.camera.rotation = Vector3::new(pitch, yaw, roll);
    }

    // ========================================================================
    // Grid / Skybox / Fog configuration
    // ========================================================================

    /// Configure grid
    pub fn configure_grid(&mut self, config: GridConfig) {
        if config.size != self.grid_config.size || config.divisions != self.grid_config.divisions {
            self.backend.destroy_buffer(self.grid_buffer);
            if let Ok((buf, count)) =
                Self::create_grid_mesh(self.backend.as_mut(), config.size, config.divisions)
            {
                self.grid_buffer = buf;
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

    // ========================================================================
    // Rendering
    // ========================================================================

    /// Render the scene
    pub fn render(&mut self, texture_manager: Option<&dyn TextureManagerTrait>) {
        self.backend.enable_depth_test();
        self.backend.set_depth_func(DepthFunc::Less);
        self.backend.enable_culling();
        self.backend.set_cull_face(CullFace::Back);
        self.backend.set_front_face(FrontFace::Ccw);

        if self.skybox_config.enabled {
            self.backend.set_clear_color(
                self.skybox_config.color.x,
                self.skybox_config.color.y,
                self.skybox_config.color.z,
                self.skybox_config.color.w,
            );
        }
        self.backend.clear_depth();

        let aspect = self.window_width as f32 / self.window_height as f32;
        let projection: Matrix4<f32> = perspective(Deg(45.0), aspect, 0.1, 1000.0);
        let view = self.camera.view_matrix();
        let view_arr = mat4_to_array(&view);
        let proj_arr = mat4_to_array(&projection);

        // --- Grid pass ---
        if self.grid_config.enabled {
            let _ = self.backend.bind_shader(self.grid_shader_handle);

            self.backend
                .set_uniform_mat4(self.grid_uniforms.view, &view_arr);
            self.backend
                .set_uniform_mat4(self.grid_uniforms.projection, &proj_arr);
            self.backend.set_uniform_vec3(
                self.grid_uniforms.view_pos,
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            );
            self.backend
                .set_uniform_float(self.grid_uniforms.alpha, 0.4);
            self.backend.set_uniform_int(
                self.grid_uniforms.fog_enabled,
                i32::from(self.fog_config.enabled),
            );
            self.backend.set_uniform_vec3(
                self.grid_uniforms.fog_color,
                self.fog_config.color.x,
                self.fog_config.color.y,
                self.fog_config.color.z,
            );
            self.backend
                .set_uniform_float(self.grid_uniforms.fog_density, self.fog_config.density);

            self.backend.enable_blending();
            self.backend
                .set_blend_func(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
            self.backend.set_depth_mask(false);

            let _ = self.backend.bind_buffer(self.grid_buffer);
            self.backend.set_vertex_attributes(&self.grid_layout);
            let _ = self.backend.draw_arrays(
                PrimitiveTopology::Lines,
                0,
                self.grid_vertex_count as u32,
            );

            self.backend.set_line_width(3.0);
            let _ = self.backend.bind_buffer(self.axis_buffer);
            self.backend.set_vertex_attributes(&self.grid_layout);
            self.backend
                .set_uniform_float(self.grid_uniforms.alpha, 1.0);
            let _ = self.backend.draw_arrays(
                PrimitiveTopology::Lines,
                0,
                self.axis_vertex_count as u32,
            );
            self.backend.set_line_width(1.0);

            self.backend.set_depth_mask(true);
            self.backend.disable_blending();
            self.backend.unbind_shader();
        }

        // --- Main objects pass ---
        let _ = self.backend.bind_shader(self.shader_handle);

        self.backend.set_uniform_mat4(self.uniforms.view, &view_arr);
        self.backend
            .set_uniform_mat4(self.uniforms.projection, &proj_arr);
        self.backend.set_uniform_vec3(
            self.uniforms.view_pos,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );

        self.backend.set_uniform_int(
            self.uniforms.fog_enabled,
            i32::from(self.fog_config.enabled),
        );
        self.backend.set_uniform_vec3(
            self.uniforms.fog_color,
            self.fog_config.color.x,
            self.fog_config.color.y,
            self.fog_config.color.z,
        );
        self.backend
            .set_uniform_float(self.uniforms.fog_density, self.fog_config.density);

        let light_count = self.lights.len().min(MAX_LIGHTS) as i32;
        self.backend
            .set_uniform_int(self.uniforms.num_lights, light_count);

        for (i, (_, light)) in self.lights.iter().enumerate().take(MAX_LIGHTS) {
            let lu = &self.uniforms.lights[i];
            self.backend
                .set_uniform_int(lu.light_type, light.light_type as i32);
            self.backend.set_uniform_vec3(
                lu.position,
                light.position.x,
                light.position.y,
                light.position.z,
            );
            self.backend.set_uniform_vec3(
                lu.direction,
                light.direction.x,
                light.direction.y,
                light.direction.z,
            );
            self.backend
                .set_uniform_vec3(lu.color, light.color.x, light.color.y, light.color.z);
            self.backend
                .set_uniform_float(lu.intensity, light.intensity);
            self.backend.set_uniform_float(lu.range, light.range);
            self.backend
                .set_uniform_float(lu.spot_angle, light.spot_angle);
            self.backend
                .set_uniform_int(lu.enabled, i32::from(light.enabled));
        }

        self.backend.set_uniform_int(self.uniforms.texture1, 0);

        for obj in self.objects.values() {
            let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
            let model_arr = mat4_to_array(&model);
            self.backend
                .set_uniform_mat4(self.uniforms.model, &model_arr);

            if obj.texture_id > 0 {
                if let Some(tm) = texture_manager {
                    tm.bind_texture(obj.texture_id, 0);
                }
                self.backend.set_uniform_int(self.uniforms.use_texture, 1);
                self.backend
                    .set_uniform_vec4(self.uniforms.object_color, 1.0, 1.0, 1.0, 1.0);
            } else {
                self.backend.set_uniform_int(self.uniforms.use_texture, 0);
                self.backend
                    .set_uniform_vec4(self.uniforms.object_color, 0.8, 0.8, 0.8, 1.0);
            }

            let _ = self.backend.bind_buffer(obj.buffer);
            self.backend.set_vertex_attributes(&self.object_layout);
            let _ =
                self.backend
                    .draw_arrays(PrimitiveTopology::Triangles, 0, obj.vertex_count as u32);
        }

        self.backend.unbind_shader();
    }

    // ========================================================================
    // Utility
    // ========================================================================

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
        for obj in self.objects.values() {
            self.backend.destroy_buffer(obj.buffer);
        }
        self.backend.destroy_buffer(self.grid_buffer);
        self.backend.destroy_buffer(self.axis_buffer);
        self.backend.destroy_shader(self.shader_handle);
        self.backend.destroy_shader(self.grid_shader_handle);
    }
}

/// Trait for texture manager integration.
///
/// This is a legacy bridge: texture binding still happens through the caller's
/// GL-backed implementation. A future refactor should route textures through
/// [`RenderBackend`] for full backend portability.
pub trait TextureManagerTrait {
    /// Bind a texture to a slot
    fn bind_texture(&self, texture_id: u32, slot: u32);
}

// ============================================================================
// Helpers
// ============================================================================

fn mat4_to_array(m: &Matrix4<f32>) -> [f32; 16] {
    // cgmath matrices are column-major, matching what the backend expects
    let ptr = m.as_ptr();
    let mut arr = [0.0f32; 16];
    // SAFETY: Matrix4<f32> is 16 contiguous floats in column-major order
    unsafe {
        std::ptr::copy_nonoverlapping(ptr, arr.as_mut_ptr(), 16);
    }
    arr
}
