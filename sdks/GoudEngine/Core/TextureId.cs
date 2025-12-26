using System;

namespace GoudEngine.Core;

/// <summary>
/// Type-safe wrapper for texture identifiers.
/// </summary>
public readonly struct TextureId : IEquatable<TextureId>
{
    public uint Value { get; }

    public TextureId(uint value)
    {
        Value = value;
    }

    /// <summary>
    /// Returns true if this ID represents a valid texture.
    /// </summary>
    public bool IsValid => Value != uint.MaxValue;

    /// <summary>
    /// An invalid texture ID.
    /// </summary>
    public static TextureId Invalid => new(uint.MaxValue);

    // Implicit conversion to uint for backwards compatibility
    public static implicit operator uint(TextureId id) => id.Value;

    // Explicit conversion from uint
    public static explicit operator TextureId(uint value) => new(value);

    // Equality
    public bool Equals(TextureId other) => Value == other.Value;
    public override bool Equals(object? obj) => obj is TextureId other && Equals(other);
    public override int GetHashCode() => Value.GetHashCode();
    public override string ToString() => $"Texture({Value})";

    public static bool operator ==(TextureId a, TextureId b) => a.Equals(b);
    public static bool operator !=(TextureId a, TextureId b) => !a.Equals(b);
}
