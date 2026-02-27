using System;

namespace GoudEngine.Math;

/// <summary>
/// Represents a 2D vector with X and Y components.
/// </summary>
public readonly struct Vector2 : IEquatable<Vector2>
{
    public float X { get; }
    public float Y { get; }

    public Vector2(float x, float y)
    {
        X = x;
        Y = y;
    }

    public Vector2(float value) : this(value, value) { }

    // Common vectors
    public static Vector2 Zero => new(0, 0);
    public static Vector2 One => new(1, 1);
    public static Vector2 UnitX => new(1, 0);
    public static Vector2 UnitY => new(0, 1);

    // Properties
    public float Length => MathF.Sqrt(X * X + Y * Y);
    public float LengthSquared => X * X + Y * Y;

    // Methods
    public Vector2 Normalized()
    {
        float len = Length;
        if (len == 0) return Zero;
        return new Vector2(X / len, Y / len);
    }

    public float Dot(Vector2 other) => X * other.X + Y * other.Y;

    public static float Distance(Vector2 a, Vector2 b)
    {
        float dx = b.X - a.X;
        float dy = b.Y - a.Y;
        return MathF.Sqrt(dx * dx + dy * dy);
    }

    public static Vector2 Lerp(Vector2 a, Vector2 b, float t)
    {
        t = System.Math.Clamp(t, 0f, 1f);
        return new Vector2(
            a.X + (b.X - a.X) * t,
            a.Y + (b.Y - a.Y) * t
        );
    }

    // Operators
    public static Vector2 operator +(Vector2 a, Vector2 b) => new(a.X + b.X, a.Y + b.Y);
    public static Vector2 operator -(Vector2 a, Vector2 b) => new(a.X - b.X, a.Y - b.Y);
    public static Vector2 operator *(Vector2 v, float s) => new(v.X * s, v.Y * s);
    public static Vector2 operator *(float s, Vector2 v) => new(v.X * s, v.Y * s);
    public static Vector2 operator /(Vector2 v, float s) => new(v.X / s, v.Y / s);
    public static Vector2 operator -(Vector2 v) => new(-v.X, -v.Y);

    public static bool operator ==(Vector2 a, Vector2 b) => a.Equals(b);
    public static bool operator !=(Vector2 a, Vector2 b) => !a.Equals(b);

    // Equality
    public bool Equals(Vector2 other) => X == other.X && Y == other.Y;
    public override bool Equals(object? obj) => obj is Vector2 other && Equals(other);
    public override int GetHashCode() => HashCode.Combine(X, Y);
    public override string ToString() => $"({X}, {Y})";

    // Deconstruction
    public void Deconstruct(out float x, out float y)
    {
        x = X;
        y = Y;
    }
}
