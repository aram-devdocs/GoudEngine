using GoudEngine;
using IsometricRpg.Core;
using IsometricRpg.Combat;

namespace IsometricRpg.Enemies;

/// <summary>
/// AI states for enemy behavior.
/// </summary>
public enum EnemyAIState
{
    Idle,
    Chase,
    Attack,
    Hurt,
    Death
}

/// <summary>
/// Simple AI controller for enemies with chase and attack behavior.
/// </summary>
public class EnemyAI
{
    private readonly GoudGame _game;
    private readonly Enemy _enemy;
    private readonly CombatSystem _combatSystem;

    // AI Configuration
    public float DetectionRange { get; set; } = 200f;
    public float AttackRange { get; set; } = 40f;
    public float MoveSpeed { get; set; } = 80f;
    public float AttackCooldown { get; set; } = 1.5f;
    public float AttackDuration { get; set; } = 0.4f;
    public int AttackDamage { get; set; } = 10;

    // State
    public EnemyAIState CurrentState { get; private set; } = EnemyAIState.Idle;

    // Target (player)
    private EntityBase? _target;

    // Timers
    private float _attackCooldownTimer;
    private float _attackActiveTimer;
    private float _hurtTimer;
    private const float HurtDuration = 0.3f;

    // Idle behavior
    private float _idleTimer;
    private const float IdleWaitTime = 1f;

    public EnemyAI(GoudGame game, Enemy enemy, CombatSystem combatSystem)
    {
        _game = game;
        _enemy = enemy;
        _combatSystem = combatSystem;
    }

    /// <summary>
    /// Set the target for the AI to chase/attack.
    /// </summary>
    public void SetTarget(EntityBase? target)
    {
        _target = target;
    }

    /// <summary>
    /// Update AI behavior.
    /// </summary>
    public void Update(float deltaTime)
    {
        if (!_enemy.IsActive || _enemy.Health.IsDead)
        {
            SetState(EnemyAIState.Death);
            return;
        }

        // Update timers
        if (_attackCooldownTimer > 0) _attackCooldownTimer -= deltaTime;
        if (_hurtTimer > 0)
        {
            _hurtTimer -= deltaTime;
            if (_hurtTimer <= 0 && CurrentState == EnemyAIState.Hurt)
            {
                SetState(EnemyAIState.Idle);
            }
            return; // Don't process AI while hurt
        }

        if (_attackActiveTimer > 0)
        {
            _attackActiveTimer -= deltaTime;
            if (_attackActiveTimer <= 0)
            {
                SetState(EnemyAIState.Chase);
            }
            return; // Don't move while attacking
        }

        // No target - stay idle
        if (_target == null || !_target.IsActive)
        {
            SetState(EnemyAIState.Idle);
            return;
        }

        float distanceToTarget = _enemy.DistanceTo(_target);

        switch (CurrentState)
        {
            case EnemyAIState.Idle:
                UpdateIdle(deltaTime, distanceToTarget);
                break;

            case EnemyAIState.Chase:
                UpdateChase(deltaTime, distanceToTarget);
                break;

            case EnemyAIState.Attack:
                // Handled by attack timer above
                break;

            case EnemyAIState.Death:
                // Do nothing
                break;
        }
    }

    /// <summary>
    /// Update idle state - check if target comes in range.
    /// </summary>
    private void UpdateIdle(float deltaTime, float distanceToTarget)
    {
        _idleTimer += deltaTime;

        if (distanceToTarget <= DetectionRange)
        {
            SetState(EnemyAIState.Chase);
        }
    }

    /// <summary>
    /// Update chase state - move toward target, attack if in range.
    /// </summary>
    private void UpdateChase(float deltaTime, float distanceToTarget)
    {
        // Lost target - go back to idle
        if (distanceToTarget > DetectionRange * 1.5f)
        {
            SetState(EnemyAIState.Idle);
            return;
        }

        // In attack range - try to attack
        if (distanceToTarget <= AttackRange && _attackCooldownTimer <= 0)
        {
            PerformAttack();
            return;
        }

        // Move toward target
        MoveTowardTarget(deltaTime);
    }

    /// <summary>
    /// Move the enemy toward the target.
    /// </summary>
    private void MoveTowardTarget(float deltaTime)
    {
        if (_target == null) return;

        float dx = _target.X - _enemy.X;
        float dy = _target.Y - _enemy.Y;

        var (normX, normY) = IsometricUtils.Normalize(dx, dy);

        _enemy.X += normX * MoveSpeed * deltaTime;
        _enemy.Y += normY * MoveSpeed * deltaTime;

        // Update facing direction
        _enemy.Direction = IsometricUtils.GetDirectionFromDelta(dx, dy);
        if (_enemy.Direction < 0) _enemy.Direction = 3; // Default down
    }

    /// <summary>
    /// Perform an attack on the target.
    /// </summary>
    private void PerformAttack()
    {
        SetState(EnemyAIState.Attack);
        _attackCooldownTimer = AttackCooldown;
        _attackActiveTimer = AttackDuration;

        // Get attack position (in front of enemy)
        var (centerX, centerY) = _enemy.GetCenter();

        // Create hitbox through combat system
        _combatSystem.CreateEnemyAttack(
            _enemy,
            centerX,
            centerY,
            AttackRange,
            AttackDamage,
            AttackDuration
        );

        _game.GameLog($"Enemy attacks! Damage: {AttackDamage}");
    }

    /// <summary>
    /// Called when enemy takes damage.
    /// </summary>
    public void OnDamaged()
    {
        SetState(EnemyAIState.Hurt);
        _hurtTimer = HurtDuration;
    }

    /// <summary>
    /// Set AI state.
    /// </summary>
    private void SetState(EnemyAIState newState)
    {
        if (CurrentState == newState) return;
        CurrentState = newState;
        _idleTimer = 0;
    }
}
