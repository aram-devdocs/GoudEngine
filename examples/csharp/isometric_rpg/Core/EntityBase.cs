// Core/EntityBase.cs
// Simplified entity base class for immediate-mode rendering

namespace IsometricRpg.Core;

/// <summary>
/// Base class for simple entities in the game.
/// Uses immediate-mode rendering - no sprite IDs needed.
/// </summary>
public abstract class SimpleEntity
{
    public float X { get; set; }
    public float Y { get; set; }
    public float Width { get; protected set; } = 32f;
    public float Height { get; protected set; } = 32f;
    public bool IsActive { get; set; } = true;

    protected SimpleEntity(float x, float y)
    {
        X = x;
        Y = y;
    }

    public abstract void Update(float deltaTime);

    public bool CollidesWith(SimpleEntity other)
    {
        if (!IsActive || !other.IsActive) return false;
        
        return X < other.X + other.Width &&
               X + Width > other.X &&
               Y < other.Y + other.Height &&
               Y + Height > other.Y;
    }

    public float DistanceTo(SimpleEntity other)
    {
        float dx = X - other.X;
        float dy = Y - other.Y;
        return (float)System.Math.Sqrt(dx * dx + dy * dy);
    }
}
