using System;

namespace GoudEngine.Core;

/// <summary>
/// Type-safe wrapper for light identifiers.
/// </summary>
public readonly struct LightId : IEquatable<LightId>
{
    public uint Value { get; }

    public LightId(uint value)
    {
        Value = value;
    }

    /// <summary>
    /// Returns true if this ID represents a valid light.
    /// </summary>
    public bool IsValid => Value != uint.MaxValue;

    /// <summary>
    /// An invalid light ID.
    /// </summary>
    public static LightId Invalid => new(uint.MaxValue);

    // Implicit conversion to uint for backwards compatibility
    public static implicit operator uint(LightId id) => id.Value;

    // Explicit conversion from uint
    public static explicit operator LightId(uint value) => new(value);

    // Equality
    public bool Equals(LightId other) => Value == other.Value;
    public override bool Equals(object? obj) => obj is LightId other && Equals(other);
    public override int GetHashCode() => Value.GetHashCode();
    public override string ToString() => $"Light({Value})";

    public static bool operator ==(LightId a, LightId b) => a.Equals(b);
    public static bool operator !=(LightId a, LightId b) => !a.Equals(b);
}
