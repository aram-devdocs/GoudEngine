using GoudEngine.Core;
using GoudEngine.Math;

namespace GoudEngine.Config;

/// <summary>
/// Configuration for creating a new sprite.
/// </summary>
public class SpriteConfig
{
    /// <summary>
    /// The texture to use for this sprite.
    /// </summary>
    public TextureId TextureId { get; set; }

    /// <summary>
    /// The position of the sprite.
    /// </summary>
    public Vector2 Position { get; set; } = Vector2.Zero;

    /// <summary>
    /// The dimensions of the sprite.
    /// </summary>
    public Vector2 Dimensions { get; set; } = new(32, 32);

    /// <summary>
    /// The scale of the sprite.
    /// </summary>
    public Vector2 Scale { get; set; } = Vector2.One;

    /// <summary>
    /// The rotation in degrees.
    /// </summary>
    public float Rotation { get; set; } = 0;

    /// <summary>
    /// The Z-layer for render ordering.
    /// </summary>
    public int ZLayer { get; set; } = 0;

    /// <summary>
    /// The source rectangle for texture sampling (normalized 0-1).
    /// Use Rectangle.Empty for full texture.
    /// </summary>
    public Rectangle SourceRect { get; set; } = new(0, 0, 1, 1);

    /// <summary>
    /// The frame rectangle for sprite sheet animation.
    /// </summary>
    public Rectangle Frame { get; set; } = Rectangle.Empty;

    /// <summary>
    /// Whether to show debug outline.
    /// </summary>
    public bool Debug { get; set; } = false;

    /// <summary>
    /// Creates a sprite configuration with the specified texture.
    /// </summary>
    public SpriteConfig(TextureId textureId)
    {
        TextureId = textureId;
    }

    /// <summary>
    /// Creates a sprite configuration with position and dimensions.
    /// </summary>
    public SpriteConfig(TextureId textureId, Vector2 position, Vector2 dimensions)
    {
        TextureId = textureId;
        Position = position;
        Dimensions = dimensions;
    }

    /// <summary>
    /// Sets the position and returns this config for chaining.
    /// </summary>
    public SpriteConfig At(float x, float y)
    {
        Position = new Vector2(x, y);
        return this;
    }

    /// <summary>
    /// Sets the position and returns this config for chaining.
    /// </summary>
    public SpriteConfig At(Vector2 position)
    {
        Position = position;
        return this;
    }

    /// <summary>
    /// Sets the dimensions and returns this config for chaining.
    /// </summary>
    public SpriteConfig WithSize(float width, float height)
    {
        Dimensions = new Vector2(width, height);
        return this;
    }

    /// <summary>
    /// Sets the scale and returns this config for chaining.
    /// </summary>
    public SpriteConfig WithScale(float scaleX, float scaleY)
    {
        Scale = new Vector2(scaleX, scaleY);
        return this;
    }

    /// <summary>
    /// Sets uniform scale and returns this config for chaining.
    /// </summary>
    public SpriteConfig WithScale(float scale)
    {
        Scale = new Vector2(scale, scale);
        return this;
    }

    /// <summary>
    /// Sets the rotation and returns this config for chaining.
    /// </summary>
    public SpriteConfig WithRotation(float degrees)
    {
        Rotation = degrees;
        return this;
    }

    /// <summary>
    /// Sets the Z-layer and returns this config for chaining.
    /// </summary>
    public SpriteConfig OnLayer(int zLayer)
    {
        ZLayer = zLayer;
        return this;
    }

    /// <summary>
    /// Enables debug rendering and returns this config for chaining.
    /// </summary>
    public SpriteConfig WithDebug(bool enabled = true)
    {
        Debug = enabled;
        return this;
    }

    /// <summary>
    /// Converts to the native FFI type.
    /// </summary>
    internal CsBindgen.SpriteCreateDto ToNative() => new()
    {
        x = Position.X,
        y = Position.Y,
        z_layer = ZLayer,
        scale_x = Scale.X,
        scale_y = Scale.Y,
        dimension_x = Dimensions.X,
        dimension_y = Dimensions.Y,
        rotation = Rotation,
        source_rect = SourceRect.ToNative(),
        texture_id = TextureId,
        debug = Debug,
        frame = Frame.ToNative()
    };
}
