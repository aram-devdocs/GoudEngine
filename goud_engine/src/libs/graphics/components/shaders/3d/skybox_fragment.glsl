#version 330 core
out vec4 FragColor;

in vec3 TexCoords;

uniform samplerCube skybox;

void main()
{    
    vec4 texColor = texture(skybox, TexCoords);
    // Ensure we're not outputting pure black
    vec3 color = max(texColor.rgb, vec3(0.1, 0.1, 0.2));
    FragColor = vec4(color, 1.0);
} 