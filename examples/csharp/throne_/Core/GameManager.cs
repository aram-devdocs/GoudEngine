// Core/GameManager.cs
// Main game manager for throne_

using GoudEngine.Input;
using Throne.Scenes;

namespace Throne.Core;

/// <summary>
/// Main game manager handling initialization, updates, and scene management.
/// </summary>
public class GameManager
{
    private readonly GoudGame _game;
    private readonly GameStateManager _stateManager;
    private IScene? _currentScene;

    public const uint ScreenWidth = 800;
    public const uint ScreenHeight = 600;

    public GameManager(GoudGame game)
    {
        _game = game;
        _stateManager = new GameStateManager();
        _stateManager.OnStateChanged += OnStateChanged;
    }

    public void Initialize()
    {
        _game.GameLog("throne_ initializing...");

        // Start with main menu
        TransitionToScene(new MainMenuScene(_game, ScreenWidth, ScreenHeight));
        _stateManager.SetState(GameState.MainMenu);

        _game.GameLog("throne_ initialized");
    }

    public void Update(float deltaTime)
    {
        _currentScene?.Update(deltaTime);
    }

    public void Draw()
    {
        _currentScene?.Draw();
    }

    private void TransitionToScene(IScene newScene)
    {
        _currentScene?.Exit();
        _currentScene = newScene;
        _currentScene.Enter();
        _game.GameLog($"Transitioned to scene: {newScene.Name}");
    }

    private void OnStateChanged(GameState oldState, GameState newState)
    {
        _game.GameLog($"State changed: {oldState} -> {newState}");
    }
}
