using GoudEngine.Math;

namespace GoudEngine.Config;

/// <summary>
/// Configuration for the 3D skybox.
/// </summary>
public class SkyboxConfig
{
    /// <summary>
    /// Whether the skybox is enabled.
    /// </summary>
    public bool Enabled { get; set; } = true;

    /// <summary>
    /// The size of the skybox.
    /// </summary>
    public float Size { get; set; } = 100.0f;

    /// <summary>
    /// The texture size for generated skybox textures.
    /// </summary>
    public uint TextureSize { get; set; } = 128;

    /// <summary>
    /// Color of the right face (+X).
    /// </summary>
    public Color RightFaceColor { get; set; } = new(0.7f, 0.8f, 0.9f);

    /// <summary>
    /// Color of the left face (-X).
    /// </summary>
    public Color LeftFaceColor { get; set; } = new(0.7f, 0.8f, 0.9f);

    /// <summary>
    /// Color of the top face (+Y).
    /// </summary>
    public Color TopFaceColor { get; set; } = new(0.6f, 0.7f, 0.9f);

    /// <summary>
    /// Color of the bottom face (-Y).
    /// </summary>
    public Color BottomFaceColor { get; set; } = new(0.3f, 0.3f, 0.4f);

    /// <summary>
    /// Color of the front face (-Z).
    /// </summary>
    public Color FrontFaceColor { get; set; } = new(0.7f, 0.8f, 0.9f);

    /// <summary>
    /// Color of the back face (+Z).
    /// </summary>
    public Color BackFaceColor { get; set; } = new(0.7f, 0.8f, 0.9f);

    /// <summary>
    /// Blend factor for color interpolation.
    /// </summary>
    public float BlendFactor { get; set; } = 0.5f;

    /// <summary>
    /// Minimum color for the skybox gradient.
    /// </summary>
    public Color MinColor { get; set; } = new(0.1f, 0.1f, 0.2f);

    /// <summary>
    /// Whether to use custom textures instead of generated colors.
    /// </summary>
    public bool UseCustomTextures { get; set; } = false;

    /// <summary>
    /// Creates a default skybox configuration.
    /// </summary>
    public static SkyboxConfig Default => new();

    /// <summary>
    /// Creates a clear blue sky configuration.
    /// </summary>
    public static SkyboxConfig ClearSky => new()
    {
        TopFaceColor = new Color(0.4f, 0.6f, 1.0f),
        RightFaceColor = new Color(0.6f, 0.8f, 1.0f),
        LeftFaceColor = new Color(0.6f, 0.8f, 1.0f),
        FrontFaceColor = new Color(0.6f, 0.8f, 1.0f),
        BackFaceColor = new Color(0.6f, 0.8f, 1.0f),
        BottomFaceColor = new Color(0.3f, 0.4f, 0.5f)
    };

    /// <summary>
    /// Creates a sunset sky configuration.
    /// </summary>
    public static SkyboxConfig Sunset => new()
    {
        TopFaceColor = new Color(0.2f, 0.3f, 0.6f),
        RightFaceColor = new Color(1.0f, 0.5f, 0.3f),
        LeftFaceColor = new Color(0.8f, 0.4f, 0.5f),
        FrontFaceColor = new Color(1.0f, 0.6f, 0.4f),
        BackFaceColor = new Color(0.5f, 0.3f, 0.6f),
        BottomFaceColor = new Color(0.2f, 0.1f, 0.2f)
    };

    /// <summary>
    /// Creates a night sky configuration.
    /// </summary>
    public static SkyboxConfig Night => new()
    {
        TopFaceColor = new Color(0.02f, 0.02f, 0.1f),
        RightFaceColor = new Color(0.05f, 0.05f, 0.15f),
        LeftFaceColor = new Color(0.05f, 0.05f, 0.15f),
        FrontFaceColor = new Color(0.05f, 0.05f, 0.15f),
        BackFaceColor = new Color(0.05f, 0.05f, 0.15f),
        BottomFaceColor = new Color(0.01f, 0.01f, 0.05f)
    };
}
