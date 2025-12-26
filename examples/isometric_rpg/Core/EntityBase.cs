using GoudEngine;
using CsBindgen;

namespace IsometricRpg.Core;

/// <summary>
/// Base class for all game entities (player, enemies, NPCs, projectiles).
/// Provides common functionality for position, sprite, and lifecycle management.
/// </summary>
public abstract class EntityBase
{
    protected readonly GoudGame Game;

    // Sprite ID returned by engine
    public uint SpriteId { get; protected set; }

    // World position
    public float X { get; set; }
    public float Y { get; set; }

    // Sprite dimensions (for collision)
    public float Width { get; protected set; } = 32f;
    public float Height { get; protected set; } = 32f;

    // Rendering
    public int ZLayer { get; protected set; } = 1;
    public float ScaleX { get; set; } = 1f;
    public float ScaleY { get; set; } = 1f;
    public float Rotation { get; set; } = 0f;

    // Current facing direction (0-7)
    public int Direction { get; set; } = 3; // Default: facing down/SE

    // Is this entity active/alive?
    public bool IsActive { get; set; } = true;

    // Texture ID for the sprite
    protected uint TextureId;

    protected EntityBase(GoudGame game)
    {
        Game = game;
    }

    /// <summary>
    /// Initialize the entity. Called once when entity is created.
    /// Override to set up textures, animations, etc.
    /// </summary>
    public abstract void Initialize();

    /// <summary>
    /// Update the entity each frame.
    /// </summary>
    public abstract void Update(float deltaTime);

    /// <summary>
    /// Create the sprite in the engine.
    /// </summary>
    protected void CreateSprite(uint textureId, float x, float y, int zLayer = 1)
    {
        TextureId = textureId;
        X = x;
        Y = y;
        ZLayer = zLayer;

        SpriteId = Game.AddSprite(new SpriteCreateDto
        {
            texture_id = textureId,
            x = x,
            y = y,
            z_layer = zLayer,
            scale_x = ScaleX,
            scale_y = ScaleY
        });
    }

    /// <summary>
    /// Update sprite position and properties in the engine.
    /// </summary>
    protected void UpdateSprite()
    {
        // Calculate z-layer based on Y position for depth sorting
        ZLayer = IsometricUtils.CalculateZLayer(Y);

        Game.UpdateSprite(new SpriteUpdateDto
        {
            id = SpriteId,
            x = X,
            y = Y,
            z_layer = ZLayer,
            scale_x = ScaleX,
            scale_y = ScaleY,
            rotation = Rotation
        });
    }

    /// <summary>
    /// Update sprite texture (for animation frame changes).
    /// </summary>
    protected void UpdateSpriteTexture(uint newTextureId)
    {
        TextureId = newTextureId;
        Game.UpdateSprite(new SpriteUpdateDto
        {
            id = SpriteId,
            texture_id = newTextureId
        });
    }

    /// <summary>
    /// Remove the entity's sprite from the engine.
    /// </summary>
    public virtual void Destroy()
    {
        IsActive = false;
        if (SpriteId != 0)
        {
            Game.RemoveSprite(SpriteId);
            SpriteId = 0;
        }
    }

    /// <summary>
    /// Check collision with another entity.
    /// </summary>
    public bool CollidesWith(EntityBase other)
    {
        if (!IsActive || !other.IsActive) return false;
        return Game.CheckCollision(SpriteId, other.SpriteId);
    }

    /// <summary>
    /// Get distance to another entity.
    /// </summary>
    public float DistanceTo(EntityBase other)
    {
        return IsometricUtils.Distance(X, Y, other.X, other.Y);
    }

    /// <summary>
    /// Get center position of entity.
    /// </summary>
    public (float x, float y) GetCenter()
    {
        return (X + Width / 2, Y + Height / 2);
    }
}
