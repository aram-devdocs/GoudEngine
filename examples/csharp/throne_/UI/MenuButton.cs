// UI/MenuButton.cs
// Menu button component with locked/unlocked state

using GoudEngine.Math;

namespace Throne.UI;

/// <summary>
/// A menu button that can be locked (grayed out) or unlocked.
/// </summary>
public class MenuButton : IUIComponent
{
    private readonly GoudGame _game;
    private readonly ulong _texture;
    private readonly ulong _lockTexture;

    public float X { get; set; }
    public float Y { get; set; }
    public float Width { get; set; }
    public float Height { get; set; }
    public bool IsVisible { get; set; } = true;

    /// <summary>
    /// Whether the button is locked (non-interactable).
    /// </summary>
    public bool IsLocked { get; set; }

    /// <summary>
    /// Label for the button (for debugging/identification).
    /// </summary>
    public string Label { get; set; }

    /// <summary>
    /// Color tint when button is locked.
    /// </summary>
    public Color LockedColor { get; set; } = new Color(0.4f, 0.4f, 0.4f, 0.8f);

    /// <summary>
    /// Color tint when button is unlocked.
    /// </summary>
    public Color UnlockedColor { get; set; } = Color.White;

    public MenuButton(GoudGame game, ulong texture, ulong lockTexture, 
                      float x, float y, float width, float height, 
                      string label, bool isLocked = false)
    {
        _game = game;
        _texture = texture;
        _lockTexture = lockTexture;
        X = x;
        Y = y;
        Width = width;
        Height = height;
        Label = label;
        IsLocked = isLocked;
    }

    public void Update(float deltaTime)
    {
        // Future: Add hover/click detection when unlocked
    }

    public void Draw()
    {
        if (!IsVisible) return;

        // Draw button with appropriate color
        Color tint = IsLocked ? LockedColor : UnlockedColor;
        _game.DrawSprite(_texture, X, Y, Width, Height, 0f, tint);

        // Draw lock icon overlay if locked
        if (IsLocked && _lockTexture != 0)
        {
            float lockSize = Math.Min(Width, Height) * 0.4f;
            _game.DrawSprite(_lockTexture, X + Width * 0.35f, Y, lockSize, lockSize);
        }
    }

    /// <summary>
    /// Check if a point is within the button bounds.
    /// </summary>
    public bool ContainsPoint(float px, float py)
    {
        float halfW = Width / 2f;
        float halfH = Height / 2f;
        return px >= X - halfW && px <= X + halfW &&
               py >= Y - halfH && py <= Y + halfH;
    }
}
