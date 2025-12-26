using System;

namespace GoudEngine.Math;

/// <summary>
/// Represents a rectangle with position and dimensions.
/// </summary>
public readonly struct Rectangle : IEquatable<Rectangle>
{
    public float X { get; }
    public float Y { get; }
    public float Width { get; }
    public float Height { get; }

    public Rectangle(float x, float y, float width, float height)
    {
        X = x;
        Y = y;
        Width = width;
        Height = height;
    }

    public Rectangle(Vector2 position, Vector2 size)
        : this(position.X, position.Y, size.X, size.Y) { }

    // Properties
    public float Left => X;
    public float Right => X + Width;
    public float Top => Y;
    public float Bottom => Y + Height;

    public Vector2 Position => new(X, Y);
    public Vector2 Size => new(Width, Height);
    public Vector2 Center => new(X + Width / 2, Y + Height / 2);

    public float Area => Width * Height;

    public static Rectangle Empty => new(0, 0, 0, 0);

    // Methods
    public bool Contains(float x, float y)
        => x >= Left && x < Right && y >= Top && y < Bottom;

    public bool Contains(Vector2 point)
        => Contains(point.X, point.Y);

    public bool Intersects(Rectangle other)
        => Left < other.Right && Right > other.Left &&
           Top < other.Bottom && Bottom > other.Top;

    public Rectangle Offset(float dx, float dy)
        => new(X + dx, Y + dy, Width, Height);

    public Rectangle Offset(Vector2 delta)
        => Offset(delta.X, delta.Y);

    public Rectangle Inflate(float horizontalAmount, float verticalAmount)
        => new(
            X - horizontalAmount,
            Y - verticalAmount,
            Width + horizontalAmount * 2,
            Height + verticalAmount * 2
        );

    public static Rectangle? Intersection(Rectangle a, Rectangle b)
    {
        float left = System.Math.Max(a.Left, b.Left);
        float top = System.Math.Max(a.Top, b.Top);
        float right = System.Math.Min(a.Right, b.Right);
        float bottom = System.Math.Min(a.Bottom, b.Bottom);

        if (right > left && bottom > top)
            return new Rectangle(left, top, right - left, bottom - top);

        return null;
    }

    public static Rectangle Union(Rectangle a, Rectangle b)
    {
        float left = System.Math.Min(a.Left, b.Left);
        float top = System.Math.Min(a.Top, b.Top);
        float right = System.Math.Max(a.Right, b.Right);
        float bottom = System.Math.Max(a.Bottom, b.Bottom);
        return new Rectangle(left, top, right - left, bottom - top);
    }

    // Conversion to FFI type
    internal CsBindgen.Rectangle ToNative() => new()
    {
        x = X,
        y = Y,
        width = Width,
        height = Height
    };

    // Equality
    public bool Equals(Rectangle other)
        => X == other.X && Y == other.Y && Width == other.Width && Height == other.Height;

    public override bool Equals(object? obj) => obj is Rectangle other && Equals(other);
    public override int GetHashCode() => HashCode.Combine(X, Y, Width, Height);
    public override string ToString() => $"Rectangle({X}, {Y}, {Width}, {Height})";

    public static bool operator ==(Rectangle a, Rectangle b) => a.Equals(b);
    public static bool operator !=(Rectangle a, Rectangle b) => !a.Equals(b);
}
