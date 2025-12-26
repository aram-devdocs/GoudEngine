using GoudEngine.Core;
using GoudEngine.Math;

namespace GoudEngine.Entities;

/// <summary>
/// High-level wrapper for lights that provides property-based access
/// and automatically syncs changes to the engine.
/// </summary>
public class Light
{
    private readonly GoudGame _game;
    private LightType _type;
    private Vector3 _position;
    private Vector3 _direction;
    private Color _color;
    private float _intensity;
    private float _temperature;
    private float _range;
    private float _spotAngle;

    /// <summary>
    /// The unique identifier for this light.
    /// </summary>
    public LightId Id { get; }

    /// <summary>
    /// Gets or sets the type of light.
    /// </summary>
    public LightType Type
    {
        get => _type;
        set
        {
            _type = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the position of this light.
    /// </summary>
    public Vector3 Position
    {
        get => _position;
        set
        {
            _position = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the direction of this light (for directional and spot lights).
    /// </summary>
    public Vector3 Direction
    {
        get => _direction;
        set
        {
            _direction = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the color of this light.
    /// </summary>
    public Color Color
    {
        get => _color;
        set
        {
            _color = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the intensity of this light.
    /// </summary>
    public float Intensity
    {
        get => _intensity;
        set
        {
            _intensity = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the color temperature in Kelvin.
    /// </summary>
    public float Temperature
    {
        get => _temperature;
        set
        {
            _temperature = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the range of this light (for point and spot lights).
    /// </summary>
    public float Range
    {
        get => _range;
        set
        {
            _range = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the spot angle in degrees (for spot lights).
    /// </summary>
    public float SpotAngle
    {
        get => _spotAngle;
        set
        {
            _spotAngle = value;
            SyncToEngine();
        }
    }

    internal Light(
        GoudGame game,
        LightId id,
        LightType type,
        Vector3 position,
        Vector3 direction,
        Color color,
        float intensity,
        float temperature,
        float range,
        float spotAngle)
    {
        _game = game;
        Id = id;
        _type = type;
        _position = position;
        _direction = direction;
        _color = color;
        _intensity = intensity;
        _temperature = temperature;
        _range = range;
        _spotAngle = spotAngle;
    }

    /// <summary>
    /// Removes this light from the scene.
    /// </summary>
    public void Destroy()
    {
        _game.RemoveLight(Id);
    }

    private void SyncToEngine()
    {
        _game.UpdateLight(
            Id,
            _type,
            _position.X, _position.Y, _position.Z,
            _direction.X, _direction.Y, _direction.Z,
            _color.R, _color.G, _color.B,
            _intensity,
            _temperature,
            _range,
            _spotAngle
        );
    }
}
