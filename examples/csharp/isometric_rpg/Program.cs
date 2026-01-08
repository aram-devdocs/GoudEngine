// Program.cs
// Isometric RPG - Simplified version using immediate-mode rendering

using System;
using IsometricRpg;

class Program
{
    static void Main(string[] args)
    {
        Console.WriteLine("Isometric RPG Demo");
        Console.WriteLine("==================");
        Console.WriteLine("Controls:");
        Console.WriteLine("  WASD - Move player");
        Console.WriteLine("  LMB - Attack");
        Console.WriteLine("  E - Talk to NPC");
        Console.WriteLine("  SPACE - Start game / Continue dialogue");
        Console.WriteLine("  ESC - Return to title");
        Console.WriteLine();

        using GoudGame game = new GoudGame(
            GameManager.ScreenWidth,
            GameManager.ScreenHeight,
            "Isometric RPG"
        );

        GameManager gameManager = new GameManager(game);
        gameManager.Initialize();
        gameManager.Start();

        // Game loop
        while (!game.ShouldClose())
        {
            float deltaTime = game.DeltaTime;
            
            // Begin frame
            game.BeginFrame(0.2f, 0.3f, 0.2f, 1.0f); // Dark green background
            
            // Update and draw
            gameManager.Update(deltaTime);
            gameManager.Draw();
            
            // End frame
            game.EndFrame();
        }

        Console.WriteLine("Game ended.");
    }
}
