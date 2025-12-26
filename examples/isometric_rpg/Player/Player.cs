using GoudEngine;
using CsBindgen;
using IsometricRpg.Core;

namespace IsometricRpg.Player;

/// <summary>
/// Player states for animation.
/// </summary>
public enum PlayerState
{
    Idle,
    Walking,
    MeleeAttack,
    RangedAttack,
    Hurt,
    Death
}

/// <summary>
/// The player character entity.
/// </summary>
public class Player : EntityBase
{
    // Components
    public HealthComponent Health { get; private set; } = null!;
    public PlayerController Controller { get; private set; } = null!;
    public PlayerCombat Combat { get; private set; } = null!;

    // Animation
    private AnimationController? _animationController;
    private PlayerState _currentState = PlayerState.Idle;

    // Configuration
    public int MaxHealth { get; set; } = 100;

    // State tracking
    private float _hurtTimer;
    private const float HurtDuration = 0.3f;

    public Player(GoudGame game) : base(game)
    {
        Width = 32;
        Height = 32;
    }

    public override void Initialize()
    {
        // Initialize components
        Health = new HealthComponent(MaxHealth);
        Controller = new PlayerController(Game, this);
        Combat = new PlayerCombat(Game, this);

        // Subscribe to health events
        Health.OnDeath += OnPlayerDeath;
        Health.OnDamageTaken += OnPlayerDamaged;

        // Start in center of screen
        X = GameManager.ScreenWidth / 2 - Width / 2;
        Y = GameManager.ScreenHeight / 2 - Height / 2;

        // Create placeholder sprite (will be replaced with actual texture)
        // For now, create without texture - will be set up in GameManager
    }

    /// <summary>
    /// Set up the player sprite with a texture.
    /// </summary>
    public void SetupSprite(uint textureId)
    {
        CreateSprite(textureId, X, Y, 10);
    }

    /// <summary>
    /// Set up animations for the player.
    /// </summary>
    public void SetupAnimations(Dictionary<string, AnimationStateConfig> animations)
    {
        _animationController = new AnimationController(Game, animations);
    }

    public override void Update(float deltaTime)
    {
        if (!IsActive) return;

        // Update health component (for invincibility frames)
        Health.Update(deltaTime);

        // Update hurt timer
        if (_hurtTimer > 0)
        {
            _hurtTimer -= deltaTime;
            if (_hurtTimer <= 0)
            {
                SetState(PlayerState.Idle);
            }
            return; // Don't process input while hurt
        }

        // Don't move while attacking
        if (_currentState == PlayerState.Death)
        {
            return;
        }

        // Update combat first (may set attacking state)
        Combat.Update(deltaTime);

        if (Combat.IsAttacking)
        {
            SetState(PlayerState.MeleeAttack);
        }
        else
        {
            // Handle movement input
            bool isMoving = Controller.HandleInput(deltaTime);

            if (isMoving)
            {
                SetState(PlayerState.Walking);
            }
            else
            {
                SetState(PlayerState.Idle);
            }
        }

        // Update animation
        UpdateAnimation(deltaTime);

        // Update sprite position
        UpdateSprite();
    }

    /// <summary>
    /// Update current animation frame.
    /// </summary>
    private void UpdateAnimation(float deltaTime)
    {
        if (_animationController == null) return;

        // Get animation state name based on current state and direction
        // For MVP, we'll use simplified state names
        string stateName = GetAnimationStateName();

        try
        {
            var (frame, textureId) = _animationController.GetFrame(stateName, deltaTime);

            // Update sprite with new frame
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
            // This is expected if not all animations are defined
        }
    }

    /// <summary>
    /// Get animation state name for current state and direction.
    /// </summary>
    private string GetAnimationStateName()
    {
        // Simplified: just use state name without direction for MVP
        // Full implementation would be: $"{_currentState}_{Direction}"
        return _currentState.ToString();
    }

    /// <summary>
    /// Set the current player state.
    /// </summary>
    public void SetState(PlayerState newState)
    {
        if (_currentState == newState) return;
        _currentState = newState;
    }

    /// <summary>
    /// Get current player state.
    /// </summary>
    public PlayerState CurrentState => _currentState;

    /// <summary>
    /// Handle player death.
    /// </summary>
    private void OnPlayerDeath()
    {
        SetState(PlayerState.Death);
        Game.GameLog("Player died!");
    }

    /// <summary>
    /// Handle player taking damage.
    /// </summary>
    private void OnPlayerDamaged(int amount)
    {
        SetState(PlayerState.Hurt);
        _hurtTimer = HurtDuration;
        Game.GameLog($"Player took {amount} damage! Health: {Health.CurrentHealth}/{Health.MaxHealth}");
    }

    /// <summary>
    /// Reset player to initial state (for restart).
    /// </summary>
    public void Reset()
    {
        X = GameManager.ScreenWidth / 2 - Width / 2;
        Y = GameManager.ScreenHeight / 2 - Height / 2;
        Direction = 3; // Face down
        Health.Reset();
        SetState(PlayerState.Idle);
        IsActive = true;
        UpdateSprite();
    }

    /// <summary>
    /// Check if player is dead.
    /// </summary>
    public bool IsDead => Health.IsDead;
}
