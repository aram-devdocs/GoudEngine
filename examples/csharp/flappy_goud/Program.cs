// Program.cs

using System;

public class Program
{
    static void Main(string[] args)
    {
        using GoudGame game = new GoudGame(
            GameConstants.ScreenWidth,
            GameConstants.ScreenHeight + (uint)GameConstants.BaseHeight,
            "Flappy Bird Clone"
        );

        GameManager gameManager = new GameManager(game);

        game.GameLog("Game initialized successfully!");

        // Initialize
        gameManager.Initialize();
        gameManager.Start();

        // Game loop - immediate-mode rendering
        while (!game.ShouldClose())
        {
            // Begin frame (poll events, clear screen)
            game.BeginFrame(0.4f, 0.7f, 0.9f, 1.0f); // Sky blue background

            // Update game logic and draw everything
            gameManager.Update(game.DeltaTime);

            // End frame (present to screen)
            game.EndFrame();
        }

        // Dispose is called automatically via 'using'
    }
}
