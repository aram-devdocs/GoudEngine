// Player/Player.cs
// Simple player entity

using IsometricRpg.Core;

namespace IsometricRpg.Player;

public class SimplePlayer : SimpleEntity
{
    public int Health { get; private set; } = 100;
    public int MaxHealth { get; } = 100;
    public bool IsDead => Health <= 0;
    public float HitFlash { get; private set; } = 0f;
    
    private float _invincibilityTimer = 0f;

    public SimplePlayer(float x, float y) : base(x, y)
    {
        Width = 32;
        Height = 32;
    }

    public override void Update(float deltaTime)
    {
        if (_invincibilityTimer > 0)
            _invincibilityTimer -= deltaTime;
        if (HitFlash > 0)
            HitFlash -= deltaTime;
    }

    public void Move(float dx, float dy)
    {
        X += dx;
        Y += dy;
        
        // Keep in bounds
        X = System.Math.Clamp(X, 0, GameManager.ScreenWidth - Width);
        Y = System.Math.Clamp(Y, 0, GameManager.ScreenHeight - Height);
    }

    public void TakeDamage(int damage)
    {
        if (_invincibilityTimer > 0 || IsDead) return;
        
        Health -= damage;
        HitFlash = 0.2f;
        _invincibilityTimer = 0.5f;
        
        if (Health < 0) Health = 0;
    }

    public void Reset(float x, float y)
    {
        X = x;
        Y = y;
        Health = MaxHealth;
        HitFlash = 0;
        _invincibilityTimer = 0;
        IsActive = true;
    }
}
