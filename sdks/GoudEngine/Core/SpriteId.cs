using System;

namespace GoudEngine.Core;

/// <summary>
/// Type-safe wrapper for sprite identifiers.
/// </summary>
public readonly struct SpriteId : IEquatable<SpriteId>
{
    public uint Value { get; }

    public SpriteId(uint value)
    {
        Value = value;
    }

    /// <summary>
    /// Returns true if this ID represents a valid sprite.
    /// Note: 0 is a valid ID in the engine, so we use uint.MaxValue as invalid.
    /// </summary>
    public bool IsValid => Value != uint.MaxValue;

    /// <summary>
    /// An invalid sprite ID.
    /// </summary>
    public static SpriteId Invalid => new(uint.MaxValue);

    // Implicit conversion to uint for backwards compatibility
    public static implicit operator uint(SpriteId id) => id.Value;

    // Explicit conversion from uint (require explicit cast for safety)
    public static explicit operator SpriteId(uint value) => new(value);

    // Equality
    public bool Equals(SpriteId other) => Value == other.Value;
    public override bool Equals(object? obj) => obj is SpriteId other && Equals(other);
    public override int GetHashCode() => Value.GetHashCode();
    public override string ToString() => $"Sprite({Value})";

    public static bool operator ==(SpriteId a, SpriteId b) => a.Equals(b);
    public static bool operator !=(SpriteId a, SpriteId b) => !a.Equals(b);
}
