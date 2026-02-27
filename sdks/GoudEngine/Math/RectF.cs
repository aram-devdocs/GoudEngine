namespace GoudEngine.Math;

/// <summary>
/// A floating-point rectangle, used for source rectangles in sprite sheets.
/// Values are in normalized UV coordinates (0.0 to 1.0) when used for texture sampling.
/// </summary>
public struct RectF
{
    /// <summary>
    /// X position (left edge) in normalized coordinates.
    /// </summary>
    public float X { get; set; }
    
    /// <summary>
    /// Y position (top edge) in normalized coordinates.
    /// </summary>
    public float Y { get; set; }
    
    /// <summary>
    /// Width in normalized coordinates.
    /// </summary>
    public float Width { get; set; }
    
    /// <summary>
    /// Height in normalized coordinates.
    /// </summary>
    public float Height { get; set; }

    /// <summary>
    /// Creates a new rectangle with the specified values.
    /// </summary>
    public RectF(float x, float y, float width, float height)
    {
        X = x;
        Y = y;
        Width = width;
        Height = height;
    }

    /// <summary>
    /// Creates a source rectangle from pixel coordinates given the texture size.
    /// Useful for sprite sheets where you specify frame positions in pixels.
    /// </summary>
    /// <param name="pixelX">X position in pixels</param>
    /// <param name="pixelY">Y position in pixels</param>
    /// <param name="pixelWidth">Width in pixels</param>
    /// <param name="pixelHeight">Height in pixels</param>
    /// <param name="textureWidth">Total texture width in pixels</param>
    /// <param name="textureHeight">Total texture height in pixels</param>
    public static RectF FromPixels(int pixelX, int pixelY, int pixelWidth, int pixelHeight,
                                   int textureWidth, int textureHeight)
    {
        return new RectF(
            (float)pixelX / textureWidth,
            (float)pixelY / textureHeight,
            (float)pixelWidth / textureWidth,
            (float)pixelHeight / textureHeight
        );
    }

    /// <summary>
    /// Creates a source rectangle for a frame in a grid-based sprite sheet.
    /// </summary>
    /// <param name="frameX">Column index (0-based)</param>
    /// <param name="frameY">Row index (0-based)</param>
    /// <param name="framesPerRow">Number of frames per row</param>
    /// <param name="framesPerColumn">Number of frames per column</param>
    public static RectF FromGrid(int frameX, int frameY, int framesPerRow, int framesPerColumn)
    {
        float frameWidth = 1.0f / framesPerRow;
        float frameHeight = 1.0f / framesPerColumn;
        return new RectF(
            frameX * frameWidth,
            frameY * frameHeight,
            frameWidth,
            frameHeight
        );
    }

    /// <summary>
    /// A rectangle covering the entire texture (0,0 to 1,1).
    /// </summary>
    public static RectF Full => new RectF(0, 0, 1, 1);

    public override string ToString() => $"RectF(X={X}, Y={Y}, W={Width}, H={Height})";
}
