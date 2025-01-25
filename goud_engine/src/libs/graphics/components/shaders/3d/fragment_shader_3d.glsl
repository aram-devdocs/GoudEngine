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
uniform Light lights[8];  // Maximum 8 lights

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
    
    vec4 texColor = texture(texture1, TexCoord);
    FragColor = vec4(result * texColor.rgb, texColor.a);
} 