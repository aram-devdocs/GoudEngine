using GoudEngine;
using CsBindgen;

namespace IsometricRpg.UI;

/// <summary>
/// Title screen with game title and start prompt.
/// </summary>
public class TitleScreen
{
    private readonly GoudGame _game;

    // Sprite IDs
    private uint _titleSpriteId;
    private uint _promptSpriteId;

    // Prompt animation
    private float _promptTimer;
    private bool _promptVisible = true;
    private const float PromptBlinkRate = 0.6f;

    // State
    private bool _isVisible;

    public TitleScreen(GoudGame game)
    {
        _game = game;
    }

    /// <summary>
    /// Initialize title screen with textures.
    /// </summary>
    public void Initialize(uint titleTexture, uint promptTexture)
    {
        // Create title (centered near top)
        _titleSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = titleTexture,
            x = GameManager.ScreenWidth / 2 - 200,
            y = 150,
            z_layer = 101
        });

        // Create "Press SPACE to Start" prompt (centered lower)
        _promptSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = promptTexture,
            x = GameManager.ScreenWidth / 2 - 150,
            y = 400,
            z_layer = 101
        });

        _isVisible = true;
    }

    /// <summary>
    /// Update title screen (prompt blinking).
    /// </summary>
    public void Update(float deltaTime)
    {
        if (!_isVisible) return;

        // Blink the prompt
        _promptTimer += deltaTime;
        if (_promptTimer >= PromptBlinkRate)
        {
            _promptTimer = 0;
            _promptVisible = !_promptVisible;

            if (_promptSpriteId != 0)
            {
                _game.UpdateSprite(new SpriteUpdateDto
                {
                    id = _promptSpriteId,
                    scale_x = _promptVisible ? 1f : 0f,
                    scale_y = _promptVisible ? 1f : 0f
                });
            }
        }
    }

    /// <summary>
    /// Show the title screen.
    /// </summary>
    public void Show()
    {
        if (_isVisible) return;
        _isVisible = true;
        SetSpritesVisible(true);
    }

    /// <summary>
    /// Hide the title screen.
    /// </summary>
    public void Hide()
    {
        if (!_isVisible) return;
        _isVisible = false;
        SetSpritesVisible(false);
    }

    private void SetSpritesVisible(bool visible)
    {
        float scale = visible ? 1f : 0f;

        if (_titleSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _titleSpriteId,
                scale_x = scale,
                scale_y = scale
            });
        }

        if (_promptSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _promptSpriteId,
                scale_x = scale,
                scale_y = scale
            });
        }
    }

    /// <summary>
    /// Clean up sprites.
    /// </summary>
    public void Destroy()
    {
        if (_titleSpriteId != 0) _game.RemoveSprite(_titleSpriteId);
        if (_promptSpriteId != 0) _game.RemoveSprite(_promptSpriteId);
    }

    public bool IsVisible => _isVisible;
}
