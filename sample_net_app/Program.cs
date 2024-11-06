using System;

class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Hello, World!");

        game.Initialize(() =>
        {
            game.AddSprite("../sample_game/assets/bluebird-midflap.png", 0, 0, 1, 1, 0);
        });

        game.Run(() =>
        {
            // Update game logic here
        });

        game.Terminate();
    }
}