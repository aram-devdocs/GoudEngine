using System;

namespace GoudEngine.Core;

/// <summary>
/// Type-safe wrapper for tiled map identifiers.
/// </summary>
public readonly struct TiledMapId : IEquatable<TiledMapId>
{
    public uint Value { get; }

    public TiledMapId(uint value)
    {
        Value = value;
    }

    /// <summary>
    /// Returns true if this ID represents a valid tiled map.
    /// </summary>
    public bool IsValid => Value != uint.MaxValue;

    /// <summary>
    /// An invalid tiled map ID.
    /// </summary>
    public static TiledMapId Invalid => new(uint.MaxValue);

    // Implicit conversion to uint for backwards compatibility
    public static implicit operator uint(TiledMapId id) => id.Value;

    // Explicit conversion from uint
    public static explicit operator TiledMapId(uint value) => new(value);

    // Equality
    public bool Equals(TiledMapId other) => Value == other.Value;
    public override bool Equals(object? obj) => obj is TiledMapId other && Equals(other);
    public override int GetHashCode() => Value.GetHashCode();
    public override string ToString() => $"TiledMap({Value})";

    public static bool operator ==(TiledMapId a, TiledMapId b) => a.Equals(b);
    public static bool operator !=(TiledMapId a, TiledMapId b) => !a.Equals(b);
}
