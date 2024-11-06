using System;

class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Hello, World!");

        Movement movement = new Movement(game);

        game.Initialize(() =>
        {

            GoudGame.SpriteData data = new GoudGame.SpriteData { X = 0, Y = 0, ScaleX = 0.2f, ScaleY = 0.2f, Rotation = 0 };
            // Add a sprite and get its index (assuming index is 0)
            game.AddSprite("../sample_game/assets/bluebird-midflap.png", data);

            // Initialize movement
            movement.AddSprite(0, data);
        });

        game.Start(() =>
        {
            Console.WriteLine("Game started.");
        });

        game.Update(() =>
        {
            // Update movement logic
            movement.Update();

            // Handle input (e.g., close window on Escape key)
            if (Console.KeyAvailable)
            {
                var key = Console.ReadKey(true);
                if (key.Key == ConsoleKey.Escape)
                {
                    game.Close();
                }
            }
        });

        game.Terminate();
    }
}