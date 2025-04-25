#version 330 core
out vec4 FragColor;

in vec3 TexCoords;

uniform samplerCube skybox;
uniform vec3 minColor;  // Minimum color values from config

void main()
{    
    vec4 texColor = texture(skybox, TexCoords);
    // Ensure we're not outputting colors below the minimum
    vec3 color = max(texColor.rgb, minColor);
    FragColor = vec4(color, 1.0);
} 