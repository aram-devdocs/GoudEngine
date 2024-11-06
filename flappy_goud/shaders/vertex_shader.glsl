#version 330 core

layout (location = 0) in vec3 aPos;      // Vertex position
layout (location = 1) in vec2 aTexCoord; // Texture coordinate

out vec2 TexCoord; // Pass texture coordinate to fragment shader

uniform mat4 projection;
uniform mat4 model;
uniform vec4 sourceRect; // x, y, width, height

void main()
{
    gl_Position = projection * model * vec4(aPos, 1.0); // Apply projection and model transformations
    TexCoord = vec2(aTexCoord.x, 1.0 - aTexCoord.y) * sourceRect.zw + sourceRect.xy; // Flip Y-axis
}