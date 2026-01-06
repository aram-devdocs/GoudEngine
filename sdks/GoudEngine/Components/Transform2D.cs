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
    /// Memory layout: 20 bytes total
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
        public Vector2 Position;

        /// <summary>
        /// Rotation angle in radians (counter-clockwise).
        /// </summary>
        public float Rotation;

        /// <summary>
        /// Scale along each axis. (1, 1) means no scaling.
        /// Negative values flip the object along that axis.
        /// </summary>
        public Vector2 Scale;

        /// <inheritdoc/>
        public ulong TypeId => 0x1234567890ABCDEF; // TODO: Compute actual hash

        /// <inheritdoc/>
        public int SizeInBytes => 20;

        /// <summary>
        /// Creates a new Transform2D with specified position, rotation, and scale.
        /// </summary>
        /// <param name="position">Position in 2D space.</param>
        /// <param name="rotation">Rotation angle in radians.</param>
        /// <param name="scale">Scale along each axis.</param>
        public Transform2D(Vector2 position, float rotation, Vector2 scale)
        {
            Position = position;
            Rotation = rotation;
            Scale = scale;
        }

        /// <summary>
        /// Creates a Transform2D at the specified position with default rotation and scale.
        /// </summary>
        /// <param name="position">Position in 2D space.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromPosition(Vector2 position)
        {
            return new Transform2D(position, 0f, Vector2.One);
        }

        /// <summary>
        /// Creates a Transform2D with the specified rotation (in radians).
        /// </summary>
        /// <param name="rotation">Rotation angle in radians.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromRotation(float rotation)
        {
            return new Transform2D(Vector2.Zero, rotation, Vector2.One);
        }

        /// <summary>
        /// Creates a Transform2D with the specified rotation (in degrees).
        /// </summary>
        /// <param name="degrees">Rotation angle in degrees.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromRotationDegrees(float degrees)
        {
            return FromRotation(degrees * (float)Math.PI / 180f);
        }

        /// <summary>
        /// Creates a Transform2D with the specified scale.
        /// </summary>
        /// <param name="scale">Scale along each axis.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromScale(Vector2 scale)
        {
            return new Transform2D(Vector2.Zero, 0f, scale);
        }

        /// <summary>
        /// Creates a Transform2D with uniform scale.
        /// </summary>
        /// <param name="scale">Uniform scale factor.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromScaleUniform(float scale)
        {
            return FromScale(new Vector2(scale, scale));
        }

        /// <summary>
        /// Creates a Transform2D at the specified position with rotation.
        /// </summary>
        /// <param name="position">Position in 2D space.</param>
        /// <param name="rotation">Rotation angle in radians.</param>
        /// <returns>A new Transform2D.</returns>
        public static Transform2D FromPositionRotation(Vector2 position, float rotation)
        {
            return new Transform2D(position, rotation, Vector2.One);
        }

        /// <summary>
        /// Gets the default identity transform (zero position, zero rotation, unit scale).
        /// </summary>
        public static Transform2D Identity => new Transform2D(Vector2.Zero, 0f, Vector2.One);

        /// <summary>
        /// Translates (moves) the transform by the specified offset.
        /// </summary>
        /// <param name="offset">The translation offset.</param>
        public void Translate(Vector2 offset)
        {
            Position += offset;
        }

        /// <summary>
        /// Rotates the transform by the specified angle (in radians).
        /// </summary>
        /// <param name="radians">The rotation angle to add.</param>
        public void Rotate(float radians)
        {
            Rotation += radians;
            NormalizeRotation();
        }

        /// <summary>
        /// Rotates the transform by the specified angle (in degrees).
        /// </summary>
        /// <param name="degrees">The rotation angle to add in degrees.</param>
        public void RotateDegrees(float degrees)
        {
            Rotate(degrees * (float)System.Math.PI / 180f);
        }

        /// <summary>
        /// Gets the rotation angle in degrees.
        /// </summary>
        public float RotationDegrees => Rotation * 180f / (float)System.Math.PI;

        /// <summary>
        /// Sets the rotation angle in degrees.
        /// </summary>
        /// <param name="degrees">The rotation angle in degrees.</param>
        public void SetRotationDegrees(float degrees)
        {
            Rotation = degrees * (float)System.Math.PI / 180f;
            NormalizeRotation();
        }

        /// <summary>
        /// Gets the forward direction vector (based on rotation).
        /// Points in the direction of rotation = 0 (right), then rotated.
        /// </summary>
        public Vector2 Forward()
        {
            return new Vector2((float)System.Math.Cos(Rotation), (float)System.Math.Sin(Rotation));
        }

        /// <summary>
        /// Gets the right direction vector (perpendicular to forward).
        /// </summary>
        public Vector2 Right()
        {
            return new Vector2((float)System.Math.Cos(Rotation + System.Math.PI / 2), (float)System.Math.Sin(Rotation + System.Math.PI / 2));
        }

        /// <summary>
        /// Normalizes the rotation angle to the range [-PI, PI].
        /// </summary>
        private void NormalizeRotation()
        {
            const float TwoPI = 2f * (float)System.Math.PI;
            Rotation = ((Rotation + (float)System.Math.PI) % TwoPI + TwoPI) % TwoPI - (float)System.Math.PI;
        }

        /// <inheritdoc/>
        public override string ToString()
        {
            return $"Transform2D(pos: {Position}, rot: {RotationDegrees:F1}°, scale: {Scale})";
        }

        /// <inheritdoc/>
        public override bool Equals(object? obj)
        {
            return obj is Transform2D other &&
                   Position.Equals(other.Position) &&
                   System.Math.Abs(Rotation - other.Rotation) < 0.0001f &&
                   Scale.Equals(other.Scale);
        }

        /// <inheritdoc/>
        public override int GetHashCode()
        {
            return HashCode.Combine(Position, Rotation, Scale);
        }

        /// <summary>
        /// Checks equality between two Transform2D instances.
        /// </summary>
        public static bool operator ==(Transform2D left, Transform2D right)
        {
            return left.Equals(right);
        }

        /// <summary>
        /// Checks inequality between two Transform2D instances.
        /// </summary>
        public static bool operator !=(Transform2D left, Transform2D right)
        {
            return !(left == right);
        }
    }
}
