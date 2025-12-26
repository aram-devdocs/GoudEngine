using GoudEngine;
using CsBindgen;
using IsometricRpg.Core;

namespace IsometricRpg.UI;

/// <summary>
/// Visual health bar UI element.
/// </summary>
public class HealthBar
{
    private readonly GoudGame _game;

    // Sprite IDs
    private uint _backgroundSpriteId;
    private uint _fillSpriteId;

    // Position and size
    private float _x;
    private float _y;
    private float _maxWidth;
    private float _height;

    // State
    private float _currentPercent = 1f;
    private bool _isVisible = true;

    // Z-layer for UI
    private const int UiZLayer = 100;

    public HealthBar(GoudGame game, float x, float y, float width, float height)
    {
        _game = game;
        _x = x;
        _y = y;
        _maxWidth = width;
        _height = height;
    }

    /// <summary>
    /// Initialize health bar with textures.
    /// </summary>
    public void Initialize(uint backgroundTexture, uint fillTexture)
    {
        // Background (dark bar)
        _backgroundSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = backgroundTexture,
            x = _x,
            y = _y,
            z_layer = UiZLayer
        });

        // Fill (colored health portion)
        _fillSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = fillTexture,
            x = _x,
            y = _y,
            z_layer = UiZLayer + 1
        });
    }

    /// <summary>
    /// Update health bar from health component.
    /// </summary>
    public void UpdateFromHealth(HealthComponent health)
    {
        SetPercent(health.HealthPercent);
    }

    /// <summary>
    /// Set health bar fill percentage (0-1).
    /// </summary>
    public void SetPercent(float percent)
    {
        _currentPercent = Math.Clamp(percent, 0f, 1f);

        if (_fillSpriteId != 0 && _isVisible)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _fillSpriteId,
                scale_x = _currentPercent
            });
        }
    }

    /// <summary>
    /// Show the health bar.
    /// </summary>
    public void Show()
    {
        if (_isVisible) return;
        _isVisible = true;

        if (_backgroundSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _backgroundSpriteId,
                scale_x = 1f,
                scale_y = 1f
            });
        }

        if (_fillSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _fillSpriteId,
                scale_x = _currentPercent,
                scale_y = 1f
            });
        }
    }

    /// <summary>
    /// Hide the health bar.
    /// </summary>
    public void Hide()
    {
        if (!_isVisible) return;
        _isVisible = false;

        if (_backgroundSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _backgroundSpriteId,
                scale_x = 0f,
                scale_y = 0f
            });
        }

        if (_fillSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _fillSpriteId,
                scale_x = 0f,
                scale_y = 0f
            });
        }
    }

    /// <summary>
    /// Clean up sprites.
    /// </summary>
    public void Destroy()
    {
        if (_backgroundSpriteId != 0) _game.RemoveSprite(_backgroundSpriteId);
        if (_fillSpriteId != 0) _game.RemoveSprite(_fillSpriteId);
    }

    public bool IsVisible => _isVisible;
    public float CurrentPercent => _currentPercent;
}
