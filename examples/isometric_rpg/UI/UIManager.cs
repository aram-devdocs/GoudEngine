using GoudEngine;
using IsometricRpg.NPCs;

namespace IsometricRpg.UI;

/// <summary>
/// Manages all UI elements and coordinates visibility based on game state.
/// </summary>
public class UIManager
{
    private readonly GoudGame _game;

    // UI Components
    public TitleScreen TitleScreen { get; private set; } = null!;
    public HealthBar HealthBar { get; private set; } = null!;
    public DialogueUI DialogueUI { get; private set; } = null!;

    // Game over screen sprites
    private uint _gameOverSpriteId;
    private uint _restartPromptSpriteId;

    // State
    private GameState _currentState;

    public UIManager(GoudGame game)
    {
        _game = game;
    }

    /// <summary>
    /// Initialize all UI components with proper textures.
    /// </summary>
    public void Initialize(
        DialogueSystem dialogueSystem,
        uint healthBgTexture,
        uint healthFillTexture,
        uint dialogueBoxTexture,
        uint arrowTexture,
        uint titleTexture,
        uint pressStartTexture)
    {
        // Create title screen
        TitleScreen = new TitleScreen(_game);
        TitleScreen.Initialize(titleTexture, pressStartTexture);

        // Create health bar (top-left corner)
        HealthBar = new HealthBar(_game, 20, 20, 150, 20);
        HealthBar.Initialize(healthBgTexture, healthFillTexture);
        HealthBar.Hide();

        // Create dialogue UI
        DialogueUI = new DialogueUI(_game, dialogueSystem);
        DialogueUI.Initialize(dialogueBoxTexture, arrowTexture);
    }

    /// <summary>
    /// Update UI based on current game state.
    /// </summary>
    public void Update(float deltaTime, GameState state)
    {
        if (_currentState != state)
        {
            OnStateChanged(_currentState, state);
            _currentState = state;
        }

        switch (state)
        {
            case GameState.Title:
                TitleScreen.Update(deltaTime);
                break;

            case GameState.Playing:
                // Health bar updates are handled externally
                break;

            case GameState.Dialogue:
                DialogueUI.Update(deltaTime);
                break;

            case GameState.GameOver:
                // Could add game over animations
                break;
        }
    }

    /// <summary>
    /// Handle state changes.
    /// </summary>
    private void OnStateChanged(GameState oldState, GameState newState)
    {
        // Hide old state UI
        switch (oldState)
        {
            case GameState.Title:
                TitleScreen.Hide();
                break;
            case GameState.GameOver:
                HideGameOver();
                break;
        }

        // Show new state UI
        switch (newState)
        {
            case GameState.Title:
                TitleScreen.Show();
                HealthBar.Hide();
                break;

            case GameState.Playing:
                TitleScreen.Hide();
                HealthBar.Show();
                break;

            case GameState.Dialogue:
                // DialogueUI handles its own visibility via events
                break;

            case GameState.GameOver:
                ShowGameOver();
                break;
        }
    }

    /// <summary>
    /// Update health bar from player health.
    /// </summary>
    public void UpdateHealthBar(float healthPercent)
    {
        HealthBar.SetPercent(healthPercent);
    }

    /// <summary>
    /// Show game over screen.
    /// </summary>
    private void ShowGameOver()
    {
        // For MVP, just log - could add game over sprites
        _game.GameLog("=== GAME OVER ===");
        _game.GameLog("Press R to restart");
    }

    /// <summary>
    /// Hide game over screen.
    /// </summary>
    private void HideGameOver()
    {
        if (_gameOverSpriteId != 0)
        {
            _game.RemoveSprite(_gameOverSpriteId);
            _gameOverSpriteId = 0;
        }
        if (_restartPromptSpriteId != 0)
        {
            _game.RemoveSprite(_restartPromptSpriteId);
            _restartPromptSpriteId = 0;
        }
    }

    /// <summary>
    /// Clean up all UI.
    /// </summary>
    public void Destroy()
    {
        TitleScreen.Destroy();
        HealthBar.Destroy();
        DialogueUI.Destroy();
        HideGameOver();
    }
}
