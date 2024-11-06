using System;

using CsBindgen;
class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(288, 512, "Flappy Bird");

        GameManager gameManager;

        game.Initialize(() =>
        {
            Console.WriteLine("Game Initialized!");



            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/3

            SpriteData backgroundData = new SpriteData
            {
                x = 0,
                y = 0,

                // TODO: Passing values here breaks the game
                // px_x = 800, // Set to window width
                // px_y = 600, // Set to window height
            };
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