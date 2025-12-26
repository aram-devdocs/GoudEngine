using GoudEngine;
using IsometricRpg.Core;
using IsometricRpg.Enemies;
using IsometricRpg.Player;

namespace IsometricRpg.Combat;

/// <summary>
/// Represents an active melee hitbox.
/// </summary>
public class MeleeHitbox
{
    public float X { get; }
    public float Y { get; }
    public float Radius { get; }
    public int Damage { get; }
    public EntityBase Owner { get; }
    public float Duration { get; private set; }
    public HashSet<EntityBase> HitEntities { get; } = new();

    public MeleeHitbox(EntityBase owner, float x, float y, float radius, int damage, float duration)
    {
        Owner = owner;
        X = x;
        Y = y;
        Radius = radius;
        Damage = damage;
        Duration = duration;
    }

    public void Update(float deltaTime)
    {
        Duration -= deltaTime;
    }

    public bool IsExpired => Duration <= 0;

    public bool IsInRange(float targetX, float targetY)
    {
        float dx = targetX - X;
        float dy = targetY - Y;
        float distance = MathF.Sqrt(dx * dx + dy * dy);
        return distance <= Radius;
    }
}

/// <summary>
/// Manages all combat interactions: melee hitboxes, projectiles, and damage resolution.
/// </summary>
public class CombatSystem
{
    private readonly GoudGame _game;

    // Active combat elements
    private readonly List<MeleeHitbox> _activeHitboxes = new();
    private readonly List<Projectile> _activeProjectiles = new();

    // Entity references
    private Player.Player? _player;
    private readonly List<Enemy> _enemies = new();

    // Projectile texture
    private uint _projectileTextureId;

    // Events
    public event Action<EntityBase, int>? OnEntityDamaged;
    public event Action<EntityBase>? OnEntityKilled;

    public CombatSystem(GoudGame game)
    {
        _game = game;
    }

    /// <summary>
    /// Initialize combat system with texture for projectiles.
    /// </summary>
    public void Initialize(uint projectileTextureId)
    {
        _projectileTextureId = projectileTextureId;
    }

    /// <summary>
    /// Register the player with the combat system.
    /// </summary>
    public void RegisterPlayer(Player.Player player)
    {
        _player = player;

        // Subscribe to player combat events
        player.Combat.OnMeleeAttack += HandlePlayerMeleeAttack;
        player.Combat.OnRangedAttack += HandlePlayerRangedAttack;
    }

    /// <summary>
    /// Register an enemy with the combat system.
    /// </summary>
    public void RegisterEnemy(Enemy enemy)
    {
        _enemies.Add(enemy);
    }

    /// <summary>
    /// Unregister an enemy from the combat system.
    /// </summary>
    public void UnregisterEnemy(Enemy enemy)
    {
        _enemies.Remove(enemy);
    }

    /// <summary>
    /// Clear all enemies.
    /// </summary>
    public void ClearEnemies()
    {
        _enemies.Clear();
    }

    /// <summary>
    /// Update combat system - check collisions and apply damage.
    /// </summary>
    public void Update(float deltaTime)
    {
        UpdateHitboxes(deltaTime);
        UpdateProjectiles(deltaTime);
        CheckCollisions();
    }

    /// <summary>
    /// Update active melee hitboxes.
    /// </summary>
    private void UpdateHitboxes(float deltaTime)
    {
        for (int i = _activeHitboxes.Count - 1; i >= 0; i--)
        {
            var hitbox = _activeHitboxes[i];
            hitbox.Update(deltaTime);

            if (hitbox.IsExpired)
            {
                _activeHitboxes.RemoveAt(i);
            }
        }
    }

    /// <summary>
    /// Update active projectiles.
    /// </summary>
    private void UpdateProjectiles(float deltaTime)
    {
        for (int i = _activeProjectiles.Count - 1; i >= 0; i--)
        {
            var projectile = _activeProjectiles[i];
            projectile.Update(deltaTime);

            if (!projectile.IsActive)
            {
                _activeProjectiles.RemoveAt(i);
            }
        }
    }

