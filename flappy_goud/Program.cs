using System;

using CsBindgen;
class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Flappy Bird");

        GameManager gameManager;

        game.Initialize(() =>
        {
            Console.WriteLine("Game Initialized!");



            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/3

            SpriteData backgroundData = new SpriteData { x = 0, y = 0, scale_x = 1, scale_y = 1, rotation = 0 };
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