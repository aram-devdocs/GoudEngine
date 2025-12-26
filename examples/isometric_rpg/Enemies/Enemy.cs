using GoudEngine;
using CsBindgen;
using IsometricRpg.Core;
using IsometricRpg.Combat;

namespace IsometricRpg.Enemies;

/// <summary>
/// Enemy states for animation.
/// </summary>
public enum EnemyState
{
    Idle,
    Walking,
    Attack,
    Hurt,
    Death
}

/// <summary>
/// A basic enemy entity with AI behavior.
/// </summary>
public class Enemy : EntityBase
{
    // Components
    public HealthComponent Health { get; private set; } = null!;
    public EnemyAI AI { get; private set; } = null!;

    // Combat system reference
    private CombatSystem _combatSystem = null!;

    // Configuration
    public int MaxHealth { get; set; } = 50;

    // Animation
    private AnimationController? _animationController;
    private EnemyState _currentState = EnemyState.Idle;

    // Death handling
    private float _deathTimer;
    private const float DeathAnimationDuration = 1f;
    private bool _isDeathAnimationComplete;

    public Enemy(GoudGame game) : base(game)
    {
        Width = 32;
        Height = 32;
    }

    public override void Initialize()
    {
        Health = new HealthComponent(MaxHealth);
        Health.OnDeath += OnEnemyDeath;
        Health.OnDamageTaken += OnEnemyDamaged;
    }

    /// <summary>
    /// Set up the enemy with combat system and AI.
    /// </summary>
    public void Setup(CombatSystem combatSystem, EntityBase target, uint textureId, float x, float y)
    {
        _combatSystem = combatSystem;
        X = x;
        Y = y;

        // Create AI
        AI = new EnemyAI(Game, this, combatSystem);
        AI.SetTarget(target);

        // Create sprite
        CreateSprite(textureId, x, y, 10);

        // Register with combat system
        combatSystem.RegisterEnemy(this);
    }

    /// <summary>
    /// Set up animations for the enemy.
    /// </summary>
    public void SetupAnimations(Dictionary<string, AnimationStateConfig> animations)
    {
        _animationController = new AnimationController(Game, animations);
    }

    public override void Update(float deltaTime)
    {
        if (!IsActive) return;

        // Handle death animation
        if (Health.IsDead)
        {
            _deathTimer += deltaTime;
            if (_deathTimer >= DeathAnimationDuration && !_isDeathAnimationComplete)
            {
                _isDeathAnimationComplete = true;
                // Could destroy sprite or keep corpse visible
            }
            return;
        }

        // Update health (invincibility frames)
        Health.Update(deltaTime);

        // Update AI
        AI.Update(deltaTime);

        // Update animation state based on AI state
        UpdateAnimationState();

        // Update animation
        UpdateAnimation(deltaTime);

        // Update sprite position
        UpdateSprite();
    }

    /// <summary>
    /// Update animation state based on AI state.
    /// </summary>
    private void UpdateAnimationState()
    {
        EnemyState newState = AI.CurrentState switch
        {
            EnemyAIState.Idle => EnemyState.Idle,
            EnemyAIState.Chase => EnemyState.Walking,
            EnemyAIState.Attack => EnemyState.Attack,
            EnemyAIState.Hurt => EnemyState.Hurt,
            EnemyAIState.Death => EnemyState.Death,
            _ => EnemyState.Idle
        };

        if (_currentState != newState)
        {
            _currentState = newState;
        }
    }

    /// <summary>
    /// Update animation frame.
    /// </summary>
    private void UpdateAnimation(float deltaTime)
    {
        if (_animationController == null) return;

        string stateName = _currentState.ToString();

        try
        {
            var (frame, textureId) = _animationController.GetFrame(stateName, deltaTime);

            Game.UpdateSprite(new SpriteUpdateDto
            {
                id = SpriteId,
                texture_id = textureId,
                source_rect = frame,
                x = X,
                y = Y,
                z_layer = ZLayer
            });
        }
        catch (ArgumentException)
        {
            // Animation state not found - use default
        }
    }

    /// <summary>
    /// Handle enemy death.
    /// </summary>
    private void OnEnemyDeath()
    {
        _currentState = EnemyState.Death;
        Game.GameLog("Enemy died!");
    }

    /// <summary>
    /// Handle enemy taking damage.
    /// </summary>
    private void OnEnemyDamaged(int amount)
    {
        AI.OnDamaged();
        Game.GameLog($"Enemy took {amount} damage! Health: {Health.CurrentHealth}/{Health.MaxHealth}");
    }

    /// <summary>
    /// Clean up enemy.
    /// </summary>
    public override void Destroy()
    {
        _combatSystem?.UnregisterEnemy(this);
        base.Destroy();
    }

    /// <summary>
    /// Check if death animation is complete.
    /// </summary>
    public bool IsDeathAnimationComplete => _isDeathAnimationComplete;

    /// <summary>
    /// Get current animation state.
    /// </summary>
    public EnemyState CurrentState => _currentState;
}
