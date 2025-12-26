using GoudEngine;
using IsometricRpg.Core;

namespace IsometricRpg.Player;

/// <summary>
/// Handles player combat actions: melee and ranged attacks.
/// </summary>
public class PlayerCombat
{
    private readonly GoudGame _game;
    private readonly Player _player;

    // Melee attack configuration
    public float MeleeAttackCooldown { get; set; } = 0.4f;
    public float MeleeAttackDuration { get; set; } = 0.3f;
    public int MeleeAttackDamage { get; set; } = 25;
    public float MeleeAttackRange { get; set; } = 40f;

    // Ranged attack configuration
    public float RangedAttackCooldown { get; set; } = 0.8f;
    public int RangedAttackDamage { get; set; } = 15;
    public float ProjectileSpeed { get; set; } = 300f;

    // Attack state
    private float _meleeCooldownTimer;
    private float _rangedCooldownTimer;
    private float _meleeActiveTimer;

    public bool IsMeleeAttacking => _meleeActiveTimer > 0;
    public bool IsAttacking => IsMeleeAttacking;

    // Melee hitbox sprite
    private uint _meleeHitboxSpriteId;

    // Events for combat system integration
    public event Action<float, float, int, float>? OnMeleeAttack; // (x, y, direction, range)
    public event Action<float, float, float, float, int>? OnRangedAttack; // (x, y, velX, velY, damage)

    // Mouse buttons
    private const int MouseLeft = 0;
    private const int MouseRight = 1;

    public PlayerCombat(GoudGame game, Player player)
    {
        _game = game;
        _player = player;
    }

    /// <summary>
    /// Handle combat input and update attack states.
    /// </summary>
    public void Update(float deltaTime)
    {
        // Update cooldowns
        if (_meleeCooldownTimer > 0) _meleeCooldownTimer -= deltaTime;
        if (_rangedCooldownTimer > 0) _rangedCooldownTimer -= deltaTime;
        if (_meleeActiveTimer > 0)
        {
            _meleeActiveTimer -= deltaTime;
            if (_meleeActiveTimer <= 0)
            {
                DeactivateMeleeHitbox();
            }
        }

        // Don't process input if already attacking
        if (IsAttacking) return;

        // Left mouse = Melee attack
        if (_game.IsMouseButtonPressed(MouseLeft) && _meleeCooldownTimer <= 0)
        {
            PerformMeleeAttack();
        }

        // Right mouse = Ranged attack
        if (_game.IsMouseButtonPressed(MouseRight) && _rangedCooldownTimer <= 0)
        {
            PerformRangedAttack();
        }
    }

    /// <summary>
    /// Perform melee attack in player's facing direction.
    /// </summary>
    private void PerformMeleeAttack()
    {
        _meleeCooldownTimer = MeleeAttackCooldown;
        _meleeActiveTimer = MeleeAttackDuration;

        // Calculate hitbox position based on direction
        var (offsetX, offsetY) = GetDirectionOffset(_player.Direction, MeleeAttackRange);

        float hitboxX = _player.X + _player.Width / 2 + offsetX;
        float hitboxY = _player.Y + _player.Height / 2 + offsetY;

        // Notify combat system
        OnMeleeAttack?.Invoke(hitboxX, hitboxY, _player.Direction, MeleeAttackRange);

        _game.GameLog($"Melee attack! Direction: {_player.Direction}");
    }

    /// <summary>
    /// Perform ranged attack toward mouse position.
    /// </summary>
    private void PerformRangedAttack()
    {
        _rangedCooldownTimer = RangedAttackCooldown;

        // Get direction vector based on player facing
        var (dirX, dirY) = GetDirectionOffset(_player.Direction, 1f);
        var (normX, normY) = IsometricUtils.Normalize(dirX, dirY);

        float startX = _player.X + _player.Width / 2;
        float startY = _player.Y + _player.Height / 2;

        float velX = normX * ProjectileSpeed;
        float velY = normY * ProjectileSpeed;

        // Notify combat system to spawn projectile
        OnRangedAttack?.Invoke(startX, startY, velX, velY, RangedAttackDamage);

        _game.GameLog($"Ranged attack! Velocity: ({velX:F1}, {velY:F1})");
    }

    /// <summary>
    /// Get offset position for a direction.
    /// </summary>
    private (float x, float y) GetDirectionOffset(int direction, float distance)
    {
        // Direction 0-7, clockwise from North
        return direction switch
        {
            0 => (distance * 0.7f, -distance * 0.7f),   // N (up-right)
            1 => (distance, 0),                          // NE (right)
            2 => (distance * 0.7f, distance * 0.7f),    // E (down-right)
            3 => (0, distance),                          // SE (down)
            4 => (-distance * 0.7f, distance * 0.7f),   // S (down-left)
            5 => (-distance, 0),                         // SW (left)
            6 => (-distance * 0.7f, -distance * 0.7f),  // W (up-left)
            7 => (0, -distance),                         // NW (up)
            _ => (0, distance)                           // Default: down
        };
    }

    /// <summary>
    /// Deactivate melee hitbox when attack ends.
    /// </summary>
    private void DeactivateMeleeHitbox()
    {
        if (_meleeHitboxSpriteId != 0)
        {
            _game.RemoveSprite(_meleeHitboxSpriteId);
            _meleeHitboxSpriteId = 0;
        }
    }

    /// <summary>
    /// Check if can attack (not on cooldown and not already attacking).
    /// </summary>
    public bool CanMeleeAttack => _meleeCooldownTimer <= 0 && !IsAttacking;
    public bool CanRangedAttack => _rangedCooldownTimer <= 0 && !IsAttacking;
}
