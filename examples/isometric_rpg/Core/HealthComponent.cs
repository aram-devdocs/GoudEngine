namespace IsometricRpg.Core;

/// <summary>
/// Component for managing entity health, damage, and death.
/// </summary>
public class HealthComponent
{
    public int MaxHealth { get; private set; }
    public int CurrentHealth { get; private set; }

    public bool IsDead => CurrentHealth <= 0;
    public float HealthPercent => MaxHealth > 0 ? (float)CurrentHealth / MaxHealth : 0f;

    // Invincibility frames after taking damage
    private float _invincibilityTimer = 0f;
    private const float InvincibilityDuration = 0.5f;
    public bool IsInvincible => _invincibilityTimer > 0;

    // Events
    public event Action<int, int>? OnHealthChanged;  // (current, max)
    public event Action<int>? OnDamageTaken;         // (amount)
    public event Action? OnDeath;
    public event Action<int>? OnHealed;              // (amount)

    public HealthComponent(int maxHealth)
    {
        MaxHealth = maxHealth;
        CurrentHealth = maxHealth;
    }

    /// <summary>
    /// Update invincibility timer.
    /// </summary>
    public void Update(float deltaTime)
    {
        if (_invincibilityTimer > 0)
        {
            _invincibilityTimer -= deltaTime;
        }
    }

    /// <summary>
    /// Take damage. Returns actual damage dealt.
    /// </summary>
    public int TakeDamage(int amount, bool ignoreInvincibility = false)
    {
        if (IsDead) return 0;
        if (IsInvincible && !ignoreInvincibility) return 0;
        if (amount <= 0) return 0;

        int actualDamage = Math.Min(amount, CurrentHealth);
        CurrentHealth -= actualDamage;

        // Start invincibility frames
        _invincibilityTimer = InvincibilityDuration;

        OnDamageTaken?.Invoke(actualDamage);
        OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);

        if (IsDead)
        {
            OnDeath?.Invoke();
        }

        return actualDamage;
    }

    /// <summary>
    /// Heal by amount. Returns actual amount healed.
    /// </summary>
    public int Heal(int amount)
    {
        if (IsDead) return 0;
        if (amount <= 0) return 0;

        int actualHeal = Math.Min(amount, MaxHealth - CurrentHealth);
        CurrentHealth += actualHeal;

        if (actualHeal > 0)
        {
            OnHealed?.Invoke(actualHeal);
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
        }

        return actualHeal;
    }

    /// <summary>
    /// Set health to max.
    /// </summary>
    public void FullHeal()
    {
        int healed = MaxHealth - CurrentHealth;
        CurrentHealth = MaxHealth;
        if (healed > 0)
        {
            OnHealed?.Invoke(healed);
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
        }
    }

    /// <summary>
    /// Reset health and state (for respawning).
    /// </summary>
    public void Reset()
    {
        CurrentHealth = MaxHealth;
        _invincibilityTimer = 0;
        OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
    }

    /// <summary>
    /// Set new max health (optionally heal to new max).
    /// </summary>
    public void SetMaxHealth(int newMax, bool healToMax = false)
    {
        MaxHealth = Math.Max(1, newMax);
        if (healToMax || CurrentHealth > MaxHealth)
        {
            CurrentHealth = MaxHealth;
        }
        OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
    }
}
