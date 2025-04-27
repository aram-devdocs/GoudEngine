#version 330 core
layout (location = 0) in vec3 aPos;

out vec3 TexCoords;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    vec4 pos = projection * view * vec4(aPos, 1.0);
    gl_Position = pos.xyww;  // Force z to be 1.0 (furthest) after perspective divide
    TexCoords = aPos;  // Use position as cubemap texture coordinates
} 