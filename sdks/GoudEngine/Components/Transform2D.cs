using System;
using System.Runtime.InteropServices;
using GoudEngine.Math;

namespace GoudEngine.Components
{
    /// <summary>
    /// 2D spatial transformation component for position, rotation, and scale.
    /// Optimized for 2D games with simpler representation than 3D transforms.
    /// </summary>
    /// <remarks>
    /// This is a pure data struct. All transformation logic is implemented in
    /// the Rust engine and accessible via FFI functions (goud_transform2d_*).
    /// 
    /// Memory layout: 20 bytes total (must match Rust FfiTransform2D)
    /// - position: Vec2 (8 bytes)
    /// - rotation: f32 (4 bytes)
    /// - scale: Vec2 (8 bytes)
    /// </remarks>
    [StructLayout(LayoutKind.Sequential)]
    [Component(0x1234567890ABCDEF, 20)] // TODO: Compute actual TypeId hash
    public struct Transform2D : IComponent
    {
        /// <summary>
        /// Position in world space (or local space if entity has a parent).
        /// </summary>
        public float PositionX;

        /// <summary>
        /// Position Y in world space (or local space if entity has a parent).
        /// </summary>
        public float PositionY;

        /// <summary>
        /// Rotation angle in radians (counter-clockwise).
        /// </summary>
        public float Rotation;

        /// <summary>
        /// Scale along X axis. 1.0 means no scaling.
        /// Negative values flip the object along that axis.
        /// </summary>
        public float ScaleX;

        /// <summary>
        /// Scale along Y axis. 1.0 means no scaling.
        /// Negative values flip the object along that axis.
        /// </summary>
        public float ScaleY;

        /// <inheritdoc/>
        public ulong TypeId => 0x1234567890ABCDEF; // TODO: Compute actual hash

        /// <inheritdoc/>
        public int SizeInBytes => 20;

        /// <summary>
        /// Gets or sets the position as a Vector2.
        /// </summary>
        public Vector2 Position
        {
            readonly get => new(PositionX, PositionY);
            set
            {
                PositionX = value.X;
                PositionY = value.Y;
            }
        }

        /// <summary>
        /// Gets or sets the scale as a Vector2.
        /// </summary>
        public Vector2 Scale
        {
            readonly get => new(ScaleX, ScaleY);
            set
            {
                ScaleX = value.X;
                ScaleY = value.Y;
            }
        }

        /// <summary>
        /// Creates a new Transform2D with specified position, rotation, and scale.
        /// </summary>
        /// <param name="positionX">X position in 2D space.</param>
        /// <param name="positionY">Y position in 2D space.</param>
        /// <param name="rotation">Rotation angle in radians.</param>
        /// <param name="scaleX">Scale along X axis.</param>
        /// <param name="scaleY">Scale along Y axis.</param>
        public Transform2D(float positionX, float positionY, float rotation, float scaleX, float scaleY)
        {
            PositionX = positionX;
            PositionY = positionY;
            Rotation = rotation;
            ScaleX = scaleX;
            ScaleY = scaleY;
        }

        /// <summary>
        /// Creates a new Transform2D with Vector2 position and scale.
        /// </summary>
        /// <param name="position">Position in 2D space.</param>
        /// <param name="rotation">Rotation angle in radians.</param>
        /// <param name="scale">Scale along each axis.</param>
        public Transform2D(Vector2 position, float rotation, Vector2 scale)
            : this(position.X, position.Y, rotation, scale.X, scale.Y)
        {
        }

        /// <summary>
        /// Creates a Transform2D at the specified position with default rotation and scale.
        /// </summary>
        public static Transform2D FromPosition(float x, float y)
            => new(x, y, 0f, 1f, 1f);

        /// <summary>
        /// Creates a Transform2D at the specified position with default rotation and scale.
        /// </summary>
        public static Transform2D FromPosition(Vector2 position)
            => FromPosition(position.X, position.Y);

        /// <summary>
        /// Creates a Transform2D with the specified rotation (in radians).
        /// </summary>
        public static Transform2D FromRotation(float rotation)
            => new(0f, 0f, rotation, 1f, 1f);

        /// <summary>
        /// Creates a Transform2D with the specified rotation (in degrees).
        /// </summary>
        public static Transform2D FromRotationDegrees(float degrees)
            => FromRotation(degrees * MathF.PI / 180f);

        /// <summary>
        /// Creates a Transform2D with the specified scale.
        /// </summary>
        public static Transform2D FromScale(float scaleX, float scaleY)
            => new(0f, 0f, 0f, scaleX, scaleY);

        /// <summary>
        /// Creates a Transform2D with uniform scale.
        /// </summary>
        public static Transform2D FromScaleUniform(float scale)
            => FromScale(scale, scale);

        /// <summary>
        /// Creates a Transform2D at the specified position with rotation.
        /// </summary>
        public static Transform2D FromPositionRotation(Vector2 position, float rotation)
            => new(position.X, position.Y, rotation, 1f, 1f);

        /// <summary>
        /// Gets the default identity transform (zero position, zero rotation, unit scale).
        /// </summary>
        public static Transform2D Identity => new(0f, 0f, 0f, 1f, 1f);

        /// <summary>
        /// Gets the rotation angle in degrees.
        /// </summary>
        public readonly float RotationDegrees => Rotation * 180f / MathF.PI;

        /// <inheritdoc/>
        public override readonly string ToString()
            => $"Transform2D(pos: ({PositionX}, {PositionY}), rot: {RotationDegrees:F1}°, scale: ({ScaleX}, {ScaleY}))";

        /// <inheritdoc/>
        public override readonly bool Equals(object? obj)
            => obj is Transform2D other &&
               MathF.Abs(PositionX - other.PositionX) < 0.0001f &&
               MathF.Abs(PositionY - other.PositionY) < 0.0001f &&
               MathF.Abs(Rotation - other.Rotation) < 0.0001f &&
               MathF.Abs(ScaleX - other.ScaleX) < 0.0001f &&
               MathF.Abs(ScaleY - other.ScaleY) < 0.0001f;

        /// <inheritdoc/>
        public override readonly int GetHashCode()
            => HashCode.Combine(PositionX, PositionY, Rotation, ScaleX, ScaleY);

        /// <summary>
        /// Checks equality between two Transform2D instances.
        /// </summary>
        public static bool operator ==(Transform2D left, Transform2D right) => left.Equals(right);

        /// <summary>
        /// Checks inequality between two Transform2D instances.
        /// </summary>
        public static bool operator !=(Transform2D left, Transform2D right) => !left.Equals(right);
    }
}
