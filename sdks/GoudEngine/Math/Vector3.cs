using System;

namespace GoudEngine.Math;

/// <summary>
/// Represents a 3D vector with X, Y, and Z components.
/// </summary>
public readonly struct Vector3 : IEquatable<Vector3>
{
    public float X { get; }
    public float Y { get; }
    public float Z { get; }

    public Vector3(float x, float y, float z)
    {
        X = x;
        Y = y;
        Z = z;
    }

    public Vector3(float value) : this(value, value, value) { }

    public Vector3(Vector2 xy, float z) : this(xy.X, xy.Y, z) { }

    // Common vectors
    public static Vector3 Zero => new(0, 0, 0);
    public static Vector3 One => new(1, 1, 1);
    public static Vector3 UnitX => new(1, 0, 0);
    public static Vector3 UnitY => new(0, 1, 0);
    public static Vector3 UnitZ => new(0, 0, 1);
    public static Vector3 Up => new(0, 1, 0);
    public static Vector3 Down => new(0, -1, 0);
    public static Vector3 Left => new(-1, 0, 0);
    public static Vector3 Right => new(1, 0, 0);
    public static Vector3 Forward => new(0, 0, -1);
    public static Vector3 Backward => new(0, 0, 1);

    // Properties
    public float Length => MathF.Sqrt(X * X + Y * Y + Z * Z);
    public float LengthSquared => X * X + Y * Y + Z * Z;
    public Vector2 XY => new(X, Y);

    // Methods
    public Vector3 Normalized()
    {
        float len = Length;
        if (len == 0) return Zero;
        return new Vector3(X / len, Y / len, Z / len);
    }

    public float Dot(Vector3 other) => X * other.X + Y * other.Y + Z * other.Z;

    public Vector3 Cross(Vector3 other) => new(
        Y * other.Z - Z * other.Y,
        Z * other.X - X * other.Z,
        X * other.Y - Y * other.X
    );

    public static float Distance(Vector3 a, Vector3 b)
    {
        float dx = b.X - a.X;
        float dy = b.Y - a.Y;
        float dz = b.Z - a.Z;
        return MathF.Sqrt(dx * dx + dy * dy + dz * dz);
    }

    public static Vector3 Lerp(Vector3 a, Vector3 b, float t)
    {
        t = System.Math.Clamp(t, 0f, 1f);
        return new Vector3(
            a.X + (b.X - a.X) * t,
            a.Y + (b.Y - a.Y) * t,
            a.Z + (b.Z - a.Z) * t
        );
    }

    // Operators
    public static Vector3 operator +(Vector3 a, Vector3 b) => new(a.X + b.X, a.Y + b.Y, a.Z + b.Z);
    public static Vector3 operator -(Vector3 a, Vector3 b) => new(a.X - b.X, a.Y - b.Y, a.Z - b.Z);
    public static Vector3 operator *(Vector3 v, float s) => new(v.X * s, v.Y * s, v.Z * s);
    public static Vector3 operator *(float s, Vector3 v) => new(v.X * s, v.Y * s, v.Z * s);
    public static Vector3 operator /(Vector3 v, float s) => new(v.X / s, v.Y / s, v.Z / s);
    public static Vector3 operator -(Vector3 v) => new(-v.X, -v.Y, -v.Z);

    public static bool operator ==(Vector3 a, Vector3 b) => a.Equals(b);
    public static bool operator !=(Vector3 a, Vector3 b) => !a.Equals(b);

    // Equality
    public bool Equals(Vector3 other) => X == other.X && Y == other.Y && Z == other.Z;
    public override bool Equals(object? obj) => obj is Vector3 other && Equals(other);
    public override int GetHashCode() => HashCode.Combine(X, Y, Z);
    public override string ToString() => $"({X}, {Y}, {Z})";

    // Deconstruction
    public void Deconstruct(out float x, out float y, out float z)
    {
        x = X;
        y = Y;
        z = Z;
    }
}
