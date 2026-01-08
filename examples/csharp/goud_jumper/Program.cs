// Program.cs
// Goud Jumper - A simple platformer demo using immediate-mode rendering

using System;

public class Program
{
    static void Main(string[] args)
    {
        var TileSize = 32;
        var width = 30;
        var height = 20;
        
        using GoudGame game = new GoudGame(
            (uint)(width * TileSize),
            (uint)(height * TileSize),
            "Goud Jumper"
        );
        
        Console.WriteLine("Goud Jumper - Controls:");
        Console.WriteLine("  A/D - Move left/right");
        Console.WriteLine("  Space - Jump");
        Console.WriteLine("  C+1/2/3 - Custom animations");
        Console.WriteLine("  ESC - Quit");
        
        GameManager gameManager = new GameManager(game);
        gameManager.Initialize();
        gameManager.Start();
        
        // Game loop - immediate-mode rendering
        while (!game.ShouldClose())
        {
            float deltaTime = game.DeltaTime;
            
            // Begin frame with sky blue background
            game.BeginFrame(0.4f, 0.6f, 0.9f, 1.0f);
            
            // Update game logic
            gameManager.Update(deltaTime);
            gameManager.HandleInput();
            
            // Draw everything
            gameManager.Draw();
            
            // End frame
            game.EndFrame();
        }
    }
}
