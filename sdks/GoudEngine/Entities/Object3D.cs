using GoudEngine.Core;
using GoudEngine.Math;

namespace GoudEngine.Entities;

/// <summary>
/// High-level wrapper for 3D objects that provides property-based access
/// and automatically syncs changes to the engine.
/// </summary>
public class Object3D
{
    private readonly GoudGame _game;
    private Vector3 _position;
    private Vector3 _rotation;
    private Vector3 _scale;

    /// <summary>
    /// The unique identifier for this 3D object.
    /// </summary>
    public ObjectId Id { get; }

    /// <summary>
    /// The type of primitive this object represents.
    /// </summary>
    public PrimitiveType Type { get; }

    /// <summary>
    /// The texture used by this object.
    /// </summary>
    public TextureId TextureId { get; }

    /// <summary>
    /// Gets or sets the position of this object.
    /// </summary>
    public Vector3 Position
    {
        get => _position;
        set
        {
            _position = value;
            _game.SetObjectPosition(Id, value.X, value.Y, value.Z);
        }
    }

    /// <summary>
    /// Gets or sets the rotation of this object (in degrees).
    /// </summary>
    public Vector3 Rotation
    {
        get => _rotation;
        set
        {
            _rotation = value;
            _game.SetObjectRotation(Id, value.X, value.Y, value.Z);
        }
    }

    /// <summary>
    /// Gets or sets the scale of this object.
    /// </summary>
    public Vector3 Scale
    {
        get => _scale;
        set
        {
            _scale = value;
            _game.SetObjectScale(Id, value.X, value.Y, value.Z);
        }
    }

    internal Object3D(
        GoudGame game,
        ObjectId id,
        PrimitiveType type,
        TextureId textureId,
        Vector3 position,
        Vector3 rotation,
        Vector3 scale)
    {
        _game = game;
        Id = id;
        Type = type;
        TextureId = textureId;
        _position = position;
        _rotation = rotation;
        _scale = scale;
    }

    /// <summary>
    /// Moves the object by the given delta.
    /// </summary>
    public void Move(Vector3 delta)
    {
        Position += delta;
    }

    /// <summary>
    /// Rotates the object by the given delta (in degrees).
    /// </summary>
    public void Rotate(Vector3 delta)
    {
        Rotation += delta;
    }
}
