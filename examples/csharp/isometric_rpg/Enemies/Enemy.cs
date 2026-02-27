// Enemies/Enemy.cs
// Simple enemy entity

using IsometricRpg.Core;

namespace IsometricRpg.Enemies;

public class SimpleEnemy : SimpleEntity
{
    public int Health { get; private set; } = 50;
    public bool IsDead => Health <= 0;
    public float HitFlash { get; private set; } = 0f;
    public float DeathTimer { get; private set; } = 0f;
    
    private float _speed = 80f;

    public SimpleEnemy(float x, float y) : base(x, y)
    {
        Width = 32;
        Height = 32;
    }

    public override void Update(float deltaTime)
    {
        if (HitFlash > 0)
            HitFlash -= deltaTime;
            
        if (IsDead && DeathTimer > 0)
            DeathTimer -= deltaTime;
    }

    public void MoveTowards(float targetX, float targetY, float deltaTime)
    {
        if (IsDead) return;
        
        float dx = targetX - X;
        float dy = targetY - Y;
        float dist = (float)System.Math.Sqrt(dx * dx + dy * dy);
        
        if (dist > 5)
        {
            X += (dx / dist) * _speed * deltaTime;
            Y += (dy / dist) * _speed * deltaTime;
        }
    }

    public void TakeDamage(int damage)
    {
        if (IsDead) return;
        
        Health -= damage;
        HitFlash = 0.15f;
        
        if (Health <= 0)
        {
            Health = 0;
            DeathTimer = 0.5f; // Time before removal
        }
    }
}
