// Program.cs

using System;

public class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Game Title", 60);
        GameManager gameManager = new GameManager(game);

        game.Initialize(() => gameManager.Initialize());
        game.Start(() => gameManager.Start());
        game.Update(() => gameManager.Update(game.UpdateResponseData.delta_time));

        game.Terminate();
    }
}
