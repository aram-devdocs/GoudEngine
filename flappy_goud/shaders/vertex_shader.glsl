#version 330 core

layout (location = 0) in vec3 aPos;      // Vertex position
layout (location = 1) in vec2 aTexCoord; // Texture coordinate

out vec2 TexCoord; // Pass texture coordinate to fragment shader

uniform mat4 model;
uniform vec4 sourceRect; // x, y, width, height

void main()
{
    gl_Position = model * vec4(aPos, 1.0); // Apply model transformation
    TexCoord = aTexCoord * sourceRect.zw + sourceRect.xy; // Adjust texture coordinate for spritesheet
}