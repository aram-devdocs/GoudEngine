#version 330 core

out vec4 FragColor; // Output fragment color

in vec2 TexCoord;   // Texture coordinate from vertex shader

uniform sampler2D texture1; // Texture sampler

void main()
{
    // Sample the texture at the given coordinate
    FragColor = texture(texture1, TexCoord);
}