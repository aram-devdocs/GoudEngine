// Combat/Projectile.cs
// Simple projectile entity

using IsometricRpg.Core;

namespace IsometricRpg.Combat;

public class SimpleProjectile : SimpleEntity
{
    private float _velocityX;
    private float _velocityY;
    private float _lifetime = 2f;
    private const float Speed = 400f;

    public SimpleProjectile(float startX, float startY, float targetX, float targetY) 
        : base(startX, startY)
    {
        Width = 16;
        Height = 16;
        
        // Calculate direction
        float dx = targetX - startX;
        float dy = targetY - startY;
        float dist = (float)System.Math.Sqrt(dx * dx + dy * dy);
        
        if (dist > 0)
        {
            _velocityX = (dx / dist) * Speed;
            _velocityY = (dy / dist) * Speed;
        }
    }

    public override void Update(float deltaTime)
    {
        X += _velocityX * deltaTime;
        Y += _velocityY * deltaTime;
        
        _lifetime -= deltaTime;
        
        // Deactivate if out of bounds or expired
        if (_lifetime <= 0 || X < -50 || X > GameManager.ScreenWidth + 50 ||
            Y < -50 || Y > GameManager.ScreenHeight + 50)
        {
            IsActive = false;
        }
    }
}
