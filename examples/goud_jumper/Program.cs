// Program.cs

using System;

public class Program
{
    static void Main(string[] args)
    {
        var TileSize = 32;
        var width = 30;
        var height = 20;
        GoudGame game = new GoudGame(
            (uint)(width * TileSize),
            (uint)(height * TileSize),
            "Game Title"
        );
        GameManager gameManager = new GameManager(game);

        game.Initialize(() => gameManager.Initialize());
        game.Start(() => gameManager.Start());
        game.Update(() =>
        {
            gameManager.Update(game.UpdateResponseData.delta_time);
            gameManager.HandleInput();
        });

        game.Terminate();
    }
}
