// UI/TitleText.cs
// Title text component displaying "throne_"

using GoudEngine.Math;

namespace Throne.UI;

/// <summary>
/// Displays the game title with optional animation effects.
/// </summary>
public class TitleText : IUIComponent
{
    private readonly GoudGame _game;
    private readonly ulong _texture;
    private float _animTime;

    public float X { get; set; }
    public float Y { get; set; }
    public float Width { get; set; }
    public float Height { get; set; }
    public bool IsVisible { get; set; } = true;

    /// <summary>
    /// Enable subtle floating animation.
    /// </summary>
    public bool AnimateFloat { get; set; } = true;

    /// <summary>
    /// Floating animation amplitude in pixels.
    /// </summary>
    public float FloatAmplitude { get; set; } = 5f;

    /// <summary>
    /// Floating animation speed.
    /// </summary>
    public float FloatSpeed { get; set; } = 2f;

    public TitleText(GoudGame game, ulong texture, float x, float y, float width, float height)
    {
        _game = game;
        _texture = texture;
        X = x;
        Y = y;
        Width = width;
        Height = height;
    }

    public void Update(float deltaTime)
    {
        if (AnimateFloat)
        {
            _animTime += deltaTime;
        }
    }

    public void Draw()
    {
        if (!IsVisible) return;

        float yOffset = AnimateFloat 
            ? (float)Math.Sin(_animTime * FloatSpeed) * FloatAmplitude 
            : 0f;

        _game.DrawSprite(_texture, X, Y + yOffset, Width, Height);
    }
}