    /// <summary>
    /// Check all combat collisions.
    /// </summary>
    private void CheckCollisions()
    {
        // Check melee hitboxes against enemies
        foreach (var hitbox in _activeHitboxes)
        {
            if (hitbox.Owner == _player)
            {
                // Player's hitbox - check against enemies
                foreach (var enemy in _enemies)
                {
                    if (!enemy.IsActive || enemy.Health.IsDead) continue;
                    if (hitbox.HitEntities.Contains(enemy)) continue;

                    var (ex, ey) = enemy.GetCenter();
                    if (hitbox.IsInRange(ex, ey))
                    {
                        ApplyDamage(enemy, hitbox.Damage);
                        hitbox.HitEntities.Add(enemy);
                    }
                }
            }
        }

        // Check projectiles against enemies
        foreach (var projectile in _activeProjectiles.ToList())
        {
            if (!projectile.IsActive) continue;

            if (projectile.Owner == _player)
            {
                // Player's projectile - check against enemies
                foreach (var enemy in _enemies)
                {
                    if (!enemy.IsActive || enemy.Health.IsDead) continue;

                    if (_game.CheckCollision(projectile.SpriteId, enemy.SpriteId))
                    {
                        ApplyDamage(enemy, projectile.Damage);
                        projectile.OnHit();
                        break;
                    }
                }
            }
            else
            {
                // Enemy projectile - check against player
                if (_player != null && _player.IsActive && !_player.Health.IsDead)
                {
                    if (_game.CheckCollision(projectile.SpriteId, _player.SpriteId))
                    {
                        ApplyDamage(_player, projectile.Damage);
                        projectile.OnHit();
                    }
                }
            }
        }

        // Check enemy melee attacks against player
        foreach (var enemy in _enemies)
        {
            if (!enemy.IsActive || enemy.Health.IsDead) continue;

            // Enemy attack check is handled by EnemyAI
        }
    }

    /// <summary>
    /// Apply damage to an entity.
    /// </summary>
    public void ApplyDamage(EntityBase target, int damage)
    {
        HealthComponent? health = target switch
        {
            Player.Player player => player.Health,
            Enemy enemy => enemy.Health,
            _ => null
        };

        if (health == null) return;

        int actualDamage = health.TakeDamage(damage);

        if (actualDamage > 0)
        {
            OnEntityDamaged?.Invoke(target, actualDamage);

            if (health.IsDead)
            {
                OnEntityKilled?.Invoke(target);
            }
        }
    }

    /// <summary>
    /// Handle player melee attack.
    /// </summary>
    private void HandlePlayerMeleeAttack(float x, float y, int direction, float range)
    {
        if (_player == null) return;

        var hitbox = new MeleeHitbox(
            _player,
            x, y,
            range,
            _player.Combat.MeleeAttackDamage,
            _player.Combat.MeleeAttackDuration
        );

        _activeHitboxes.Add(hitbox);
    }

    /// <summary>
    /// Handle player ranged attack - spawn projectile.
    /// </summary>
    private void HandlePlayerRangedAttack(float x, float y, float velX, float velY, int damage)
    {
        if (_player == null) return;

        var projectile = new Projectile(_game, _player, x, y, velX, velY, damage);
        projectile.Initialize();
        projectile.Setup(_projectileTextureId);

        _activeProjectiles.Add(projectile);
    }

    /// <summary>
    /// Create an enemy attack hitbox.
    /// </summary>
    public void CreateEnemyAttack(Enemy enemy, float x, float y, float radius, int damage, float duration)
    {
        var hitbox = new MeleeHitbox(enemy, x, y, radius, damage, duration);
        _activeHitboxes.Add(hitbox);

        // Check immediate collision with player
        if (_player != null && _player.IsActive && !_player.Health.IsDead)
        {
            var (px, py) = _player.GetCenter();
            if (hitbox.IsInRange(px, py))
            {
                ApplyDamage(_player, damage);
                hitbox.HitEntities.Add(_player);
            }
        }
    }

    /// <summary>
    /// Get active enemies for AI and other systems.
    /// </summary>
    public IReadOnlyList<Enemy> GetEnemies() => _enemies;

    /// <summary>
    /// Get active projectile count.
    /// </summary>
    public int ProjectileCount => _activeProjectiles.Count;

    /// <summary>
    /// Clean up all combat elements.
    /// </summary>
    public void Clear()
    {
        _activeHitboxes.Clear();

        foreach (var projectile in _activeProjectiles)
        {
            projectile.Destroy();
        }
        _activeProjectiles.Clear();
    }
}
