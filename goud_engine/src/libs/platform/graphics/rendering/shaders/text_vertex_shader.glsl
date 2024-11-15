// src/shaders/text_vertex_shader.glsl

#version 330 core
layout (location = 0) in vec2 aPos; // Vertex position
layout (location = 1) in vec2 aTexCoord; // Texture coordinate

out vec2 TexCoord;

uniform mat4 projection;

void main()
{
    gl_Position = projection * vec4(aPos.xy, 0.0, 1.0);
    TexCoord = aTexCoord;
}