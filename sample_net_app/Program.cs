using System;

class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Flappy Bird");

        GameManager gameManager;

        game.Initialize(() =>
        {
            Console.WriteLine("Game Initialized!");

            // update the game manager on line 9

            GoudGame.SpriteData backgroundData = new GoudGame.SpriteData { X = 0, Y = 0, ScaleX = 1, ScaleY = 1, Rotation = 0 };
            // Background
            game.AddSprite("assets/sprites/background-day.png", backgroundData);

        });

        gameManager = new GameManager(game);


        game.Start(() =>
        {
            Console.WriteLine("Game Started!");
        });

        game.Update(() =>
        {

            gameManager.Update();
        });

        game.Terminate();
    }
}