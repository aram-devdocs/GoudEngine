#version 330 core

out vec4 FragColor; // Output fragment color

in vec2 TexCoord;   // Texture coordinate from vertex shader

uniform sampler2D texture1; // Texture sampler
uniform vec4 outlineColor;  // Outline color (e.g., red)
uniform float outlineWidth; // Outline width

void main()
{
    // Sample the texture at the given coordinate
    vec4 texColor = texture(texture1, TexCoord);

    // Check if the pixel is close to the edge within the outline width
    if (TexCoord.x < outlineWidth || TexCoord.y < outlineWidth || 
        TexCoord.x > 1.0 - outlineWidth || TexCoord.y > 1.0 - outlineWidth) 
    {
        // Apply outline color for edge pixels
        FragColor = outlineColor;
    }
    else 
    {
        // Otherwise, use the texture color
        FragColor = texColor;
    }
}