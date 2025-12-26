using System;

namespace GoudEngine.Math;

/// <summary>
/// Represents an RGBA color with components in the range [0, 1].
/// </summary>
public readonly struct Color : IEquatable<Color>
{
    public float R { get; }
    public float G { get; }
    public float B { get; }
    public float A { get; }

    public Color(float r, float g, float b, float a = 1.0f)
    {
        R = System.Math.Clamp(r, 0f, 1f);
        G = System.Math.Clamp(g, 0f, 1f);
        B = System.Math.Clamp(b, 0f, 1f);
        A = System.Math.Clamp(a, 0f, 1f);
    }

    /// <summary>
    /// Creates a color from byte values (0-255).
    /// </summary>
    public static Color FromBytes(byte r, byte g, byte b, byte a = 255)
        => new(r / 255f, g / 255f, b / 255f, a / 255f);

    /// <summary>
    /// Creates a color from a hex string (e.g., "#FF0000" or "FF0000").
    /// </summary>
    public static Color FromHex(string hex)
    {
        hex = hex.TrimStart('#');
        if (hex.Length == 6)
        {
            return FromBytes(
                Convert.ToByte(hex[0..2], 16),
                Convert.ToByte(hex[2..4], 16),
                Convert.ToByte(hex[4..6], 16)
            );
        }
        else if (hex.Length == 8)
        {
            return FromBytes(
                Convert.ToByte(hex[0..2], 16),
                Convert.ToByte(hex[2..4], 16),
                Convert.ToByte(hex[4..6], 16),
                Convert.ToByte(hex[6..8], 16)
            );
        }
        throw new ArgumentException("Invalid hex color format", nameof(hex));
    }

    // Common colors
    public static Color White => new(1, 1, 1);
    public static Color Black => new(0, 0, 0);
    public static Color Red => new(1, 0, 0);
    public static Color Green => new(0, 1, 0);
    public static Color Blue => new(0, 0, 1);
    public static Color Yellow => new(1, 1, 0);
    public static Color Cyan => new(0, 1, 1);
    public static Color Magenta => new(1, 0, 1);
    public static Color Orange => new(1, 0.5f, 0);
    public static Color Purple => new(0.5f, 0, 0.5f);
    public static Color Gray => new(0.5f, 0.5f, 0.5f);
    public static Color LightGray => new(0.75f, 0.75f, 0.75f);
    public static Color DarkGray => new(0.25f, 0.25f, 0.25f);
    public static Color Transparent => new(0, 0, 0, 0);

    // Conversion to array for FFI
    public float[] ToArray() => new[] { R, G, B };
    public float[] ToArrayWithAlpha() => new[] { R, G, B, A };

    // Convert to Vector3 (useful for passing to engine)
    public Vector3 ToVector3() => new(R, G, B);

    // Methods
    public Color WithAlpha(float alpha) => new(R, G, B, alpha);

    public static Color Lerp(Color a, Color b, float t)
    {
        t = System.Math.Clamp(t, 0f, 1f);
        return new Color(
            a.R + (b.R - a.R) * t,
            a.G + (b.G - a.G) * t,
            a.B + (b.B - a.B) * t,
            a.A + (b.A - a.A) * t
        );
    }

    // Operators
    public static Color operator *(Color c, float s) => new(c.R * s, c.G * s, c.B * s, c.A);
    public static Color operator *(float s, Color c) => new(c.R * s, c.G * s, c.B * s, c.A);

    public static bool operator ==(Color a, Color b) => a.Equals(b);
    public static bool operator !=(Color a, Color b) => !a.Equals(b);

    // Equality
    public bool Equals(Color other) => R == other.R && G == other.G && B == other.B && A == other.A;
    public override bool Equals(object? obj) => obj is Color other && Equals(other);
    public override int GetHashCode() => HashCode.Combine(R, G, B, A);
    public override string ToString() => $"RGBA({R:F2}, {G:F2}, {B:F2}, {A:F2})";

    // Deconstruction
    public void Deconstruct(out float r, out float g, out float b)
    {
        r = R;
        g = G;
        b = B;
    }

    public void Deconstruct(out float r, out float g, out float b, out float a)
    {
        r = R;
        g = G;
        b = B;
        a = A;
    }
}
