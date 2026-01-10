// Core/GameState.cs
// Game state enum and state manager for throne_

namespace Throne.Core;

/// <summary>
/// Game states for throne_
/// </summary>
public enum GameState
{
    MainMenu,
    Playing,
    Paused
}

/// <summary>
/// Manages game state transitions with events.
/// </summary>
public class GameStateManager
{
    public GameState CurrentState { get; private set; } = GameState.MainMenu;
    public GameState PreviousState { get; private set; } = GameState.MainMenu;

    public event Action<GameState, GameState>? OnStateChanged;

    public void SetState(GameState newState)
    {
        if (newState == CurrentState) return;

        PreviousState = CurrentState;
        CurrentState = newState;
        OnStateChanged?.Invoke(PreviousState, CurrentState);
    }

    public bool CanTransitionTo(GameState targetState)
    {
        return CurrentState switch
        {
            GameState.MainMenu => targetState == GameState.Playing,
            GameState.Playing => targetState is GameState.Paused or GameState.MainMenu,
            GameState.Paused => targetState is GameState.Playing or GameState.MainMenu,
            _ => false
        };
    }
}
