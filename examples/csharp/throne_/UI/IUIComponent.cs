// UI/IUIComponent.cs
// Base interface for atomic UI components

namespace Throne.UI;

/// <summary>
/// Interface for atomic UI components.
/// All UI elements implement this for consistent lifecycle management.
/// </summary>
public interface IUIComponent
{
    /// <summary>
    /// X position of the component center.
    /// </summary>
    float X { get; set; }

    /// <summary>
    /// Y position of the component center.
    /// </summary>
    float Y { get; set; }

    /// <summary>
    /// Whether the component is visible.
    /// </summary>
    bool IsVisible { get; set; }

    /// <summary>
    /// Update component state (animations, input, etc.)
    /// </summary>
    void Update(float deltaTime);

    /// <summary>
    /// Draw the component.
    /// </summary>
    void Draw();
}
