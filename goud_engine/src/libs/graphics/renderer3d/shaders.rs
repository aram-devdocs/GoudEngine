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
out vec4 FragPosLightSpace;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 lightSpaceMatrix;

void main()
{
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoord = aTexCoord;
    FragPosLightSpace = lightSpaceMatrix * vec4(FragPos, 1.0);

    gl_Position = projection * view * vec4(FragPos, 1.0);
}
"#;

pub(super) const FRAGMENT_SHADER_3D: &str = r#"
#version 330 core

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoord;
in vec4 FragPosLightSpace;

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
uniform sampler2D shadowMap;
uniform vec3 viewPos;
uniform int numLights;
uniform Light lights[8];
uniform bool useTexture;
uniform vec4 objectColor;
uniform bool shadowsEnabled;
uniform float shadowBias;

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

float calculateShadowFactor() {
    if (!shadowsEnabled) {
        return 0.0;
    }

    vec3 projCoords = FragPosLightSpace.xyz / max(FragPosLightSpace.w, 0.0001);
    projCoords = projCoords * 0.5 + 0.5;
    if (projCoords.x < 0.0 || projCoords.x > 1.0 || projCoords.y < 0.0 || projCoords.y > 1.0 || projCoords.z > 1.0) {
        return 0.0;
    }

    float currentDepth = projCoords.z;
    float shadow = 0.0;
    vec2 texelSize = 1.0 / vec2(textureSize(shadowMap, 0));
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            float closestDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - shadowBias > closestDepth ? 1.0 : 0.0;
        }
    }
    return shadow / 9.0;
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

    float shadow = calculateShadowFactor();
    vec3 finalColor = (1.0 - 0.65 * shadow) * result * baseColor.rgb;

    if (fogEnabled) {
        float distance = length(FragPos - viewPos);
        float visibility = calculateFog(distance);
        finalColor = mix(fogColor, finalColor, visibility);
    }

    FragColor = vec4(finalColor, baseColor.a);
}
"#;

pub(super) const INSTANCED_VERTEX_SHADER_3D: &str = r#"
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;
layout (location = 3) in vec4 iModelCol0;
layout (location = 4) in vec4 iModelCol1;
layout (location = 5) in vec4 iModelCol2;
layout (location = 6) in vec4 iModelCol3;
layout (location = 7) in vec4 iColor;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoord;
out vec4 InstanceColor;
out vec4 FragPosLightSpace;

uniform mat4 view;
uniform mat4 projection;
uniform mat4 lightSpaceMatrix;

void main()
{
    mat4 model = mat4(iModelCol0, iModelCol1, iModelCol2, iModelCol3);
    vec4 worldPos = model * vec4(aPos, 1.0);
    FragPos = worldPos.xyz;
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoord = aTexCoord;
    InstanceColor = iColor;
    FragPosLightSpace = lightSpaceMatrix * worldPos;
    gl_Position = projection * view * worldPos;
}
"#;

pub(super) const INSTANCED_FRAGMENT_SHADER_3D: &str = r#"
#version 330 core

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoord;
in vec4 InstanceColor;
in vec4 FragPosLightSpace;

out vec4 FragColor;

struct Light {
    int type;
    vec3 position;
    vec3 direction;
    vec3 color;
    float intensity;
    float range;
    float spotAngle;
    bool enabled;
};

uniform sampler2D texture1;
uniform sampler2D shadowMap;
uniform vec3 viewPos;
uniform int numLights;
uniform Light lights[8];
uniform bool useTexture;
uniform vec4 objectColor;
uniform bool fogEnabled;
uniform vec3 fogColor;
uniform float fogDensity;
uniform bool shadowsEnabled;
uniform float shadowBias;

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

float calculateFog(float distance) {
    float fog = exp(-pow(fogDensity * distance, 2.0));
    return clamp(fog, 0.0, 1.0);
}

