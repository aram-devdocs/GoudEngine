//! Shader sources and cached uniform location structs for the 3D renderer.

use crate::libs::graphics::backend::{RenderBackend, ShaderHandle};

use super::types::MAX_LIGHTS;

// ============================================================================
// Shader sources
// ============================================================================

pub(super) const VERTEX_SHADER_3D: &str = r#"
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

pub(super) const FRAGMENT_SHADER_3D: &str = r#"
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

pub(super) const GRID_VERTEX_SHADER: &str = r#"
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

pub(super) const GRID_FRAGMENT_SHADER: &str = r#"
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

pub(super) const VERTEX_SHADER_3D_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.normal = input.normal;
    return output;
}
"#;

pub(super) const FRAGMENT_SHADER_3D_WGSL: &str = r#"
@fragment
fn main(@location(0) normal: vec3<f32>) -> @location(0) vec4<f32> {
    let lit = 0.35 + 0.65 * abs(normalize(normal).z);
    return vec4<f32>(lit, lit, lit, 1.0);
}
"#;

pub(super) const GRID_VERTEX_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}
"#;

pub(super) const GRID_FRAGMENT_SHADER_WGSL: &str = r#"
@fragment
fn main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
"#;

// ============================================================================
// Cached uniform location structs
// ============================================================================

pub(super) struct LightUniforms {
    pub(super) light_type: i32,
    pub(super) position: i32,
    pub(super) direction: i32,
    pub(super) color: i32,
    pub(super) intensity: i32,
    pub(super) range: i32,
    pub(super) spot_angle: i32,
    pub(super) enabled: i32,
}

pub(super) struct MainUniforms {
    pub(super) model: i32,
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) view_pos: i32,
    pub(super) num_lights: i32,
    pub(super) use_texture: i32,
    pub(super) object_color: i32,
    pub(super) texture1: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) lights: Vec<LightUniforms>,
}

pub(super) struct GridUniforms {
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) view_pos: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) alpha: i32,
}

pub(super) fn resolve_main_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> MainUniforms {
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

pub(super) fn resolve_grid_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> GridUniforms {
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
