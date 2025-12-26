using System;

namespace GoudEngine.Core;

/// <summary>
/// Type-safe wrapper for 3D object identifiers.
/// </summary>
public readonly struct ObjectId : IEquatable<ObjectId>
{
    public uint Value { get; }

    public ObjectId(uint value)
    {
        Value = value;
    }

    /// <summary>
    /// Returns true if this ID represents a valid 3D object.
    /// </summary>
    public bool IsValid => Value != uint.MaxValue;

    /// <summary>
    /// An invalid object ID.
    /// </summary>
    public static ObjectId Invalid => new(uint.MaxValue);

    // Implicit conversion to uint for backwards compatibility
    public static implicit operator uint(ObjectId id) => id.Value;

    // Explicit conversion from uint
    public static explicit operator ObjectId(uint value) => new(value);

    // Equality
    public bool Equals(ObjectId other) => Value == other.Value;
    public override bool Equals(object? obj) => obj is ObjectId other && Equals(other);
    public override int GetHashCode() => Value.GetHashCode();
    public override string ToString() => $"Object3D({Value})";

    public static bool operator ==(ObjectId a, ObjectId b) => a.Equals(b);
    public static bool operator !=(ObjectId a, ObjectId b) => !a.Equals(b);
}
