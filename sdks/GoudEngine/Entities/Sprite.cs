using GoudEngine.Core;
using GoudEngine.Math;

namespace GoudEngine.Entities;

/// <summary>
/// High-level wrapper for 2D sprites that provides property-based access
/// and automatically syncs changes to the engine.
/// </summary>
public class Sprite
{
    private readonly GoudGame _game;
    private Vector2 _position;
    private Vector2 _scale;
    private float _rotation;
    private int _zLayer;
    private Rectangle _sourceRect;
    private Rectangle _frame;
    private bool _debug;

    /// <summary>
    /// The unique identifier for this sprite.
    /// </summary>
    public SpriteId Id { get; }

    /// <summary>
    /// The texture used by this sprite.
    /// </summary>
    public TextureId TextureId { get; }

    /// <summary>
    /// The dimensions of this sprite.
    /// </summary>
    public Vector2 Dimensions { get; }

    /// <summary>
    /// Gets or sets the position of this sprite.
    /// </summary>
    public Vector2 Position
    {
        get => _position;
        set
        {
            _position = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the X position.
    /// </summary>
    public float X
    {
        get => _position.X;
        set => Position = new Vector2(value, _position.Y);
    }

    /// <summary>
    /// Gets or sets the Y position.
    /// </summary>
    public float Y
    {
        get => _position.Y;
        set => Position = new Vector2(_position.X, value);
    }

    /// <summary>
    /// Gets or sets the scale of this sprite.
    /// </summary>
    public Vector2 Scale
    {
        get => _scale;
        set
        {
            _scale = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the rotation in degrees.
    /// </summary>
    public float Rotation
    {
        get => _rotation;
        set
        {
            _rotation = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the Z-layer (render order).
    /// </summary>
    public int ZLayer
    {
        get => _zLayer;
        set
        {
            _zLayer = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the source rectangle for texture sampling (normalized 0-1).
    /// </summary>
    public Rectangle SourceRect
    {
        get => _sourceRect;
        set
        {
            _sourceRect = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets the frame rectangle for sprite sheet animation.
    /// </summary>
    public Rectangle Frame
    {
        get => _frame;
        set
        {
            _frame = value;
            SyncToEngine();
        }
    }

    /// <summary>
    /// Gets or sets whether debug rendering is enabled.
    /// </summary>
    public bool Debug
    {
        get => _debug;
        set
        {
            _debug = value;
            SyncToEngine();
        }
    }

    internal Sprite(
        GoudGame game,
        SpriteId id,
        TextureId textureId,
        Vector2 position,
        Vector2 dimensions,
        Vector2 scale,
        float rotation,
        int zLayer,
        Rectangle sourceRect,
        Rectangle frame,
        bool debug)
    {
        _game = game;
        Id = id;
        TextureId = textureId;
        Dimensions = dimensions;
        _position = position;
        _scale = scale;
        _rotation = rotation;
        _zLayer = zLayer;
        _sourceRect = sourceRect;
        _frame = frame;
        _debug = debug;
    }

    /// <summary>
    /// Checks if this sprite collides with another sprite.
    /// </summary>
    public bool CollidesWith(Sprite other)
    {
        return _game.CheckCollision(Id, other.Id);
    }

    /// <summary>
    /// Checks if this sprite collides with another sprite by ID.
    /// </summary>
    public bool CollidesWith(SpriteId otherId)
    {
        return _game.CheckCollision(Id, otherId);
    }

    /// <summary>
    /// Moves the sprite by the given delta.
    /// </summary>
    public void Move(Vector2 delta)
    {
        Position += delta;
    }

    /// <summary>
    /// Moves the sprite by the given delta.
    /// </summary>
    public void Move(float dx, float dy)
    {
        Position = new Vector2(_position.X + dx, _position.Y + dy);
    }

    /// <summary>
    /// Removes this sprite from the game.
    /// </summary>
    public void Destroy()
    {
        _game.RemoveSprite(Id);
    }

    private void SyncToEngine()
    {
        var updateDto = new CsBindgen.SpriteUpdateDto
        {
            id = Id,
            x = _position.X,
            y = _position.Y,
            z_layer = _zLayer,
            scale_x = _scale.X,
            scale_y = _scale.Y,
            dimension_x = Dimensions.X,
            dimension_y = Dimensions.Y,
            rotation = _rotation,
            source_rect = _sourceRect.ToNative(),
            texture_id = TextureId,
            debug = _debug,
            frame = _frame.ToNative()
        };
        _game.UpdateSprite(updateDto);
    }
}
