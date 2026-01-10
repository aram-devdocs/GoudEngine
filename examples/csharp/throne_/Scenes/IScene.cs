// Scenes/IScene.cs
// Base interface for game scenes

namespace Throne.Scenes;

/// <summary>
/// Interface for game scenes (screens/levels).
/// Scenes manage a collection of entities and handle state transitions.
/// </summary>
public interface IScene
{
    /// <summary>
    /// Scene name for identification.
    /// </summary>
    string Name { get; }

    /// <summary>
    /// Called when the scene becomes active.
    /// </summary>
    void Enter();

    /// <summary>
    /// Called when the scene becomes inactive.
    /// </summary>
    void Exit();

    /// <summary>
    /// Update scene logic.
    /// </summary>
    void Update(float deltaTime);

    /// <summary>
    /// Draw scene content.
    /// </summary>
    void Draw();
}
