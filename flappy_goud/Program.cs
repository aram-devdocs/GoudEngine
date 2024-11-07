// Program.cs

using System;

public class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(GameConstants.ScreenWidth, GameConstants.ScreenHeight, "Flappy Bird Clone");
        GameManager gameManager = new GameManager(game);

        game.Initialize(() => gameManager.Initialize());
        game.Start(() => gameManager.Start());
        game.Update(() => gameManager.Update());

        game.Terminate();
    }
}