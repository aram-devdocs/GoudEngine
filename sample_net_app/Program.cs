using System;

class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Hello, World!");

        Movement movement = new Movement(game);

        game.Initialize(() =>
        {
            // Add a sprite and get its index (assuming index is 0)
            game.AddSprite("../sample_game/assets/bluebird-midflap.png", 0, 0, 1, 1, 0);

            // Initialize movement
            movement.AddSprite(0, 0, 0, 1, 1, 0);
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