using GoudEngine;
using CsBindgen;
using IsometricRpg.Core;

namespace IsometricRpg.Combat;

/// <summary>
/// A projectile entity for ranged attacks.
/// </summary>
public class Projectile : EntityBase
{
    // Projectile properties
    public int Damage { get; }
    public EntityBase? Owner { get; }

    // Physics
    private float _velocityX;
    private float _velocityY;
    private float _lifetime;
    private const float MaxLifetime = 3f;


    public Projectile(GoudGame game, EntityBase? owner, float x, float y, float velX, float velY, int damage)
        : base(game)
    {
        Owner = owner;
        X = x;
        Y = y;
        _velocityX = velX;
        _velocityY = velY;
        Damage = damage;
        _lifetime = MaxLifetime;

        Width = 8;
        Height = 8;
    }

    public override void Initialize()
    {
        // Projectiles will be set up by CombatSystem with proper texture
    }

    /// <summary>
    /// Set up projectile with texture and create sprite.
    /// </summary>
    public void Setup(uint textureId)
    {
        CreateSprite(textureId, X, Y, 50); // High z-layer for projectiles
        ScaleX = 0.5f;
        ScaleY = 0.5f;

        // Calculate rotation based on velocity
        Rotation = MathF.Atan2(_velocityY, _velocityX) * (180f / MathF.PI);
    }

    public override void Update(float deltaTime)
    {
        if (!IsActive) return;

        // Update position
        X += _velocityX * deltaTime;
        Y += _velocityY * deltaTime;

        // Update lifetime
        _lifetime -= deltaTime;

        // Check if out of bounds or expired
        if (_lifetime <= 0 || IsOutOfBounds())
        {
            Destroy();
            return;
        }

        // Update sprite
        UpdateSprite();
    }

    /// <summary>
    /// Check if projectile is outside screen bounds.
    /// </summary>
    private bool IsOutOfBounds()
    {
        const float padding = 50f;
        return X < -padding || X > GameManager.ScreenWidth + padding ||
               Y < -padding || Y > GameManager.ScreenHeight + padding;
    }

    /// <summary>
    /// Called when projectile hits something.
    /// </summary>
    public void OnHit()
    {
        // Could add hit effect here
        Destroy();
    }
}