float calculateShadowFactor() {
    if (!shadowsEnabled) {
        return 0.0;
    }

    vec3 projCoords = FragPosLightSpace.xyz / max(FragPosLightSpace.w, 0.0001);
    projCoords = projCoords * 0.5 + 0.5;
    if (projCoords.x < 0.0 || projCoords.x > 1.0 || projCoords.y < 0.0 || projCoords.y > 1.0 || projCoords.z > 1.0) {
        return 0.0;
    }

    float currentDepth = projCoords.z;
    float shadow = 0.0;
    vec2 texelSize = 1.0 / vec2(textureSize(shadowMap, 0));
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            float closestDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - shadowBias > closestDepth ? 1.0 : 0.0;
        }
    }
    return shadow / 9.0;
}

void main() {
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 result = vec3(0.1);

    for (int i = 0; i < numLights && i < 8; i++) {
        if (!lights[i].enabled) continue;

        vec3 lightContribution;
        if (lights[i].type == 0) {
            lightContribution = calculatePointLight(lights[i], norm, FragPos, viewDir);
        } else if (lights[i].type == 1) {
            lightContribution = calculateDirectionalLight(lights[i], norm, viewDir);
        } else {
            lightContribution = calculateSpotLight(lights[i], norm, FragPos, viewDir);
        }
        result += lightContribution;
    }

    vec4 baseColor = useTexture ? texture(texture1, TexCoord) : objectColor;
    baseColor *= InstanceColor;
    float shadow = calculateShadowFactor();
    vec3 finalColor = (1.0 - 0.65 * shadow) * result * baseColor.rgb;

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

pub(super) const INSTANCED_VERTEX_SHADER_3D_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
    @location(7) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let model = mat4x4<f32>(input.model_0, input.model_1, input.model_2, input.model_3);
    output.clip_position = model * vec4<f32>(input.position, 1.0);
    output.normal = input.normal;
    output.color = input.color;
    return output;
}
"#;

pub(super) const INSTANCED_FRAGMENT_SHADER_3D_WGSL: &str = r#"
@fragment
fn main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
) -> @location(0) vec4<f32> {
    let lit = 0.35 + 0.65 * abs(normalize(normal).z);
    return vec4<f32>(lit * color.rgb, color.a);
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

pub(super) const POSTPROCESS_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;
out vec2 TexCoord;
void main() {
    TexCoord = aTexCoord;
    gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

pub(super) const POSTPROCESS_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 TexCoord;
out vec4 FragColor;
uniform sampler2D screenTexture;
void main() {
    FragColor = texture(screenTexture, TexCoord);
}
"#;

pub(super) const POSTPROCESS_VERTEX_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};
@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    return output;
}
"#;

pub(super) const POSTPROCESS_FRAGMENT_SHADER_WGSL: &str = r#"
@group(1) @binding(0) var screen_texture: texture_2d<f32>;
@group(1) @binding(1) var screen_sampler: sampler;
@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(screen_texture, screen_sampler, tex_coord);
}
"#;

// ============================================================================
// Cached uniform location structs
// ============================================================================

#[derive(Clone)]
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

#[derive(Clone)]
pub(super) struct MainUniforms {
    pub(super) model: i32,
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) light_space_matrix: i32,
    pub(super) view_pos: i32,
    pub(super) num_lights: i32,
    pub(super) use_texture: i32,
    pub(super) object_color: i32,
    pub(super) texture1: i32,
    pub(super) shadow_map: i32,
    pub(super) shadows_enabled: i32,
    pub(super) shadow_bias: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) lights: Vec<LightUniforms>,
}

#[derive(Clone)]
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
        light_space_matrix: loc("lightSpaceMatrix"),
        view_pos: loc("viewPos"),
        num_lights: loc("numLights"),
        use_texture: loc("useTexture"),
        object_color: loc("objectColor"),
        texture1: loc("texture1"),
        shadow_map: loc("shadowMap"),
        shadows_enabled: loc("shadowsEnabled"),
        shadow_bias: loc("shadowBias"),
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
