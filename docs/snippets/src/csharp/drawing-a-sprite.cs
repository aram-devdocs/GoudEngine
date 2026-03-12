using GoudEngine;

using var game = new GoudGame(800, 600, "My Game");

ulong textureId = game.LoadTexture("assets/sprite.png");

float x = 100f;
float y = 100f;
float width = 64f;
float height = 64f;

while (!game.ShouldClose())
{
    game.BeginFrame(0.1f, 0.1f, 0.1f, 1.0f);

    if (game.IsKeyPressed(Keys.Escape))
    {
        game.Close();
        continue;
    }

    game.DrawSprite(textureId, x, y, width, height);

    game.EndFrame();
}
