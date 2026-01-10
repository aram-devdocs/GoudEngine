// Program.cs
// throne_ - Main menu example with 2D/3D mixed rendering

using System;
using Throne.Core;

class Program
{
    static void Main(string[] args)
    {

        using GoudGame game = new GoudGame(
            GameManager.ScreenWidth,
            GameManager.ScreenHeight,
            "throne_"
        );

        GameManager gameManager = new GameManager(game);
        gameManager.Initialize();

        // Main game loop
        while (!game.ShouldClose())
        {
            float deltaTime = game.DeltaTime;

            // Begin frame with dark background
            game.BeginFrame(0.02f, 0.02f, 0.05f, 1.0f);

            // Update and draw
            gameManager.Update(deltaTime);
            gameManager.Draw();

            // End frame
            game.EndFrame();
        }

        Console.WriteLine("throne_ ended");
    }
}
