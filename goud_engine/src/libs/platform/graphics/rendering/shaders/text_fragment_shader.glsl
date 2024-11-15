// src/shaders/text_fragment_shader.glsl

#version 330 core

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D text;
uniform vec3 textColor;

void main()
{
    float alpha = texture(text, TexCoord).r;
    FragColor = vec4(textColor, alpha);
}