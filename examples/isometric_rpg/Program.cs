using GoudEngine;

namespace IsometricRpg;

class Program
{
    static void Main(string[] args)
    {
        // Create game with 2D renderer for isometric view
        // 800x600 window, suitable for isometric gameplay
        GoudGame game = new GoudGame(800, 600, "Isometric RPG", RendererType.Renderer2D, 60);

        GameManager gameManager = new GameManager(game);

        game.Initialize(() =>
        {
            gameManager.Initialize();
        });

        game.Start(() =>
        {
            gameManager.Start();
        });

        game.Update(() =>
        {
            float deltaTime = game.UpdateResponseData.delta_time;
            gameManager.Update(deltaTime);
        });

        game.Dispose();
    }
}
