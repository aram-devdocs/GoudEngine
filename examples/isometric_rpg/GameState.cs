namespace IsometricRpg;

public enum GameState
{
    Title,
    Playing,
    Dialogue,
    GameOver
}

public class GameStateManager
{
    public GameState CurrentState { get; private set; } = GameState.Title;
    public GameState PreviousState { get; private set; } = GameState.Title;

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
            GameState.Title => targetState == GameState.Playing,
            GameState.Playing => targetState is GameState.Dialogue or GameState.GameOver or GameState.Title,
            GameState.Dialogue => targetState == GameState.Playing,
            GameState.GameOver => targetState is GameState.Playing or GameState.Title,
            _ => false
        };
    }
}
