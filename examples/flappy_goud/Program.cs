// Program.cs

using System;
using CsBindgen;

public class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(
            GameConstants.ScreenWidth,
            GameConstants.ScreenHeight + GameConstants.BaseHeight,
            "Flappy Bird Clone"
        // RendererType.Equal
        // GameConstants.TargetFPS
        );

        GameManager gameManager = new GameManager(game);

        game.GameLog("Game initialized successfully!");

        game.Initialize(() => gameManager.Initialize());
        game.Start(() => gameManager.Start());
        game.Update(() => gameManager.Update(game.UpdateResponseData.delta_time));

        game.Terminate();
    }
}
